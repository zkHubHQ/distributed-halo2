use ark_std::{end_timer, start_timer};
use dist_primitives::dmsm::dmsm::packexp_from_public;
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

    let mut y_pub: Vec<E::Scalar> = Vec::new();
    let mut x_pub: Vec<E::G1> = Vec::new();

    for i in 0..dom.size() {
        y_pub[i] = E::Scalar::random(&mut rng);
        x_pub[i] = E::G1::identity() * y_pub[i];
    }

    let x_share: Vec<E::G1> = x_pub
        .chunks(pp.l)
        .map(|s| packexp_from_public::<E>(&s.to_vec(), &pp)[Net::party_id()])
        .collect();

    let y_share: Vec<E::Scalar> = y_pub
        .chunks(pp.l)
        .map(|s| pp.pack_from_public(&s.to_vec())[Net::party_id()])
        .collect();

    // Will be comparing against this in the end
    let nmsm = start_timer!(|| "Halo2 msm");
    let mut msm = MSMKZG::<E>::new();

    for (base, scalar) in x_pub.iter().zip(y_pub.iter()) {
        msm.append_term(*scalar, *base)
    }

    let should_be_output = msm.eval();
    end_timer!(nmsm);

    let dmsm = start_timer!(|| "Distributed msm");
    let output = d_msm::<E>(&x_share, &y_share, pp);
    end_timer!(dmsm);

    if Net::am_king() {
        assert_eq!(should_be_output, output);
    }
}

fn main() {
    env_logger::builder().format_timestamp(None).init();

    let opt = Opt::from_args();

    Net::init_from_file(opt.input.to_str().unwrap(), opt.id);

    let pp = PackedSharingParams::<Fr>::new(opt.l);
    let dom = EvaluationDomain::<Fr>::new(1, (opt.m as f64).log2() as u32);
    d_msm_test::<Bn256>(&pp, &dom);

    Net::deinit();
}
