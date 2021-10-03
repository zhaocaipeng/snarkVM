// Copyright (C) 2019-2021 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

#![allow(clippy::module_inception)]
#![deny(unused_import_braces, unused_qualifications, trivial_casts, trivial_numeric_casts)]
#![deny(
    single_use_lifetimes,
    unused_qualifications,
    variant_size_differences,
    stable_features,
    unreachable_pub
)]
#![deny(
    non_shorthand_field_patterns,
    unused_attributes,
    unused_imports,
    unused_extern_crates
)]
#![deny(
    renamed_and_removed_lints,
    stable_features,
    unused_allocation,
    unused_comparisons,
    bare_trait_objects
)]
#![deny(
    const_err,
    unused_must_use,
    unused_mut,
    unused_unsafe,
    private_in_public,
    unsafe_code
)]
#![forbid(unsafe_code)]
#![cfg_attr(feature = "clippy", deny(warnings))]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(inline_always))]
#![cfg_attr(feature = "clippy", allow(too_many_arguments))]
#![cfg_attr(feature = "clippy", allow(unreadable_literal))]
#![cfg_attr(feature = "clippy", allow(many_single_char_names))]
#![cfg_attr(feature = "clippy", allow(new_without_default_derive))]

#[macro_use]
extern crate thiserror;

pub mod errors;
pub use errors::*;

pub mod ledger;
pub use ledger::*;

pub mod memdb;
pub use memdb::*;

pub mod traits;
pub use traits::*;

pub mod prelude {
    pub use crate::{errors::*, traits::*};
}

use snarkvm_utilities::{FromBytes, ToBytes};

use std::io::{Read, Result as IoResult, Write};

pub const COL_META: u32 = 0; // MISC Values
pub const COL_BLOCK_HEADER: u32 = 1; // Block hash -> block header
pub const COL_BLOCK_TRANSACTIONS: u32 = 2; // Block hash -> block transactions
pub const COL_BLOCK_LOCATOR: u32 = 3; // Block num -> block hash && block hash -> block num
pub const COL_TRANSACTION_LOCATION: u32 = 4; // Transaction Hash -> (block hash and index)
pub const COL_COMMITMENT: u32 = 5; // Commitment -> index
pub const COL_SERIAL_NUMBER: u32 = 6; // SN -> index
pub const COL_BLOCK_PREVIOUS_BLOCK_HASH: u32 = 7; // Block hash -> previous block hash // TODO (howardwu): Reorder this from 7 -> 1.
pub const COL_DIGEST: u32 = 8; // Ledger digest -> index
pub const COL_RECORDS: u32 = 9; // commitment -> record bytes
pub const COL_CHILD_HASHES: u32 = 10; // block hash -> vector of potential child hashes
pub const NUM_COLS: u32 = 11;

pub const KEY_BEST_BLOCK_NUMBER: &str = "BEST_BLOCK_NUMBER";
pub const KEY_CURR_CM_INDEX: &str = "CURRENT_CM_INDEX";
pub const KEY_CURR_SN_INDEX: &str = "CURRENT_SN_INDEX";
pub const KEY_CURR_DIGEST: &str = "CURRENT_DIGEST";

/// Represents address of certain transaction within block
#[derive(Debug, PartialEq, Clone)]
pub struct TransactionLocation {
    /// Transaction index within the block
    pub index: u32,
    /// Block hash
    pub block_hash: [u8; 32],
}

impl ToBytes for TransactionLocation {
    #[inline]
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        self.index.write_le(&mut writer)?;
        self.block_hash.write_le(&mut writer)
    }
}

impl FromBytes for TransactionLocation {
    #[inline]
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        let index: u32 = FromBytes::read_le(&mut reader)?;
        let block_hash: [u8; 32] = FromBytes::read_le(&mut reader)?;

        Ok(Self { index, block_hash })
    }
}