use group::GroupEncoding;
use halo2_proofs::halo2curves::pairing::Engine;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Default, PartialEq)]
pub struct G1Wrapper<E: Engine>(pub E::G1);

pub struct G1Visitor<E: Engine>(pub std::marker::PhantomData<E>);

impl<'de, E: Engine> Visitor<'de> for G1Visitor<E> {
    type Value = E::G1;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a byte slice representing a G1 point")
    }

    fn visit_bytes<U>(self, value: &[u8]) -> Result<Self::Value, U>
    where
        U: serde::de::Error,
    {
        let mut repr = <E::G1 as GroupEncoding>::Repr::default();
        repr.as_mut().copy_from_slice(value);
        let ct_option = E::G1::from_bytes(&repr);

        if bool::from(ct_option.is_some()) {
            Ok(ct_option.unwrap())
        } else {
            Err(U::custom("Failed to deserialize G1 point"))
        }
    }
}

impl<E: Engine> Serialize for G1Wrapper<E> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = self.0.to_bytes();
        serializer.serialize_bytes(bytes.as_ref())
    }
}

impl<'de, E: Engine> Deserialize<'de> for G1Wrapper<E> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let g1 = deserializer.deserialize_bytes(G1Visitor::<E>(std::marker::PhantomData))?;
        Ok(G1Wrapper(g1))
    }
}

impl<E: Engine> Clone for G1Wrapper<E> {
    fn clone(&self) -> Self {
        G1Wrapper(self.0.clone())
    }
}
