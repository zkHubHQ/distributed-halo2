use ff::{Field, PrimeField, WithSmallOrderMulGroup};
use group::Group;
use halo2_proofs::{arithmetic::best_fft, halo2curves::pairing::Engine, poly::EvaluationDomain};
use std::fmt::Debug;

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

// Helper function for FFT on projective group elements
pub fn fft_on_group_elements<E>(values: &mut Vec<E::G1>, domain: &EvaluationDomain<E::Scalar>)
where
    E: Engine + Debug,
    E::Scalar: PrimeField + WithSmallOrderMulGroup<3>,
{
    // resize to domain size
    values.resize(domain.size(), E::G1::identity());
    let omega = domain.get_omega();
    let log_n = domain.size().trailing_zeros();
    best_fft(values, omega, log_n);
}

// Helper function for IFFT on projective group elements
pub fn ifft_on_group_elements<E>(values: &mut Vec<E::G1>, domain: &EvaluationDomain<E::Scalar>)
where
    E: Engine + Debug,
    E::Scalar: PrimeField + WithSmallOrderMulGroup<3>,
{
    // resize to domain size
    values.resize(domain.size(), E::G1::identity());
    let omega_inv = domain.get_omega_inv();
    let log_n = domain.size().trailing_zeros();
    best_fft(values, omega_inv, log_n);
    let n_inv = E::Scalar::from(domain.size() as u64).invert().unwrap();
    for v in values.iter_mut() {
        *v *= n_inv;
    }
}
