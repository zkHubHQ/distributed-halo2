use ark_std::{end_timer, start_timer};
use dist_primitives::utils::domain_utils::EvaluationDomainExt;
use dist_primitives::Opt;
use ff::{PrimeField, WithSmallOrderMulGroup};
use halo2_proofs::halo2curves::bn256::{Bn256, Fr};
use halo2_proofs::halo2curves::pairing::Engine;
use halo2_proofs::poly::EvaluationDomain;
use mpc_net::{MpcMultiNet as Net, MpcNet};
use plonk::dpoly_commit::PackPolyCk;
use plonk::poly_commit::PolyCk;
use secret_sharing::pss::PackedSharingParams;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use structopt::StructOpt;

pub fn d_poly_commit_test<E: Engine>(
    pp: &PackedSharingParams<E::Scalar>,
    dom: &EvaluationDomain<E::Scalar>,
) where
    E: Engine + Debug,
    E::Scalar: PrimeField + WithSmallOrderMulGroup<3> + Serialize + for<'de> Deserialize<'de>,
{
    let mbyl: usize = dom.size() / pp.l;
    println!("m: {}, mbyl: {}", dom.size(), mbyl);

    let rng = &mut ark_std::test_rng();

    let pck = PackPolyCk::<E>::new(dom.size(), rng, pp);
    let peval_share: Vec<E::Scalar> = (0..mbyl).map(|i| E::Scalar::from(i as u64)).collect();

    let dmsm = start_timer!(|| "Distributed poly_commit");
    pck.commit(&peval_share, pp);
    end_timer!(dmsm);

    let dmsm = start_timer!(|| "Distributed commit_open");
    pck.open(&peval_share, E::Scalar::from(123 as u64), dom, pp);
    end_timer!(dmsm);

    if Net::am_king() {
        let ck = PolyCk::<E>::new(dom.size(), rng);
        let pevals: Vec<E::Scalar> = (0..dom.size()).map(|i| E::Scalar::from(i as u64)).collect();
        let nmsm = start_timer!(|| "Ark poly_commit");
        ck.commit(&pevals);
        end_timer!(nmsm);
        let nmsm = start_timer!(|| "Ark commit_open");
        ck.open(&pevals, E::Scalar::from(123 as u64), dom);
        end_timer!(nmsm);
    }
}

fn main() {
    env_logger::builder().format_timestamp(None).init();

    let opt = Opt::from_args();

    Net::init_from_file(opt.input.to_str().unwrap(), opt.id);

    let pp = PackedSharingParams::<Fr>::new(opt.l);
    let dom = EvaluationDomain::<Fr>::new(1, (opt.m as f64).log2() as u32);
    d_poly_commit_test::<Bn256>(&pp, &dom);

    Net::deinit();
}
