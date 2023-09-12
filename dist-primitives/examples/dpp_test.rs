use dist_primitives::utils::domain_utils::EvaluationDomainExt;
use dist_primitives::{
    channel::channel::MpcSerNet,
    dpp::dpp::d_pp,
    utils::pack::{pack_vec, transpose},
    Opt,
};
use ff::{PrimeField, WithSmallOrderMulGroup};
use halo2_proofs::{halo2curves::bn256::Fr, poly::EvaluationDomain};
use mpc_net::{MpcMultiNet as Net, MpcNet};
use secret_sharing::pss::PackedSharingParams;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

pub fn d_pp_test<F>(pp: &PackedSharingParams<F>, dom: &EvaluationDomain<F>)
where
    F: PrimeField + WithSmallOrderMulGroup<3> + Serialize + for<'de> Deserialize<'de>,
{
    // We apply FFT on this vector
    // let mut x = vec![F::ONE; cd.m];
    let mut x: Vec<F> = Vec::new();
    for i in 0..dom.size() {
        x.push(F::from((i + 1) as u64));
    }

    // Output to test against
    let should_be_output = vec![F::ONE; dom.size()];

    // pack x
    let px = transpose(pack_vec(&x, pp));

    let px_share = px[Net::party_id()].clone();
    let pp_px_share = d_pp(px_share.clone(), px_share.clone(), pp);

    // Send to king who reconstructs and checks the answer
    Net::send_to_king(&pp_px_share).map(|pp_px_shares| {
        let pp_px_shares = transpose(pp_px_shares);

        let pp_px: Vec<F> = pp_px_shares
            .into_iter()
            .flat_map(|x| pp.unpack(&x))
            .collect();

        if Net::am_king() {
            debug_assert_eq!(should_be_output, pp_px);
        }
    });
}

pub fn main() {
    env_logger::builder().format_timestamp(None).init();

    let opt = Opt::from_args();

    Net::init_from_file(opt.input.to_str().unwrap(), opt.id);
    let pp = PackedSharingParams::<Fr>::new(opt.l);
    let cd = EvaluationDomain::<Fr>::new(1, opt.m as usize);
    d_pp_test::<ark_bls12_377::Fr>(&pp, &cd);

    Net::deinit();
}
