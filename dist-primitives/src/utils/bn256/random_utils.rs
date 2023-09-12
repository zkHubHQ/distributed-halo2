use ff::Field;
use group::Group;
use halo2curves::pairing::Engine;
use rand::Rng;

pub fn random_fr<E, R>(rng: &mut R) -> E::Scalar
where
    E: Engine,
    R: Rng,
{
    E::Scalar::random(rng)
}

pub fn create_random_group_element<E, R>(rng: &mut R) -> E::G1
where
    E: Engine,
    R: Rng,
{
    let random_scalar: E::Scalar = random_fr::<E, R>(rng);
    E::G1::identity() * random_scalar
}

pub fn create_random_group_elements<E, R>(rng: &mut R, size: usize) -> Vec<E::G1>
where
    E: Engine,
    R: Rng,
{
    (0..size)
        .map(|_| create_random_group_element::<E, R>(rng))
        .collect()
}
