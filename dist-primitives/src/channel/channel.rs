use bincode;
use mpc_net::MpcNet;
use serde::{Deserialize, Serialize};

pub trait MpcSerNet: MpcNet {
    #[inline]
    fn broadcast<T: for<'de> Deserialize<'de> + Serialize>(out: &T) -> Vec<T> {
        let bytes_out = bincode::serialize(out).unwrap();
        let bytes_in = Self::broadcast_bytes(&bytes_out);
        bytes_in
            .into_iter()
            .map(|b| bincode::deserialize(&b[..]).unwrap())
            .collect()
    }

    #[inline]
    fn send_to_king<T: for<'de> Deserialize<'de> + Serialize>(out: &T) -> Option<Vec<T>> {
        let bytes_out = bincode::serialize(out).unwrap();
        Self::send_bytes_to_king(&bytes_out).map(|bytes_in| {
            bytes_in
                .into_iter()
                .map(|b| bincode::deserialize(&b[..]).unwrap())
                .collect()
        })
    }

    #[inline]
    fn recv_from_king<T: for<'de> Deserialize<'de> + Serialize>(out: Option<Vec<T>>) -> T {
        let bytes_in = Self::recv_bytes_from_king(out.map(|outs| {
            outs.iter()
                .map(|out| {
                    bincode::serialize(out).unwrap()
                })
                .collect()
        }));
        bincode::deserialize(&bytes_in[..]).unwrap()
    }
}

impl<N: MpcNet> MpcSerNet for N {}
