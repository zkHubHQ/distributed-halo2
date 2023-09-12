use ark_std::{end_timer, start_timer};
use dist_primitives::dfft::dfft::{d_fft, d_ifft};
use dist_primitives::dmsm::dmsm::d_msm;
use dist_primitives::utils::bn256::random_utils::create_random_group_element;
use dist_primitives::utils::deg_red::deg_red;
use dist_primitives::utils::domain_utils::EvaluationDomainExt;
use dist_primitives::utils::g1_serialization::G1Wrapper;
use ff::{PrimeField, WithSmallOrderMulGroup};
use halo2_proofs::halo2curves::pairing::Engine;
use halo2_proofs::poly::EvaluationDomain;
use rand::Rng;
use secret_sharing::pss::PackedSharingParams;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PackPolyCk<E>
where
    E: Engine + Debug,
    E::Scalar: PrimeField + WithSmallOrderMulGroup<3>,
{
    pub powers_of_tau: Vec<G1Wrapper<E>>, //We assume that we have eval version
}

impl<E> PackPolyCk<E>
where
    E: Engine + Debug,
    E::Scalar: PrimeField + WithSmallOrderMulGroup<3> + Serialize + for<'de> Deserialize<'de>,
{
    #[allow(unused)]
    pub fn new<R: Rng>(
        domain_size: usize,
        rng: &mut R,
        pp: &PackedSharingParams<E::Scalar>,
    ) -> Self {
        let powers_of_tau: Vec<G1Wrapper<E>> = (0..domain_size / pp.l)
            .map(|_| G1Wrapper(create_random_group_element::<E, R>(rng)))
            .collect();
        PackPolyCk::<E> {
            powers_of_tau: powers_of_tau,
        }
    }

    /// Interactively commits to a polynomial give packed shares of the evals
    #[allow(unused)]
    pub fn commit(&self, peval_share: &Vec<E::Scalar>, pp: &PackedSharingParams<E::Scalar>) {
        let powers_of_tau_g1: Vec<E::G1> = self
            .powers_of_tau
            .iter()
            .map(|wrapper| wrapper.0.clone())
            .collect();

        let commitment = d_msm::<E>(&powers_of_tau_g1, peval_share.as_slice(), pp);
        // actually getting back shares but king can publish the commitment
    }

    /// Interactively creates an opening to a polynomial at a chosen point
    #[allow(unused)]
    pub fn open(
        &self,
        peval_share: &Vec<E::Scalar>,
        point: E::Scalar,
        dom: &EvaluationDomain<E::Scalar>,
        pp: &PackedSharingParams<E::Scalar>,
    ) -> E::Scalar {
        debug_assert_eq!(
            peval_share.len() * pp.l,
            dom.size(),
            "pevals length is not equal to m/l"
        );
        // Interpolate pevals to get coeffs
        let pcoeff_share = d_ifft(peval_share.clone(), false, 1, false, dom, pp);

        // distributed poly evaluation
        let powers_of_r_share = E::Scalar::from(123 as u64); // packed shares of r drop from sky
        let point_eval_share = pcoeff_share
            .iter()
            .map(|&a| a * powers_of_r_share)
            .sum::<E::Scalar>();

        // do degree reduction and King publishes answer
        let point_eval_share = deg_red(vec![point_eval_share], pp)[0];

        // Compute the quotient polynomial
        // During iFFT king sends over the "truncated pcoeff_shares". Do FFT on this

        let ptrunc_evals = d_fft(pcoeff_share, false, 1, false, dom, pp);
        let toep_mat_share = E::Scalar::from(123 as u64); // packed shares of toeplitz matrix drop from sky
        let timer_div = start_timer!(|| "Division");
        let q_evals = ptrunc_evals
            .into_iter()
            .map(|a| a * toep_mat_share)
            .collect::<Vec<E::Scalar>>();
        end_timer!(timer_div);

        // don't have to do degree reduction since it's a secret value multiplied by two public values
        // we could pack two public values together but that would mean two msms instead of one

        let powers_of_tau_g1: Vec<E::G1> = self
            .powers_of_tau
            .iter()
            .map(|wrapper| wrapper.0.clone())
            .collect();
        // Compute the proof pi
        let pi: E::G1 = d_msm::<E>(&powers_of_tau_g1, &q_evals, pp);

        point_eval_share
    }
}
