// Copyright (C) 2019-2022 Aleo Systems Inc.
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

use super::*;
use snarkvm_circuits_environment::Constant;

impl<E: Environment> FromBitsBE for Group<E> {
    type Boolean = Boolean<E>;

    /// Initializes a new group element from the x-coordinate as a list of big-endian bits *without* leading zeros.
    fn from_bits_be(bits_be: &[Self::Boolean]) -> Self {
        // Derive the x-coordinate for the affine group element.
        let x = Field::from_bits_be(bits_be);
        // Recover the y-coordinate and return the affine group element.
        Self::from_x_coordinate(x)
    }
}

impl<E: Environment> Metadata<dyn FromBitsBE<Boolean = Boolean<E>>> for Group<E> {
    type Case = Vec<CircuitType<Boolean<E>>>;
    type OutputType = CircuitType<Self>;

    fn count(case: &Self::Case) -> Count {
        match case.iter().all(|bit| bit.is_constant()) {
            true => Count::is(3, 0, 0, 0),
            false => Count::is(2, 0, 255, 421),
        }
    }

    fn output_type(case: Self::Case) -> Self::OutputType {
        let mut bits_be = Vec::with_capacity(case.len());
        for bit in case {
            match bit {
                CircuitType::Constant(constant) => bits_be.push(constant.circuit()),
                _ => return CircuitType::Private,
            }
        }
        CircuitType::from(Group::from_bits_be(bits_be.as_slice()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_circuits_environment::Circuit;
    use snarkvm_utilities::{test_rng, UniformRand};

    const ITERATIONS: u64 = 100;

    fn check_from_bits_be(mode: Mode) {
        for i in 0..ITERATIONS {
            // Sample a random element.
            let expected: <Circuit as Environment>::Affine = UniformRand::rand(&mut test_rng());
            let given_bits = Group::<Circuit>::new(mode, expected).to_bits_be();

            Circuit::scope(&format!("{} {}", mode, i), || {
                let candidate = Group::<Circuit>::from_bits_be(&given_bits);
                assert_eq!(expected, candidate.eject_value());

                let case = given_bits.iter().map(|bit| CircuitType::from(bit)).collect();
                assert_count!(Group<Circuit>, FromBitsBE<Boolean = Boolean<Circuit>>, &case);
                assert_output_type!(Group<Circuit>, FromBitsBE<Boolean = Boolean<Circuit>>, case, candidate);
            });
            Circuit::reset();
        }
    }

    #[test]
    fn test_from_bits_be_constant() {
        check_from_bits_be(Mode::Constant);
    }

    #[test]
    fn test_from_bits_be_public() {
        check_from_bits_be(Mode::Public);
    }

    #[test]
    fn test_from_bits_be_private() {
        check_from_bits_be(Mode::Private);
    }
}
