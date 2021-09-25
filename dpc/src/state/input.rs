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

use crate::prelude::*;
use snarkvm_algorithms::CommitmentScheme;

use anyhow::{anyhow, Result};
use rand::{CryptoRng, Rng};

#[derive(Derivative)]
#[derivative(Clone(bound = "C: Parameters"))]
pub struct Input<C: Parameters> {
    record: Record<C>,
    serial_number: C::SerialNumber,
    noop_private_key: Option<PrivateKey<C>>,
    executable: Executable<C>,
}

impl<C: Parameters> Input<C> {
    pub fn new_noop<R: Rng + CryptoRng>(rng: &mut R) -> Result<Self> {
        // Construct the noop executable.
        let executable = Executable::Noop;

        // Sample a burner noop private key.
        let noop_private_key = PrivateKey::new(rng);
        let noop_compute_key = noop_private_key.to_compute_key()?;
        let noop_address = Address::from_private_key(&noop_private_key)?;

        // Construct the noop input record.
        let record = Record::new_noop_input(noop_address, rng)?;

        // Compute the serial number.
        let serial_number = record.to_serial_number(&noop_compute_key)?;

        Ok(Self {
            record,
            serial_number,
            noop_private_key: Some(noop_private_key),
            executable,
        })
    }

    /// Initializes a new instance of `Input`.
    pub fn new(compute_key: &ComputeKey<C>, record: Record<C>, executable: Option<Executable<C>>) -> Result<Self> {
        // Ensure the account address matches.
        if Address::from_compute_key(compute_key)? != record.owner() {
            return Err(anyhow!("Address from compute key does not match the record owner"));
        }

        // Retrieve the executable. If `None` is provided, construct the noop executable.
        let executable = match executable {
            Some(executable) => executable,
            None => Executable::Noop,
        };

        // Ensure its program ID matches what is declared in the record.
        if executable.program_id() != record.program_id() {
            return Err(anyhow!("Executable program ID does not match record program ID"));
        }

        // Compute the serial number.
        let serial_number = record.to_serial_number(&compute_key)?;

        Ok(Self {
            record,
            serial_number,
            noop_private_key: None,
            executable,
        })
    }

    /// Initializes a new instance of `Input`.
    pub fn new_full(
        compute_key: &ComputeKey<C>,
        value: AleoAmount,
        payload: Payload,
        executable: Executable<C>,
        serial_number_nonce: C::SerialNumberNonce,
        commitment_randomness: <C::RecordCommitmentScheme as CommitmentScheme>::Randomness,
    ) -> Result<Self> {
        // Derive the account address.
        let address = Address::from_compute_key(compute_key)?;

        // Determine if the record is a dummy.
        let is_dummy = value == AleoAmount::from_bytes(0) && payload.is_empty() && executable.is_noop();

        // Construct the input record.
        let record = Record::new_input(
            executable.program_id(),
            address,
            is_dummy,
            value.0 as u64,
            payload,
            serial_number_nonce,
            commitment_randomness,
        )?;

        // Compute the serial number.
        let serial_number = record.to_serial_number(&compute_key)?;

        Ok(Self {
            record,
            serial_number,
            noop_private_key: None,
            executable,
        })
    }

    /// Returns a reference to the input record.
    pub fn record(&self) -> &Record<C> {
        &self.record
    }

    /// Returns a reference to the input serial number.
    pub fn serial_number(&self) -> &C::SerialNumber {
        &self.serial_number
    }

    /// Returns a reference to the noop private key, if it exists.
    pub fn noop_private_key(&self) -> &Option<PrivateKey<C>> {
        &self.noop_private_key
    }

    /// Returns a reference to the executable.
    pub fn executable(&self) -> &Executable<C> {
        &self.executable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testnet2::*;

    use rand::{thread_rng, SeedableRng};
    use rand_chacha::ChaChaRng;

    const ITERATIONS: usize = 100;

    #[test]
    fn test_new_noop() {
        for _ in 0..ITERATIONS {
            // Sample a random seed for the RNG.
            let seed: u64 = thread_rng().gen();

            // Generate the expected input state.
            let (expected_record, expected_serial_number, expected_noop_private_key) = {
                let rng = &mut ChaChaRng::seed_from_u64(seed);

                let account = Account::new(rng).unwrap();
                let input_record = Record::new_noop_input(account.address, rng).unwrap();
                let serial_number = input_record
                    .to_serial_number(&account.private_key().to_compute_key().unwrap())
                    .unwrap();
                (input_record, serial_number, account.private_key().clone())
            };

            // Generate the candidate input state.
            let (candidate_record, candidate_serial_number, candidate_noop_private_key, candidate_executable) = {
                let rng = &mut ChaChaRng::seed_from_u64(seed);
                let input = Input::<Testnet2Parameters>::new_noop(rng).unwrap();
                (
                    input.record().clone(),
                    input.serial_number().clone(),
                    input.noop_private_key().clone(),
                    input.executable().clone(),
                )
            };

            assert_eq!(expected_record, candidate_record);
            assert_eq!(expected_serial_number, candidate_serial_number);
            assert_eq!(Some(expected_noop_private_key), candidate_noop_private_key);
            assert!(candidate_executable.is_noop());
        }
    }
}