use dist_primitives::Opt;
use log::debug;
use mpc_net::{MpcMultiNet as Net, MpcNet};
use plonk::{dplonk::d_plonk_test, PlonkDomain};

use halo2_proofs::halo2curves::bn256::{Bn256, Fr};
use secret_sharing::pss::PackedSharingParams;
use structopt::StructOpt;

fn main() {
    debug!("Start");

    env_logger::builder().format_timestamp(None).init();
    let opt = Opt::from_args();
    Net::init_from_file(opt.input.to_str().unwrap(), opt.id);

    let pd = PlonkDomain::<Fr>::new(opt.m);
    let pp = PackedSharingParams::<Fr>::new(opt.l);
    d_plonk_test::<Bn256>(&pd, &pp);

    if Net::am_king() {
        println!("Stats: {:#?}", Net::stats());
    }

    Net::deinit();
    debug!("Done");
}
