use std::mem;

use ark_std::{end_timer, log2, start_timer};
use dist_primitives::dfft::dfft::fft_in_place_rearrange;
use dist_primitives::utils::domain_utils::EvaluationDomainExt;
use ff::{PrimeField, WithSmallOrderMulGroup};
use halo2_proofs::{
    halo2curves::bn256::Fr,
    poly::{EvaluationDomain, Rotation},
};
use secret_sharing::pss::PackedSharingParams;
use serde::{Deserialize, Serialize};

pub fn local_dfft_test<F>(pp: &PackedSharingParams<F>, dom: &EvaluationDomain<F>)
where
    F: PrimeField + WithSmallOrderMulGroup<3> + Serialize + for<'de> Deserialize<'de>,
{
    let mbyl: usize = dom.size() / pp.l;

    // We apply FFT on this vector
    // let mut x = vec![F::one();cd.m];
    let mut x: Vec<F> = Vec::new();
    for i in 0..dom.size() {
        x.push(F::from(i as u64));
    }

    // Output to test against
    let mut expected_x = x.clone();
    let output = PackedSharingParams::fft(&mut expected_x, dom);

    // Rearranging x
    let myfft_timer = start_timer!(|| "MY FFT");

    fft_in_place_rearrange::<F>(&mut x);

    let mut px: Vec<Vec<F>> = Vec::new();
    for i in 0..mbyl {
        px.push(x.iter().skip(i).step_by(mbyl).cloned().collect::<Vec<_>>());
    }

    let mut s1 = px.clone();

    let now = start_timer!(|| "FFT1");

    // fft1
    for i in (log2(pp.l) + 1..=log2(dom.size())).rev() {
        let poly_size = dom.size() / 2usize.pow(i);
        let factor_stride = element(2usize.pow(i - 1), &dom);
        let mut factor = factor_stride;
        for k in 0..poly_size {
            for j in 0..2usize.pow(i - 1) / pp.l {
                for ii in 0..pp.l {
                    let x = s1[(2 * j) * (poly_size) + k][ii];
                    let y = s1[(2 * j + 1) * (poly_size) + k][ii] * factor;
                    s1[j * (2 * poly_size) + k][ii] = x + y;
                    s1[j * (2 * poly_size) + k + poly_size][ii] = x - y;
                }
            }
            factor = factor * factor_stride;
        }
    }

    end_timer!(now);

    // psstoss
    let mut sx: Vec<F> = Vec::new();
    for i in 0..mbyl {
        sx.append(&mut s1[i]);
    }

    // fft2
    let mut s1 = sx.clone();
    let mut s2 = sx.clone();

    let now = start_timer!(|| "FFT2");

    for i in (1..=log2(pp.l)).rev() {
        let poly_size = dom.size() / 2usize.pow(i);
        let factor_stride = element(2usize.pow(i - 1), &dom);
        let mut factor = factor_stride;
        for k in 0..poly_size {
            for j in 0..2usize.pow(i - 1) {
                let x = s1[k * (2usize.pow(i)) + 2 * j];
                let y = s1[k * (2usize.pow(i)) + 2 * j + 1] * factor;
                s2[k * (2usize.pow(i - 1)) + j] = x + y;
                s2[(k + poly_size) * (2usize.pow(i - 1)) + j] = x - y;
            }
            factor = factor * factor_stride;
        }
        mem::swap(&mut s1, &mut s2);
    }
    end_timer!(now);

    end_timer!(myfft_timer);

    s1.rotate_right(1);

    assert_eq!(output, s1);
}

// get the ith element of the domain
pub fn element<F>(i: usize, dom: &EvaluationDomain<F>) -> F
where
    F: PrimeField + WithSmallOrderMulGroup<3> + Serialize + for<'de> Deserialize<'de>,
{
    dom.rotate_omega(F::ONE, Rotation(i as i32))
}

pub fn main() {
    let pp = PackedSharingParams::<Fr>::new(2);
    let dom = EvaluationDomain::<Fr>::new(1, 8);
    local_dfft_test::<Fr>(&pp, &dom);
}
