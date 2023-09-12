use ark_std::{end_timer, start_timer, One};
use dist_primitives::utils::domain_utils::EvaluationDomainExt;
use dist_primitives::utils::{
    bn256::random_utils::create_random_group_element, g1_serialization::G1Wrapper,
};
use ff::Field;
use ff::{PrimeField, WithSmallOrderMulGroup};
use halo2_proofs::poly::commitment::MSM;
use halo2_proofs::poly::Coeff;
use halo2_proofs::{
    halo2curves::pairing::Engine,
    poly::{kzg::msm::MSMKZG, EvaluationDomain, Polynomial},
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PolyCk<E>
where
    E: Engine + Debug,
    E::Scalar: PrimeField + WithSmallOrderMulGroup<3>,
{
    pub powers_of_tau: Vec<G1Wrapper<E>>, //We assume that we have eval version
}

impl<E> PolyCk<E>
where
    E: Engine + Debug,
    E::Scalar: PrimeField + WithSmallOrderMulGroup<3> + Serialize + for<'de> Deserialize<'de>,
{
    #[allow(unused)]
    pub fn new<R: Rng>(domain_size: usize, rng: &mut R) -> Self {
        let powers_of_tau: Vec<G1Wrapper<E>> = (0..domain_size)
            .map(|_| G1Wrapper(create_random_group_element::<E, R>(rng)))
            .collect();
        PolyCk::<E> {
            powers_of_tau: powers_of_tau,
        }
    }

    /// Commits to a polynomial give the evals
    #[allow(unused)]
    pub fn commit(&self, pevals: &Vec<E::Scalar>) {
        let msm_time = start_timer!(|| "PolyCom MSM");
        let mut msm = MSMKZG::<E>::new();

        for (base, scalar) in self.powers_of_tau.iter().zip(pevals.iter()) {
            msm.append_term(*scalar, base.0)
        }

        let should_be_output = msm.eval();
        end_timer!(msm_time);
    }

    /// Creates an opening to a polynomial at a chosen point
    #[allow(unused)]
    pub fn open(
        &self,
        pevals: &Vec<E::Scalar>,
        point: E::Scalar,
        dom: &EvaluationDomain<E::Scalar>,
    ) -> E::Scalar {
        debug_assert_eq!(pevals.len(), dom.size(), "pevals length is not equal to m");
        let open_timer = start_timer!(|| "PolyCom Open");
        // Interpolate pevals to get coeffs
        let pcoeffs = Self::ifft(&mut pevals.clone(), dom);
        let p = Polynomial::<E::Scalar, Coeff>::from_coefficients_vec(pcoeffs);
        let point_eval = p.evaluate(&point); // Evaluate pcoeffs at point

        // Compute the quotient polynomial
        let p = Polynomial::from(p);
        let divisor =
            Polynomial::from(Polynomial::<E::Scalar, Coeff>::from_coefficients_vec(vec![
                -point,
                E::Scalar::ONE,
            ]));
        let mut qcoeffs = p.divide_with_q_and_r(&divisor).1.to_vec();

        // convert to evals
        let qevals = Self::fft(&mut qcoeffs, &dom);

        // Compute the proof pi
        let powers_of_tau_g1: Vec<E::G1> = self
            .powers_of_tau
            .iter()
            .map(|wrapper| wrapper.0.clone())
            .collect();
        let mut msm = MSMKZG::<E>::new();

        for (base, scalar) in powers_of_tau_g1.iter().zip(qevals.iter()) {
            msm.append_term(*scalar, *base)
        }

        let should_be_output = msm.eval();
        end_timer!(open_timer);

        point_eval
    }

    // Custom FFT function using halo2's EvaluationDomain
    pub fn fft<F>(poly: &mut Vec<F>, domain: &EvaluationDomain<F>) -> Vec<F>
    where
        F: PrimeField + WithSmallOrderMulGroup<3>,
    {
        poly.resize(1 << domain.k(), F::ZERO);
        let coeff_poly = domain.coeff_from_vec(poly.clone());
        let lagrange_poly = domain.coeff_to_extended(coeff_poly);
        lagrange_poly.iter().cloned().collect()
    }

    // Custom IFFT function using halo2's EvaluationDomain
    pub fn ifft<F>(poly: &mut Vec<F>, domain: &EvaluationDomain<F>) -> Vec<F>
    where
        F: PrimeField + WithSmallOrderMulGroup<3>,
    {
        poly.resize(1 << domain.k(), F::ZERO);
        let lagrange_poly = domain.lagrange_from_vec(poly.clone());
        let coeff_poly = domain.lagrange_to_coeff(lagrange_poly);
        coeff_poly.iter().cloned().collect()
    }
}
