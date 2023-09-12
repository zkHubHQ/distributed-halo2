use ff::{PrimeField, WithSmallOrderMulGroup};
use halo2_proofs::poly::EvaluationDomain;

pub mod dplonk;
pub mod dpoly_commit;
pub mod localplonk;
pub mod poly_commit;

#[derive(Debug, Clone)]
pub struct PlonkDomain<F>
where
    F: PrimeField + WithSmallOrderMulGroup<3>,
{
    pub n_gates: usize,
    pub gates: EvaluationDomain<F>,
    pub gates8: EvaluationDomain<F>,
}

fn get_k_value_for_domain(n: usize) -> u32 {
    (n as f64).log2() as u32
}

fn get_domain_size<F>(domain: &EvaluationDomain<F>) -> usize
where
    F: PrimeField + WithSmallOrderMulGroup<3>,
{
    1 << domain.k() as usize
}

impl<F> PlonkDomain<F>
where
    F: PrimeField + WithSmallOrderMulGroup<3>,
{
    #[allow(unused)]
    pub fn new(n_gates: usize) -> Self {
        let gates = EvaluationDomain::<F>::new(1, get_k_value_for_domain(n_gates));
        let gates8 = EvaluationDomain::<F>::new(1, get_k_value_for_domain(8 * n_gates));

        debug_assert_eq!(get_domain_size(&gates), n_gates);
        debug_assert_eq!(get_domain_size(&gates8), 8 * n_gates);

        PlonkDomain {
            n_gates,
            gates,
            gates8,
        }
    }
}
