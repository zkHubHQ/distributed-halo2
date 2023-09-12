use ark_std::{end_timer, start_timer, UniformRand, Zero};
use dist_primitives::utils::domain_utils::EvaluationDomainExt;
use dist_primitives::{dmsm::dmsm::d_msm, Opt};
use ff::Field;
use ff::{PrimeField, WithSmallOrderMulGroup};
use group::Group;
use halo2_proofs::poly::commitment::MSM;
use halo2_proofs::poly::kzg::msm::MSMKZG;
use halo2_proofs::{
    halo2curves::bn256::{Bn256, Fr},
    poly::EvaluationDomain,
};
use halo2curves::pairing::Engine;
use std::fmt::Debug;

pub fn msm_test<E: Engine>(dom: &EvaluationDomain<E::Scalar>)
where
    E: Engine + Debug,
    E::Scalar: PrimeField + WithSmallOrderMulGroup<3>,
{
    let mut rng = &mut ark_std::test_rng();

    let mut y_pub: Vec<E::Scalar> = vec![E::Scalar::ZERO; dom.size()];
    let mut x_pub: Vec<E::G1> = vec![E::G1::identity(); dom.size()];

    for i in 0..dom.size() {
        y_pub[i] = E::Scalar::random(&mut rng);
        x_pub[i] = E::G1::identity() * y_pub[i];
    }

    let nmsm = start_timer!(|| "Halo2 msm");
    let mut msm = MSMKZG::<E>::new();

    for (base, scalar) in x_pub.iter().zip(y_pub.iter()) {
        msm.append_term(*scalar, *base)
    }

    let should_be_output = msm.eval();
    end_timer!(nmsm);
    end_timer!(nmsm);
}

fn main() {
    for i in 10..20 {
        let dom = EvaluationDomain::<Fr>::new(1, i);
        println!("domain size: {}", dom.size());
        msm_test::<Bn256>(&dom);
    }
}
