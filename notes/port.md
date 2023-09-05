# Notes from the migration

## mpc-net

No Arkworks dependencies, no code changes done

## secret-sharing

- **pss.rs**
  - Change Radix2EvaluationDomain to halo2_proofs::poly::EvaluationDomain
  - Use ff::PrimeField instead of FftField
  - Use new logic for the EvakuationDomain builder method that accepts 2 inputs instead of 1 by hard coding the extended domain factor (1st arg) to 1, and the log2 of the number of coeff as the second arg, hence reproducing the existing Radix2EvaluationDomain logic
  - Method for finding Evaluation domain size does not exist in a straight forwad way like ark-poly, hence use the approach of 1 << share.k() to get the domain size
  - **Important**: Created helper method for fft/ifft in place for supporting the packing and unpacking of secrets and shares like zksaas as the same functions did not exist in halo2 polynomial libraries.
  Encountered lots of subtle gotchas:
    - Need to resize the vector list to match the eval domain, and then `coeff_to_extended`/`lagrange_to_coeff` methods for fft/ifft respectively
    - This new approach resulted in refactoring the logic in `unpack2_in_place` method's last line: `*shares = shares.iter().filter(|&&x| x != F::ZERO).copied().collect();`
  - Changed from BLS12-377 to BN256
    - Does not BN256 lib does expose a random Fr function, hence created helper method that's only used in the test cases
  - TODO: WithSmallOrderMulGroup<3>, 3 was arbitrarily chosen and needs to be changed to another value with more thought

## dist-primitives

- **domain_util.rs**
  - Created a trait called EvaluationDomainExt, and implemented it for EvaluationDomain. Meant for adding the extension method size() to the EvaluationDomain triat, rather than using "ugly math" all the time (1 << domain.k())
- **channel.rs**
  - Changed ark-serialize to serde's serialize
  - Using bincode for the type of data that gets serialized/deserialized
  - HRTB for the Deserialize trait
- **dpp.rs**
  - Update FftField to PrimeField
  - Add generic trait bound to most methods - where F: PrimeField + WithSmallOrderMulGroup<3> + Serialize + for<'de> Deserialize<'de>
  - Use the invert() method in EvaluationDomain instead of inverse()
- **dmsm.rs**
  - Change G::ScalarField to G::Scalar (Scalar is an alias for PrimeField within the group trait)
  - Change G::ScalarField to G::Scalar (Scalar is an alias for PrimeField within the group trait)
- **dfft.rs**
  - Change Radix2EvaluationDomain to EvaluationDomain
  - Change F::zero() to F::ZERO
  - Change F::one() to F::ONE
  - Import EvaluationDomainExt to access the size method extension
  - Add generic trait bound to most methods - where F: PrimeField + WithSmallOrderMulGroup<3> + Serialize + for<'de> Deserialize<'de>
  - Use the invert() method in EvaluationDomain instead of inverse()
  - Change group_gen_inv to get_omega_inv()
- **deg_red.rs**
  - Add generic trait bound to most methods - where F: PrimeField + WithSmallOrderMulGroup<3> + Serialize + for<'de>
- **pack.rs**
  - Add generic trait bound- where F: PrimeField + WithSmallOrderMulGroup<3>
