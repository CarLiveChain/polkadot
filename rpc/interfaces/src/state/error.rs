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

#![allow(missing_docs)]

use rpc;

error_chain! {
	errors {
		/// Internal error (TODO [ToDr] Temporary)
		Internal {
			description("internal error"),
			display("Internal Error"),
		}
	}
}

impl From<Error> for rpc::Error {
	fn from(e: Error) -> Self {
		match e {
			_ => rpc::Error::internal_error(),
		}
	}
}
