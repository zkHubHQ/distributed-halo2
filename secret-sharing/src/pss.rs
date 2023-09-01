use halo2_proofs::arithmetic::Field;

use halo2_proofs::poly::EvaluationDomain;

pub struct PackedSharingParams<F> 
    where F: Field
{
    pub t: usize,                     // Corruption threshold
    pub l: usize,                     // Packing factor
    pub n: usize,                     // Number of parties
    pub share: EvaluationDomain<F>,   // Share domain
    pub secret: EvaluationDomain<F>,  // Secrets domain
    pub secret2: EvaluationDomain<F>, // Secrets2 domain
} 

impl<F: Field> PackedSharingParams<F> {
    #[allow(unused)]
    pub fn new(l) -> Self {
        let n = l * 4;
        let t = l - 1;
        debug_assert_eq!(n, 2 * (t + l + 1));

        let share = EvaluationDomain::<F>::new(n).unwrap(); // TODO change fn new takes in 2 params
        let secret = EvaluationDomain::<F>::new(l + t + 1)
        .unwrap()
        .get_coset(F::GENERATOR)
        .unwrap();
        let secret2 = EvaluationDomain::<F>::new(2 * (l + t + 1))
        .unwrap()
        .get_coset(F::GENERATOR)
        .unwrap();

        debug_assert_eq!((2 ^ share.k), n); //  (share.size()) => 2^(share.k)
        debug_assert_eq!((2 ^ secret.k), l + t + 1);
        debug_assert_eq!(2 ^ secret2.k , 2 * (l + t + 1));

        PackedSharingParams {
            t,
            l,
            n,
            share,
            secret,
            secret2,
        }
    }
}
