use ff::{PrimeField, WithSmallOrderMulGroup};
use halo2_proofs::poly::EvaluationDomain;

/// Currently the packed secret sharing is deterministic, but it can easily be extended to add random values when packing
#[derive(Debug, Clone)]
pub struct PackedSharingParams<F>
where
    F: PrimeField,
{
    pub t: usize,                     // Corruption threshold
    pub l: usize,                     // Packing factor
    pub n: usize,                     // Number of parties
    pub share: EvaluationDomain<F>,   // Share domain
    pub secret: EvaluationDomain<F>,  // Secrets domain
    pub secret2: EvaluationDomain<F>, // Secrets2 domain
}

impl<F: PrimeField + WithSmallOrderMulGroup<3>> PackedSharingParams<F> {
    #[allow(unused)]
    pub fn new(l: usize) -> Self {
        let n = l * 4;
        let t = l - 1;
        debug_assert_eq!(n, 2 * (t + l + 1));

        let share = EvaluationDomain::<F>::new(1, (n as f64).log2() as u32);
        let secret = EvaluationDomain::<F>::new(1, ((l + t + 1) as f64).log2() as u32);
        let secret2 =
            EvaluationDomain::<F>::new(1, ((2 as f64) * (l + t + 1) as f64).log2() as u32);

        debug_assert_eq!(1 << share.k() as usize, n);
        debug_assert_eq!(1 << secret.k() as usize, l + t + 1);
        debug_assert_eq!(1 << secret2.k() as usize, 2 * (l + t + 1));

        PackedSharingParams {
            t,
            l,
            n,
            share,
            secret,
            secret2,
        }
    }

    // Custom FFT function using halo2's EvaluationDomain
    pub fn fft(poly: &mut Vec<F>, domain: &EvaluationDomain<F>) -> Vec<F> {
        poly.resize(1 << domain.k(), F::ZERO);
        let coeff_poly = domain.coeff_from_vec(poly.clone());
        let lagrange_poly = domain.coeff_to_extended(coeff_poly);
        lagrange_poly.iter().cloned().collect()
    }

    // Custom IFFT function using halo2's EvaluationDomain
    pub fn ifft(poly: &mut Vec<F>, domain: &EvaluationDomain<F>) -> Vec<F> {
        poly.resize(1 << domain.k(), F::ZERO);
        let lagrange_poly = domain.lagrange_from_vec(poly.clone());
        let coeff_poly = domain.lagrange_to_coeff(lagrange_poly);
        coeff_poly.iter().cloned().collect()
    }

    #[allow(unused)]
    pub fn pack_from_public(&self, secrets: &Vec<F>) -> Vec<F> {
        let mut result = secrets.clone();
        self.pack_from_public_in_place(&mut result);
        result
    }

    #[allow(unused)]
    pub fn pack_from_public_in_place(&self, secrets: &mut Vec<F>) {
        // interpolating on secrets domain
        Self::ifft(secrets, &self.secret);
        // evaluate on share domain
        Self::fft(secrets, &self.share);
    }

    #[allow(unused)]
    pub fn unpack(&self, shares: &Vec<F>) -> Vec<F> {
        let mut result = shares.clone();
        self.unpack_in_place(&mut result);
        result
    }

    #[allow(unused)]
    pub fn unpack2(&self, shares: &Vec<F>) -> Vec<F> {
        let mut result = shares.clone();
        self.unpack2_in_place(&mut result);
        result
    }

    #[allow(unused)]
    pub fn unpack_in_place(&self, shares: &mut Vec<F>) {
        // interpolating on share domain
        Self::ifft(shares, &self.share);
        // evaluate on secrets domain
        Self::fft(shares, &self.secret);
        // truncate to remove the randomness
        shares.truncate(self.l);
    }

    #[allow(unused)]
    pub fn unpack2_in_place(&self, shares: &mut Vec<F>) {
        // interpolating on share domain
        Self::ifft(shares, &self.share);
        // evaluate on secrets2 domain
        Self::fft(shares, &self.secret2);
        // drop alternate elements from shares array and only iterate till 2l as the rest of it is randomness
        *shares = shares.iter().filter(|&&x| x != F::ZERO).copied().collect();
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use halo2_proofs::halo2curves::bn256::Fr as F;
    use PackedSharingParams;

    const L: usize = 4;
    const N: usize = L * 4;
    const T: usize = N / 2 - L - 1;

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

    #[test]
    fn test_initialize() {
        let pp = PackedSharingParams::<F>::new(L);
        assert_eq!(pp.t, L - 1);
        assert_eq!(pp.l, L);
        assert_eq!(pp.n, N);
        assert_eq!(1 << pp.share.k(), N);
        assert_eq!(1 << pp.secret.k(), L + T + 1);
        assert_eq!(1 << pp.secret2.k(), 2 * (L + T + 1));
    }

    #[test]
    fn test_pack_from_public() {
        let pp = PackedSharingParams::<F>::new(L);

        let rng = &mut ark_std::test_rng();
        let mut secrets = [F::default(); L];
        for i in 0..L {
            secrets[i] = random_fr(rng);
        }
        let mut secrets = secrets.to_vec();

        let expected = secrets.clone();

        pp.pack_from_public_in_place(&mut secrets);
        pp.unpack_in_place(&mut secrets);

        assert_eq!(expected, secrets);
    }

    #[test]
    fn test_multiplication() {
        let pp = PackedSharingParams::<F>::new(L);

        let rng = &mut ark_std::test_rng();
        let mut secrets = [F::default(); L];
        for i in 0..L {
            secrets[i] = random_fr(rng);
        }
        let mut secrets = secrets.to_vec();
        let expected: Vec<F> = secrets.iter().map(|x| (*x) * (*x)).collect();

        pp.pack_from_public_in_place(&mut secrets);

        let mut shares: Vec<F> = secrets.iter().map(|x| (*x) * (*x)).collect();

        pp.unpack2_in_place(&mut shares);

        assert_eq!(expected, shares);
    }
}
