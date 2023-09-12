use halo2_proofs::transcript::{TranscriptReadBuffer, TranscriptWriterBuffer};
use halo2_proofs::{
    arithmetic::Field as FieldExt,
    circuit::{AssignedCell, Layouter, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance, Selector},
    poly::Rotation,
};
use itertools::Itertools;
use ff::PrimeField;
//use snark_verifier_sdk::util::arithmetic::PrimeField;
//use snark_verifier_sdk::verifier::plonk::PlonkVerifier;
//use snark_verifier_sdk::verifier::SnarkVerifier;
use std::iter;
use std::marker::PhantomData;
use halo2_proofs::circuit::SimpleFloorPlanner;

use rand::RngCore;
//use snark_verifier::pcs::kzg::KzgDecidingKey;
//use snark_verifier::{
//    loader::evm::{self, EvmLoader},
//    system::halo2::{compile, transcript::evm::EvmTranscript, Config},
//};
use std::{fs::File, io::prelude::*, rc::Rc};

use hex::encode;

use serde_json::json;

#[derive(Clone)]
pub(crate) struct Number<F: FieldExt>(AssignedCell<F, F>);

#[derive(Debug, Clone)]
pub(crate) struct FiboConfig {
    a: Column<Advice>,
    b: Column<Advice>,
    c: Column<Advice>,
    i: Column<Instance>,
    s: Selector,
}

// The chip that configures the gate and fills in the witness
#[derive(Debug, Clone)]
pub(crate) struct FiboChip<F: FieldExt> {
    config: FiboConfig,
    _marker: PhantomData<F>,
}

impl<F: FieldExt> FiboChip<F> {
    fn construct(config: FiboConfig) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> FiboConfig {
        // create columns
        let a = meta.advice_column();
        let b = meta.advice_column();
        let c = meta.advice_column();
        let i = meta.instance_column();
        let s = meta.selector();

        // enable permutation checks for the following columns
        meta.enable_equality(a);
        meta.enable_equality(b);
        meta.enable_equality(c);
        meta.enable_equality(i);

        // define the custom gate
        meta.create_gate("add", |meta| {
            let s = meta.query_selector(s);
            let lhs = meta.query_advice(a, Rotation::cur());
            let rhs = meta.query_advice(b, Rotation::cur());
            let out = meta.query_advice(c, Rotation::cur());
            vec![s * (lhs + rhs - out)]
        });

        FiboConfig { a, b, c, i, s }
    }

    fn load_first_row(
        &self,
        mut layouter: impl Layouter<F>,
        a: F,
        b: F,
    ) -> Result<(Number<F>, Number<F>, Number<F>), Error> {
        // load first row
        layouter.assign_region(
            || "first row",
            |mut region| {
                // enable the selector
                self.config.s.enable(&mut region, 0)?;

                let a_num = region
                    .assign_advice(
                        || "a",
                        self.config.a, // column a
                        0,             // rotation
                        || Value::known(a),
                    )
                    .map(Number)?;

                let b_num = region
                    .assign_advice(
                        || "b",
                        self.config.b, // column b
                        0,             // rotation
                        || Value::known(b),
                    )
                    .map(Number)?;

                let c_num = region
                    .assign_advice(
                        || "c",
                        self.config.c, // column c
                        0,             // rotation
                        || Value::known(a + b),
                    )
                    .map(Number)?;

                Ok((a_num, b_num, c_num))
            },
        )
    }

    fn load_row(
        &self,
        mut layouter: impl Layouter<F>,
        a: &Number<F>,
        b: &Number<F>,
    ) -> Result<Number<F>, Error> {
        layouter.assign_region(
            || "row",
            |mut region| {
                // enable the selector
                self.config.s.enable(&mut region, 0)?;

                // copy the cell from previous row
                a.0.copy_advice(|| "a", &mut region, self.config.a, 0)?;
                b.0.copy_advice(|| "b", &mut region, self.config.b, 0)?;
                let c = a.0.value().and_then(|a| b.0.value().map(|b| *a + *b));

                region
                    .assign_advice(|| "c", self.config.c, 0, || c)
                    .map(Number)
            },
        )
    }

    fn expose_public(
        &self,
        mut layouter: impl Layouter<F>,
        num: Number<F>,
        row: usize,
    ) -> Result<(), Error> {
        layouter.constrain_instance(num.0.cell(), self.config.i, row)
    }
}


#[derive(Default, Clone)]
struct FiboCircuit<F> {
    a: F,
    b: F,
    num: usize,
}


impl<F: FieldExt> Circuit<F> for FiboCircuit<F> {
    type Config = FiboConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        FiboChip::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let chip = FiboChip::construct(config);
        let (_, mut b, mut c) =
            chip.load_first_row(layouter.namespace(|| "first row"), self.a, self.b)?;
        for _ in 3..self.num {
            let new_c = chip.load_row(layouter.namespace(|| "row"), &b, &c)?;
            b = c;
            c = new_c;
        }
        chip.expose_public(layouter.namespace(|| "expose c"), c, 0)?;
        Ok(())
    }
}


fn get_fibo_seq(a: u64, b: u64, num: usize) -> Vec<u64> {
    let mut seq = vec![0; num];
    seq[0] = a;
    seq[1] = b;
    for i in 2..num {
        seq[i] = seq[i - 1] + seq[i - 2];
    }
    seq
}

pub fn encode_calldata_json<F>(instances: &[Vec<F>], proof: &[u8]) -> Vec<u8>
where
    F: PrimeField<Repr = [u8; 32]>,
{
    let instances_json: Vec<u8> = iter::empty()
        .chain(
            instances
                .iter()
                .flatten()
                .flat_map(|value| value.to_repr().as_ref().iter().rev().cloned().collect_vec()),
        )
        .collect();

    // format!("0x{}", hex::encode(x.as_ref()))
    println!(
        "instances_json {:?}",
        format!("0x{}", hex::encode(instances_json.clone()))
    );

    println!(
        "instance_proof {:?}",
        format!("0x{}", hex::encode(proof.clone()))
    );

    let proof_data = json!({
        "pub_ins": format!("0x{}",hex::encode(instances_json.clone())),
        "proof": format!("0x{}", hex::encode(proof.clone())),
    });
    let mut file = File::create("output_proof.json").unwrap();
    let json_str = serde_json::to_string(&proof_data).unwrap();
    file.write_all(json_str.as_bytes()).unwrap();
    let result = iter::empty()
        .chain(
            instances
                .iter()
                .flatten()
                .flat_map(|value| value.to_repr().as_ref().iter().rev().cloned().collect_vec()),
        )
        .chain(proof.iter().cloned())
        .collect();

    result
}

//#[cfg(test)]
//mod tests {
//    use std::{
//        io::Write,
//        iter,
//        process::{Command, Stdio},
//    };
//
//    use halo2_proofs::dev::MockProver;
//    use halo2_proofs::plonk::{
//        create_proof, keygen_pk, keygen_vk, verify_proof, ProvingKey, VerifyingKey,
//    };
//
//
//    fn get_fibo_seq(a: u64, b: u64, num: usize) -> Vec<u64> {
//        let mut seq = vec![0; num];
//        seq[0] = a;
//        seq[1] = b;
//        for i in 2..num {
//            seq[i] = seq[i - 1] + seq[i - 2];
//        }
//        seq
//    }
//
//    #[test]
//    fn gen_params() {}
//
//    #[test]
//    fn test_verifier() {
//
//        use snark_verifier_sdk::{
//            halo2::{aggregation::AccumulationSchemeSDK, gen_srs},
//            Snark, GWC, SHPLONK,
//        };
//        use halo2_proofs::halo2curves::bn256::{Bn256, Fr};
//        use halo2_base::utils::fs::gen_srs;
//
//        let params = gen_srs(10);
//
//        // Prepare the private and public inputs to the circuit!
//        let num = 14;
//        let seq = get_fibo_seq(1, 1, num);
//        let res = Fr::from(seq[num - 1]);
//
//        // Instantiate the circuit with the private inputs.
//        let fibo_circuit = FiboCircuit {
//            a: Fr::from(seq[0]),
//            b: Fr::from(seq[1]),
//            num,
//        };
//
//        // Arrange the public input. We expose the multiplication result in row 0
//        // of the instance column, so we position it there in our public inputs.
//        let mut public_inputs = vec![res];
//        //let deployment_code = gen_evm_verifier(&params, pk.get_vk(), vec![1]);
//
//
//        let mut instances = vec![vec![res]];
//
//        let proof = gen_proof(&params, &pk, fibo_circuit.clone(), vec![vec![res]]);
//
//        // println!("final proof {:?}", &proof);
//
//
//        // Set circuit size
//        let k = 10;
//
//        // Given the correct public input, our circuit will verify.
//        let prover = MockProver::run(k, &fibo_circuit, vec![public_inputs.clone()]).unwrap();
//        assert_eq!(prover.verify(), Ok(()));
//
//        // If we try some other public input, the proof will fail!
//        public_inputs[0] += Fr::one();
//        let prover = MockProver::run(k, &fibo_circuit, vec![public_inputs]).unwrap();
//        assert!(prover.verify().is_err());
//    }
//}
//