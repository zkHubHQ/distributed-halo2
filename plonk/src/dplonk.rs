use std::fmt::Debug;

use crate::dpoly_commit::PackPolyCk;
use crate::{poly_commit::PolyCk, PlonkDomain};
use ark_std::{end_timer, start_timer, One, Zero};
use dist_primitives::dfft::dfft::{d_fft, d_ifft};
use dist_primitives::dpp::dpp::d_pp;
use dist_primitives::utils::deg_red::deg_red;
use dist_primitives::utils::domain_utils::EvaluationDomainExt;
use ff::Field;
use ff::{PrimeField, WithSmallOrderMulGroup};
use halo2_proofs::halo2curves::pairing::Engine;
use halo2_proofs::poly::{EvaluationDomain, Rotation};
use rand::Rng;
use secret_sharing::pss::PackedSharingParams;
use serde::{Deserialize, Serialize};

use mpc_net::{MpcMultiNet as Net, MpcNet};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PackProvingKey<E>
where
    E: Engine + Debug,
    E::Scalar: PrimeField + WithSmallOrderMulGroup<3> + Serialize + for<'de> Deserialize<'de>,
{
    pub ql: Vec<E::Scalar>,
    pub qr: Vec<E::Scalar>,
    pub qm: Vec<E::Scalar>,
    pub qo: Vec<E::Scalar>,
    pub qc: Vec<E::Scalar>,
    pub s1: Vec<E::Scalar>,
    pub s2: Vec<E::Scalar>,
    pub s3: Vec<E::Scalar>,
}

impl<E> PackProvingKey<E>
where
    E: Engine + Debug,
    E::Scalar: PrimeField + WithSmallOrderMulGroup<3> + Serialize + for<'de> Deserialize<'de>,
{
    pub fn new<R: Rng>(n_gates: usize, rng: &mut R, pp: &PackedSharingParams<E::Scalar>) -> Self {
        let outer_time = start_timer!(|| "Dummy CRS");

        let mut qm: Vec<E::Scalar> = vec![E::Scalar::random(&mut *rng); 8 * n_gates / pp.l];
        let mut ql: Vec<E::Scalar> = qm.clone();
        let mut qr: Vec<E::Scalar> = qm.clone();
        let mut qo: Vec<E::Scalar> = qm.clone();
        let mut qc: Vec<E::Scalar> = qm.clone();
        let mut s1: Vec<E::Scalar> = qm.clone();
        let mut s2: Vec<E::Scalar> = qm.clone();
        let mut s3: Vec<E::Scalar> = qm.clone();

        for i in 0..qm.len() {
            qm[i] = E::Scalar::random(&mut *rng);
            ql[i] = E::Scalar::random(&mut *rng);
            qr[i] = E::Scalar::random(&mut *rng);
            qo[i] = E::Scalar::random(&mut *rng);
            qc[i] = E::Scalar::random(&mut *rng);
            s1[i] = E::Scalar::random(&mut *rng);
            s2[i] = E::Scalar::random(&mut *rng);
            s3[i] = E::Scalar::random(&mut *rng);
        }

        end_timer!(outer_time);

        PackProvingKey {
            qm,
            ql,
            qr,
            qo,
            qc,
            s1,
            s2,
            s3,
        }
    }
}

// get the ith element of the domain
pub fn element<F>(i: usize, dom: &EvaluationDomain<F>) -> F
where
    F: PrimeField + WithSmallOrderMulGroup<3> + Serialize + for<'de> Deserialize<'de>,
{
    dom.rotate_omega(F::ONE, Rotation(i as i32))
}

pub fn d_plonk_test<E>(pd: &PlonkDomain<E::Scalar>, pp: &PackedSharingParams<E::Scalar>)
where
    E: Engine + Debug,
    E::Scalar: PrimeField + WithSmallOrderMulGroup<3> + Serialize + for<'de> Deserialize<'de>,
{
    let mbyl = pd.n_gates / pp.l;
    if Net::am_king() {
        println!("mbyl: {}", mbyl);
    }
    // Generate CRS ===========================================
    if Net::am_king() {
        println!("Generating CRS===============================");
    }
    let rng = &mut ark_std::test_rng();
    let pk = PackProvingKey::<E>::new(pd.n_gates, rng, pp);

    let ck: PackPolyCk<E> = PackPolyCk::<E>::new(pd.n_gates, rng, pp);
    let ck8: PackPolyCk<E> = PackPolyCk::<E>::new(8 * pd.n_gates, rng, pp);

    let prover_timer = start_timer!(|| "Prover");
    if Net::am_king() {
        println!("Round 1===============================");
    }
    // Round 1 ================================================
    // Commit to a, b, c

    let mut aevals = vec![E::Scalar::random(&mut *rng); mbyl];
    let mut bevals = aevals.clone();
    let mut cevals = aevals.clone();
    for i in 0..aevals.len() {
        aevals[i] = E::Scalar::random(&mut *rng);
        bevals[i] = E::Scalar::random(&mut *rng);
        cevals[i] = E::Scalar::random(&mut *rng);
    }

    println!("Committing to a, b, c");
    ck.commit(&aevals, pp);
    ck.commit(&bevals, pp);
    ck.commit(&cevals, pp);
    println!("=======================");

    println!("Extending domain of a,b,c to 8n");
    // do ifft and fft to get evals of a,b,c on the 8n domain
    let aevals8 = d_ifft(aevals.clone(), true, 8, false, &pd.gates, pp);
    let bevals8 = d_ifft(bevals.clone(), true, 8, false, &pd.gates, pp);
    let cevals8 = d_ifft(cevals.clone(), true, 8, false, &pd.gates, pp);

    let aevals8 = d_fft(aevals8, false, 1, false, &pd.gates8, pp);
    let bevals8 = d_fft(bevals8, false, 1, false, &pd.gates8, pp);
    let cevals8 = d_fft(cevals8, false, 1, false, &pd.gates8, pp);
    println!("=======================");

    if Net::am_king() {
        println!("Round 2===============================");
    }
    // Round 2 ================================================
    // Compute z
    let beta = E::Scalar::random(&mut *rng);
    let gamma = E::Scalar::random(&mut *rng);

    let omega = element(1, &pd.gates8);
    let omage = element(1, &pd.gates8);
    let mut omegai = E::Scalar::ONE;

    let mut num = vec![E::Scalar::ONE; mbyl];
    let mut den = vec![E::Scalar::ONE; mbyl];

    let ldpp_timer = start_timer!(|| "Local DPP");
    for i in 0..mbyl {
        // (w_j+σ∗(j)β+γ)(w_{n+j}+σ∗(n+j)β+γ)(w_{2n+j}+σ∗(2n+j)β+γ)
        den[i] = (aevals[i] + beta * pk.s1[i] + gamma)
            * (bevals[i] + beta * pk.s2[i] + gamma)
            * (cevals[i] + beta * pk.s3[i] + gamma);

        // (w_j+βωj+γ)(w_{n+j}+βk1ωj+γ)(w_{2n+j}+βk2ωj+γ)
        num[i] = (aevals[i] + beta * omegai + gamma)
            * (bevals[i] + beta * omegai + gamma)
            * (cevals[i] + beta * omegai + gamma);

        omegai *= omega;
    }
    end_timer!(ldpp_timer);
    // todo: benchmark this
    // partial products
    let zevals = d_pp(num, den, pp);

    // extend to zevals8
    let zevals8 = zevals.clone();
    let zevals8 = d_ifft(zevals8, true, 8, false, &pd.gates, pp);
    let zevals8 = d_fft(zevals8, false, 1, false, &pd.gates8, pp);

    if Net::am_king() {
        println!("Round 3===============================");
    }
    // Round 3 ================================================
    // Compute t
    let alpha = E::Scalar::random(&mut *rng);

    let mut tevals8 = vec![E::Scalar::random(&mut *rng); 8 * mbyl];

    let omega = element(1, &pd.gates8);
    let omegan = element(1, &pd.gates8).pow(&([pd.n_gates as u64]));
    let womegan = (E::Scalar::ZETA * element(1, &pd.gates8)).pow(&([pd.n_gates as u64]));

    let mut omegai = E::Scalar::ONE;
    let mut omegani = E::Scalar::ONE;
    let mut womengani = E::Scalar::ONE;

    let t_timer = start_timer!(|| "Compute t");
    for i in 0..8 * mbyl {
        // ((a(X)b(X)qM(X) + a(X)qL(X) + b(X)qR(X) + c(X)qO(X) + PI(X) + qC(X))
        tevals8[i] += aevals8[i] * bevals8[i] * pk.qm[i]
            + aevals8[i] * pk.ql[i]
            + bevals8[i] * pk.qr[i]
            + cevals8[i] * pk.qo[i]
            + pk.qc[i];

        // ((a(X) + βX + γ)(b(X) + βk1X + γ)(c(X) + βk2X + γ)z(X))*alpha
        tevals8[i] += (aevals8[i] + beta * omegai + gamma)
            * (bevals8[i] + beta * omegai + gamma)
            * (cevals8[i] + beta * omegai + gamma)
            * (omegani - E::Scalar::ONE)
            * alpha;

        // - ((a(X) + βSσ1(X) + γ)(b(X) + βSσ2(X) + γ)(c(X) + βSσ3(X) + γ)z(Xω))*alpha
        tevals8[i] -= (aevals8[i] + beta * pk.s1[i] + gamma)
            * (bevals8[i] + beta * pk.s2[i] + gamma)
            * (cevals8[i] + beta * pk.s3[i] + gamma)
            * (womengani - E::Scalar::ONE)
            * alpha;

        // + (z(X)−1)L1(X)*alpha^2)/Z
        // z(X) is computed using partial products
        tevals8[i] += (zevals8[i]-E::Scalar::ONE)
                        *E::Scalar::ONE //todo:replace with L1
                        *alpha*alpha;

        omegai *= omega;
        omegani *= omegan;
        womengani *= womegan;
    }
    end_timer!(t_timer);

    // divide by ZH
    let tcoeffs = d_ifft(tevals8, true, 1, false, &pd.gates8, pp);
    let mut tevals8 = d_fft(tcoeffs, false, 1, false, &pd.gates8, pp); //king actually needs to truncate

    let toep_mat = E::Scalar::from(123 as u64); // packed shares of toeplitz matrix drop from sky
    tevals8.iter_mut().for_each(|x| *x *= toep_mat);

    let tevals8 = deg_red(tevals8, pp);

    if Net::am_king() {
        println!("Round 4===============================");
    }
    // Round 4 ================================================
    // commit to z and t
    // open a, b, c, s1, s2, s3, z, t
    // commit and open r = (open_a.open_b)qm + (open_a)ql + (open_b)qr + (open_c)qo + qc

    println!("Committing to z, t");
    ck.commit(&zevals, pp);
    ck8.commit(&tevals8, pp);

    println!("Opening a, b, c");
    let point = E::Scalar::random(&mut *rng);
    let open_a = ck.open(&aevals, point, &pd.gates, pp);
    let open_b = ck.open(&bevals, point, &pd.gates, pp);
    let open_c = ck.open(&cevals, point, &pd.gates, pp);

    println!("Opening s1, s2, s3");
    // extract every 8th element of pk.s1 using iterators
    ck.open(
        &pk.s1.iter().step_by(8).copied().collect(),
        point,
        &pd.gates,
        pp,
    );
    ck.open(
        &pk.s2.iter().step_by(8).copied().collect(),
        point,
        &pd.gates,
        pp,
    );
    ck.open(
        &pk.s3.iter().step_by(8).copied().collect(),
        point,
        &pd.gates,
        pp,
    );

    println!("Computing r");
    let r_timer = start_timer!(|| "Compute r");
    let open_ab = open_a * open_b;
    let mut revals = vec![E::Scalar::ZERO; mbyl];
    for i in 0..mbyl {
        revals[i] = open_ab * pk.qm[i]
            + open_a * pk.ql[i]
            + open_b * pk.qr[i]
            + open_c * pk.qo[i]
            + pk.qc[i];
    }
    end_timer!(r_timer);

    println!("Committing to r");
    ck.commit(&revals, pp);
    ck.open(&revals, point, &pd.gates, pp);

    end_timer!(prover_timer);
}
