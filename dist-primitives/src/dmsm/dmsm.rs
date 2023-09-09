use std::fmt::Debug;

use crate::utils::g1_serialization::G1Wrapper;
use crate::{channel::channel::MpcSerNet, utils::domain_utils};
use ark_std::{end_timer, start_timer};
use ff::{PrimeField, WithSmallOrderMulGroup};
use group::Group;
use halo2_proofs::{
    halo2curves::pairing::Engine,
    poly::{commitment::MSM, kzg::msm::MSMKZG},
};
use mpc_net::{MpcMultiNet as Net, MpcNet};
use secret_sharing::pss::PackedSharingParams;

pub fn unpackexp<E>(
    shares: &Vec<E::G1>,
    degree2: bool,
    pp: &PackedSharingParams<E::Scalar>,
) -> Vec<E::G1>
where
    E: Engine + Debug,
    E::Scalar: PrimeField + WithSmallOrderMulGroup<3>,
{
    let mut result = shares.to_vec();

    // interpolate shares
    domain_utils::ifft_on_group_elements::<E>(&mut result, &pp.share);

    // Simplified this assertion using a zero check in the last n - d - 1 entries

    #[cfg(debug_assertions)]
    {
        let n = Net::n_parties();
        let d: usize;
        if degree2 {
            d = 2 * (pp.t + pp.l)
        } else {
            d = pp.t + pp.l
        }

        for i in d + 1..n {
            let is_identity: bool = result[i].is_identity().into();
            debug_assert!(is_identity, "Polynomial has degree > degree bound {})", d);
        }
    }

    // Evalaute the polynomial on the coset to recover secrets
    if degree2 {
        domain_utils::fft_on_group_elements::<E>(&mut result, &pp.secret2);
        result[0..pp.l * 2]
            .iter()
            .step_by(2)
            .copied()
            .collect::<Vec<_>>()
    } else {
        domain_utils::fft_on_group_elements::<E>(&mut result, &pp.secret);
        result[0..pp.l].to_vec()
    }
}

pub fn packexp_from_public<E>(
    secrets: &Vec<E::G1>,
    pp: &PackedSharingParams<E::Scalar>,
) -> Vec<E::G1>
where
    E: Engine + Debug,
    E::Scalar: PrimeField + WithSmallOrderMulGroup<3>,
{
    debug_assert_eq!(secrets.len(), pp.l);

    let mut result = secrets.to_vec();
    // interpolate secrets
    domain_utils::ifft_on_group_elements::<E>(&mut result, &pp.secret);

    // evaluate polynomial to get shares
    domain_utils::fft_on_group_elements::<E>(&mut result, &pp.share);

    result
}

pub fn d_msm<E>(
    bases: &[E::G1],
    scalars: &[E::Scalar],
    pp: &PackedSharingParams<E::Scalar>,
) -> E::G1
where
    E: Engine + Debug,
    E::Scalar: PrimeField + WithSmallOrderMulGroup<3>,
{
    // Using affine is important because we don't want to create an extra vector for converting Projective to Affine.
    // Eventually we do have to convert to Projective but this will be pp.l group elements instead of m()

    // Ensure bases and scalars have the same length
    assert_eq!(bases.len(), scalars.len());

    // First round of local computation done by parties
    println!("bases: {}, scalars: {}", bases.len(), scalars.len());
    let basemsm_timer = start_timer!(|| "Base MSM");

    let mut msm = MSMKZG::<E>::new();

    for (base, scalar) in bases.iter().zip(scalars.iter()) {
        msm.append_term(*scalar, *base)
    }

    let c_share = msm.eval();

    end_timer!(basemsm_timer);
    // Now we do degree reduction -- psstoss
    // Send to king who reduces and sends shamir shares (not packed).
    // Should be randomized. First convert to projective share.

    let king_answer: Option<Vec<G1Wrapper<E>>> =
        Net::send_to_king(&G1Wrapper(c_share)).map(|wrapped_shares: Vec<G1Wrapper<E>>| {
            let shares: Vec<E::G1> = wrapped_shares
                .into_iter()
                .map(|wrapper| wrapper.0)
                .collect();
            let output: E::G1 = unpackexp::<E>(&shares, true, pp).iter().sum();
            vec![G1Wrapper(output); Net::n_parties()]
        });

    let received_answer: G1Wrapper<E> = Net::recv_from_king(king_answer);

    received_answer.0
}

#[cfg(test)]
mod tests {
    use halo2_proofs::halo2curves::bn256::{Bn256, Fr};
    use halo2_proofs::poly::commitment::MSM;
    use halo2_proofs::poly::kzg::msm::MSMKZG;
    use secret_sharing::pss::PackedSharingParams;

    type G1P = <Bn256 as halo2_proofs::halo2curves::pairing::Engine>::G1;
    type E = Bn256;
    type F = Fr;

    use crate::dmsm::dmsm::packexp_from_public;
    use crate::dmsm::dmsm::unpackexp;
    use crate::utils::pack::transpose;

    const L: usize = 2;
    const N: usize = L * 4;
    // const T:usize = N/2 - L - 1;
    const M: usize = 1 << 8;

    // Helper function to emulate the behavior of F::random.
    fn random_fr<R: rand::Rng>(rng: &mut R) -> F {
        let values = [
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
        ];

        F::from_raw(values)
    }

    fn create_random_group_elements<R: rand::Rng>(rng: &mut R, size: usize) -> Vec<G1P> {
        (0..size)
            .map(|_| {
                let random_scalar: F = random_fr(rng);
                G1P::default() * random_scalar
            })
            .collect()
    }

    #[test]
    fn pack_unpack_test() {
        let pp = PackedSharingParams::<F>::new(L);
        let rng = &mut ark_std::test_rng();
        // Generate random secrets
        let secrets = create_random_group_elements(rng, L);

        let shares = packexp_from_public::<E>(&secrets, &pp);
        let result = unpackexp::<E>(&shares, false, &pp);

        assert_eq!(secrets, result);
    }

    #[test]
    fn pack_unpack2_test() {
        let pp = PackedSharingParams::<F>::new(L);
        let rng = &mut ark_std::test_rng();

        let gsecrets = create_random_group_elements(rng, M);

        let fsecrets: [F; M] = [F::from(1 as u64); M];
        let fsecrets = fsecrets.to_vec();

        ///////////////////////////////////////
        let mut msm = MSMKZG::<E>::new();
        for (base, scalar) in gsecrets.iter().zip(fsecrets.iter()) {
            msm.append_term(*scalar, *base)
        }
        let expected = msm.eval();
        ///////////////////////////////////////
        let gshares: Vec<Vec<G1P>> = gsecrets
            .chunks(L)
            .map(|s| packexp_from_public::<E>(&s.to_vec(), &pp))
            .collect();

        let fshares: Vec<Vec<F>> = fsecrets
            .chunks(L)
            .map(|s| pp.pack_from_public(&s.to_vec()))
            .collect();

        let gshares = transpose(gshares);
        let fshares = transpose(fshares);

        let mut result = vec![G1P::default(); N];

        for i in 0..N {
            let mut msm = MSMKZG::<E>::new();
            for (base, scalar) in gshares[i].iter().zip(fshares[i].iter()) {
                msm.append_term(*scalar, *base)
            }
            result[i] = msm.eval();
        }

        let result: G1P = unpackexp::<E>(&result, true, &pp).iter().sum();
        assert_eq!(expected, result);
    }
}
