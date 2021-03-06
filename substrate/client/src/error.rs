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

//! Polkadot client possible errors.

use std;
use state_machine;
use primitives::hexdisplay::HexDisplay;

error_chain! {
	errors {
		/// Backend error.
		Backend(s: String) {
			description("Unrecoverable backend error"),
			display("Backend error: {}", s),
		}

		/// Unknown block.
		UnknownBlock(h: ::primitives::block::Id) {
			description("unknown block"),
			display("UnknownBlock: {}", h),
		}

		/// Execution error.
		Execution(e: Box<state_machine::Error>) {
			description("execution error"),
			display("Execution: {}", e),
		}

		/// Blockchain error.
		Blockchain(e: Box<std::error::Error + Send>) {
			description("Blockchain error"),
			display("Blockchain: {}", e),
		}

		/// Attempt to read storage yet nothing set for that key.
		NoValueForKey(key: Vec<u8>) {
			description("storage doesn't contain key"),
			display("Storage does not contain the key entry: {}", HexDisplay::from(key)),
		}

		/// Invalid state data.
		AuthLenEmpty {
			description("authority count state error"),
			display("Current state of blockchain has no authority count value"),
		}

		/// Invalid state data.
		AuthEmpty(i: u32) {
			description("authority value state error"),
			display("Current state of blockchain has no authority value for index {}", i),
		}

		/// Invalid state data.
		AuthLenInvalid {
			description("authority count state error"),
			display("Current state of blockchain has invalid authority count value"),
		}

		/// Invalid state data.
		AuthInvalid(i: u32) {
			description("authority value state error"),
			display("Current state of blockchain has invalid authority value for index {}", i),
		}

		/// Bad justification for header.
		BadJustification(h: ::primitives::block::Id) {
			description("bad justification for header"),
			display("bad justification for header: {}", h),
		}
	}
}

// TODO [ToDr] Temporary, state_machine::Error should be a regular error not Box.
impl From<Box<state_machine::Error>> for Error {
	fn from(e: Box<state_machine::Error>) -> Self {
		ErrorKind::Execution(e).into()
	}
}

impl From<state_machine::backend::Void> for Error {
	fn from(_e: state_machine::backend::Void) -> Self {
		unreachable!()
	}
}

impl Error {
	/// Chain a blockchain error.
	pub fn from_blockchain(e: Box<std::error::Error + Send>) -> Self {
		ErrorKind::Blockchain(e).into()
	}
}
