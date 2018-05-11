// Copyright 2017 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

use std::sync::Arc;
use primitives::block::Id as BlockId;
use state_machine::{self, OverlayedChanges, Backend as StateBackend, CodeExecutor};
use state_machine::backend::InMemory as InMemoryStateBackend;
use triehash::trie_root;

use backend;
use blockchain::Backend as ChainBackend;
use error;
use light::Fetcher;

/// Information regarding the result of a call.
#[derive(Debug)]
pub struct CallResult {
	/// The data that was returned from the call.
	pub return_data: Vec<u8>,
	/// The changes made to the state by the call.
	pub changes: OverlayedChanges,
}

/// Method call executor.
pub trait CallExecutor {
	/// Externalities error type.
	type Error: state_machine::Error;

	/// Execute a call to a contract on top of state in a block of given hash.
	///
	/// No changes are made.
	fn call(&self, id: &BlockId, method: &str, call_data: &[u8]) -> Result<CallResult, error::Error>;

	/// Execute a call to a contract on top of given state.
	///
	/// No changes are made.
	fn call_at_state<S: state_machine::Backend>(&self, state: &S, overlay: &mut OverlayedChanges, method: &str, call_data: &[u8]) -> Result<Vec<u8>, error::Error>;
}

/// Call executor that executes methods locally, querying all required
/// data from local backend.
pub struct LocalCallExecutor<B, E> {
	backend: Arc<B>,
	executor: E,
}

/// Call executor that executes methods on remote node, querying execution proof
/// and checking proof by re-executing locally.
pub struct RemoteCallExecutor<B, E> {
	backend: Arc<B>,
	executor: E,
	fetcher: Arc<Fetcher>,
}

impl<B, E> LocalCallExecutor<B, E> {
	/// Creates new instance of local call executor.
	pub fn new(backend: Arc<B>, executor: E) -> Self {
		LocalCallExecutor { backend, executor }
	}
}

impl<B, E> Clone for LocalCallExecutor<B, E> where E: Clone {
	fn clone(&self) -> Self {
		LocalCallExecutor {
			backend: self.backend.clone(),
			executor: self.executor.clone(),
		}
	}
}

impl<B, E> CallExecutor for LocalCallExecutor<B, E>
	where
		B: backend::LocalBackend,
		E: CodeExecutor,
		error::Error: From<<<B as backend::Backend>::State as StateBackend>::Error>,
{
	type Error = E::Error;

	fn call(&self, id: &BlockId, method: &str, call_data: &[u8]) -> error::Result<CallResult> {
		let mut changes = OverlayedChanges::default();
		let return_data = self.call_at_state(&self.backend.state_at(*id)?, &mut changes, method, call_data)?;
		Ok(CallResult{ return_data, changes })
	}

	fn call_at_state<S: state_machine::Backend>(&self, state: &S, changes: &mut OverlayedChanges, method: &str, call_data: &[u8]) -> error::Result<Vec<u8>> {
		state_machine::execute(
			state,
			changes,
			&self.executor,
			method,
			call_data,
		).map_err(Into::into)
	}
}

impl<B, E> RemoteCallExecutor<B, E> {
	/// Creates new instance of remote call executor.
	pub fn new(backend: Arc<B>, executor: E, fetcher: Arc<Fetcher>) -> Self {
		RemoteCallExecutor { backend, executor, fetcher }
	}
}

impl<B, E> CallExecutor for RemoteCallExecutor<B, E>
	where
		B: backend::RemoteBackend,
		E: CodeExecutor,
		error::Error: From<<<B as backend::Backend>::State as StateBackend>::Error>,
{
	type Error = E::Error;

	fn call(&self, id: &BlockId, method: &str, call_data: &[u8]) -> error::Result<CallResult> {
		let block_hash = match *id {
			BlockId::Hash(hash) => hash,
			BlockId::Number(number) => self.backend.blockchain().hash(number)?
				.ok_or_else(|| error::ErrorKind::UnknownBlock(BlockId::Number(number)))?,
		};

		let (remote_result, remote_proof) = self.fetcher.execution_proof(block_hash, method, call_data)?;

		// code below will be replaced with proper proof check once trie-based proofs will be possible

		let remote_state = state_from_execution_proof(remote_proof);
		let remote_state_root = trie_root(remote_state.pairs().into_iter()).0;

		let local_header = self.backend.blockchain().header(BlockId::Hash(block_hash))?;
		let local_header = local_header.ok_or_else(|| error::ErrorKind::UnknownBlock(BlockId::Hash(block_hash)))?;
		let local_state_root = local_header.state_root;

		if remote_state_root != *local_state_root {
			return Err(error::ErrorKind::InvalidExecutionProof.into());
		}

		let mut changes = OverlayedChanges::default();
		let local_result = state_machine::execute(
			&remote_state,
			&mut changes,
			&self.executor,
			method,
			call_data,
		)?;

		if local_result != remote_result {
			return Err(error::ErrorKind::InvalidExecutionProof.into());
		}

		Ok(CallResult { return_data: local_result, changes })
	}

	fn call_at_state<S: state_machine::Backend>(&self, _state: &S, _changes: &mut OverlayedChanges, _method: &str, _call_data: &[u8]) -> error::Result<Vec<u8>> {
		Err(error::ErrorKind::NotAvailableOnLightClient.into())
	}
}

/// Convert state to execution proof. Proof is simple the whole state (temporary).
// TODO [light]: this method must be removed after trie-based proofs are landed.
pub fn state_to_execution_proof<B: state_machine::Backend>(state: &B) -> Vec<Vec<u8>> {
	state.pairs().into_iter()
		.flat_map(|(k, v)| ::std::iter::once(k).chain(::std::iter::once(v)))
		.collect()
}

/// Convert execution proof to in-memory state for check. Reverse function for state_to_execution_proof.
// TODO [light]: this method must be removed after trie-based proofs are landed.
fn state_from_execution_proof(proof: Vec<Vec<u8>>) -> InMemoryStateBackend {
	let mut state = InMemoryStateBackend::new();
	let mut proof_iter = proof.into_iter();
	loop {
		let key = proof_iter.next();
		let value = proof_iter.next();
		if let (Some(key), Some(value)) = (key, value) {
			state.insert(key, value);
		} else {
			break;
		}
	}

	state
}
