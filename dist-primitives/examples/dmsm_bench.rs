use ark_std::{end_timer, start_timer, UniformRand, Zero};
use dist_primitives::utils::domain_utils::EvaluationDomainExt;
use dist_primitives::{dmsm::dmsm::d_msm, Opt};
use ff::Field;
use ff::{PrimeField, WithSmallOrderMulGroup};
use group::Group;
use halo2_proofs::{
    halo2curves::bn256::{Bn256, Fr},
    poly::EvaluationDomain,
};
use halo2curves::pairing::Engine;
use mpc_net::{MpcMultiNet as Net, MpcNet};
use secret_sharing::pss::PackedSharingParams;
use std::fmt::Debug;
use structopt::StructOpt;

pub fn d_msm_test<E: Engine>(pp: &PackedSharingParams<E::Scalar>, dom: &EvaluationDomain<E::Scalar>)
where
    E: Engine + Debug,
    E::Scalar: PrimeField + WithSmallOrderMulGroup<3>,
{
    // let m = pp.l*4;
    // let case_timer = start_timer!(||"affinemsm_test");
    let mbyl: usize = dom.size() / pp.l;
    println!("m: {}, mbyl: {}", dom.size(), mbyl);

    let mut rng = &mut ark_std::test_rng();

    let mut y_share: Vec<E::Scalar> = vec![E::Scalar::ZERO; dom.size()];
    let mut x_share: Vec<E::G1> = vec![E::G1::identity(); dom.size()];

    for i in 0..dom.size() {
        y_share[i] = E::Scalar::random(&mut rng);
        x_share[i] = E::G1::identity() * y_share[i];
    }

    let dmsm = start_timer!(|| "Distributed msm");
    d_msm::<E>(&x_share, &y_share, pp);
    end_timer!(dmsm);
}

fn main() {
    env_logger::builder().format_timestamp(None).init();

    let opt = Opt::from_args();

    Net::init_from_file(opt.input.to_str().unwrap(), opt.id);

    let pp = PackedSharingParams::<Fr>::new(opt.l);
    for i in 10..20 {
        let dom = EvaluationDomain::<Fr>::new(1, i);
        println!("domain size: {}", dom.size());
        d_msm_test::<Bn256>(&pp, &dom);
    }

    Net::deinit();
}
