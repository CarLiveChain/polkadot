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

//! Polkadot Client data backend

use state_machine;
use error;
use primitives::block::{self, Id as BlockId};
use primitives;

/// Block insertion operation. Keeps hold if the inserted block state and data.
pub trait BlockImportOperation {
	/// Associated state backend type.
	type State: state_machine::backend::Backend;

	/// Returns pending state. Returns None for backends with locally-unavailable state data.
	fn state(&self) -> error::Result<Option<&Self::State>>;
	/// Append block data to the transaction.
	fn set_block_data(&mut self, header: block::Header, body: Option<block::Body>, justification: Option<primitives::bft::Justification>, is_new_best: bool) -> error::Result<()>;
	/// Inject storage data into the database.
	fn set_storage<I: Iterator<Item=(Vec<u8>, Option<Vec<u8>>)>>(&mut self, changes: I) -> error::Result<()>;
	/// Inject storage data into the database replacing any existing data.
	fn reset_storage<I: Iterator<Item=(Vec<u8>, Vec<u8>)>>(&mut self, iter: I) -> error::Result<()>;
}

/// Client backend. Manages the data layer.
pub trait Backend: Send + Sync {
	/// Associated block insertion operation type.
	type BlockImportOperation: BlockImportOperation;
	/// Associated blockchain backend type.
	type Blockchain: ::blockchain::Backend;
	/// Associated state backend type.
	type State: state_machine::backend::Backend;

	/// Begin a new block insertion transaction with given parent block id.
	fn begin_operation(&self, block: BlockId) -> error::Result<Self::BlockImportOperation>;
	/// Commit block insertion.
	fn commit_operation(&self, transaction: Self::BlockImportOperation) -> error::Result<()>;
	/// Returns reference to blockchain backend.
	fn blockchain(&self) -> &Self::Blockchain;
	/// Returns state backend for specified block.
	fn state_at(&self, block: BlockId) -> error::Result<Self::State>;
}

/// Mark for all Backend implementations, that are making use of state data, stored locally.
pub trait LocalBackend: Backend {}

/// Mark for all Backend implementations, that are fetching required state data from remote nodes.
pub trait RemoteBackend: Backend {}
