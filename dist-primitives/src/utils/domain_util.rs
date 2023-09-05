use ff::{PrimeField, WithSmallOrderMulGroup};
use halo2_proofs::poly::EvaluationDomain;

pub trait EvaluationDomainExt<F: PrimeField + WithSmallOrderMulGroup<3>> {
    fn size(&self) -> usize;
}

impl<F: PrimeField + WithSmallOrderMulGroup<3>> EvaluationDomainExt<F> for EvaluationDomain<F> {
    fn size(&self) -> usize
    where
        F: WithSmallOrderMulGroup<3>,
    {
        1 << self.k() as usize
    }
}
