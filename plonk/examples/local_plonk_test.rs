use halo2_proofs::halo2curves::bn256::{Bn256, Fr};
use plonk::{localplonk::localplonk, PlonkDomain};

use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    /// size
    pub m: usize,
}

fn main() {
    env_logger::builder().format_timestamp(None).init();
    let opt = Opt::from_args();
    let cd = PlonkDomain::<Fr>::new(opt.m);
    localplonk::<Bn256>(&cd);
}
