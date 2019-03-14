// Plasma account (Merkle tree leaf)

use std::fmt::{self, Debug};
use ff::{Field, PrimeField, PrimeFieldRepr};
use time::PreciseTime;
use pairing::bn256::*;
use bellman::{Circuit};
use bellman::groth16::{
    create_random_proof, 
    generate_random_parameters, 
    prepare_verifying_key, 
    verify_proof,
};
use rand::{SeedableRng, Rng, XorShiftRng, Rand, thread_rng};
use sapling_crypto::circuit::test::*;
use sapling_crypto::alt_babyjubjub::{JubjubEngine, AltJubjubBn256, edwards::Point, PrimeOrder};
use pairing::bn256::{Bn256, Fr};

use super::primitives::{GetBits, GetBitsFixed};
use crate::sparse_merkle_tree;
use crate::sparse_merkle_tree::pedersen_hasher::PedersenHasher;
use crate::circuit::non_inclusion::{NonInclusion, BlockWitness};

const HASH_LENGTH: usize = 256;

#[derive(Debug, Clone)]
pub struct Leaf<E: JubjubEngine>{
    pub hash:    Vec<bool>,
    pub phantom: std::marker::PhantomData<E>,
}

impl<E: JubjubEngine> GetBits for Leaf<E> {
    fn get_bits_le(&self) -> Vec<bool> {
        self.hash.clone()
    }
}

impl<E: JubjubEngine> Default for Leaf<E> {
    fn default() -> Self{
        let mut v = Vec::with_capacity(HASH_LENGTH);
        v.resize(HASH_LENGTH, false);
        Self {
            hash: v,
            phantom: std::marker::PhantomData
        }
    }
}

// code below is for testing

pub type BabyTransactionLeaf = Leaf<Bn256>;
pub type BabyTransactionTree = sparse_merkle_tree::SparseMerkleTree<BabyTransactionLeaf, Fr, PedersenHasher<Bn256>>;

impl BabyTransactionTree {
    pub fn verify_proof(&self, index: u32, item: BabyTransactionLeaf, proof: Vec<(Fr, bool)>) -> bool {
        use crate::sparse_merkle_tree::hasher::Hasher;
        assert!(index < self.capacity());
        let item_bits = item.get_bits_le();
        let mut hash = self.hasher.hash_bits(item_bits);
        let mut proof_index: u32 = 0;

        for (i, e) in proof.clone().into_iter().enumerate() {
            if e.1 {
                // current is right
                proof_index |= 1 << i;
                hash = self.hasher.compress(&e.0, &hash, i);
            } else {
                // current is left
                hash = self.hasher.compress(&hash, &e.0, i);
            }
        }

        if proof_index != index {
            return false;
        }

        hash == self.root_hash()
    }
}
    
#[cfg(test)]
mod tests {

    use super::*;
    use rand::{Rand, thread_rng};

    #[test]
    fn test_balance_tree() {
        let mut tree = BabyTransactionTree::new(3);
        let leaf = BabyTransactionLeaf::default();
        tree.insert(3, leaf);
        let root = tree.root_hash();
        let path = tree.merkle_path(0);
    }


}

// TODO: - Should not be here
// MARK: - MAIN Bench test for ios

const TREE_DEPTH: u32 = 24;
const NUMBER_OF_BLOCKS_TO_PROVE: u32 = 1;

#[no_mangle]
pub extern "C" fn test_benchmark_proof_gen_for_ios() {
    let params = &AltJubjubBn256::new();

    let rng = &mut XorShiftRng::from_seed([0x3dbe6258, 0x8d313d76, 0x3237db17, 0xe5bc0654]);

    let non_inclusion_level = 2;
    // println!("Proving for intersection level = {}", non_inclusion_level);

    let interval_length = Fr::from_str(&(1 << non_inclusion_level).to_string()).unwrap();
    // println!("Interval length = {}", interval_length);

    let mut witnesses = vec![];

    let start_of_slice = 0u32;
    let index_as_field_element = Fr::from_str(&start_of_slice.to_string()).unwrap();

    for _ in 0..NUMBER_OF_BLOCKS_TO_PROVE {
        // create an empty tree

        let mut tree = BabyTransactionTree::new(TREE_DEPTH);

        // test will prove the large [0, 3] (length 4), 
        // so we need to enter non-zero element at the leaf number 4

        let mut random_bools = vec![];
        for _ in 0..256 {
            let bit: bool = rng.gen::<bool>();
            random_bools.push(bit);
        }

        let empty_leaf = BabyTransactionLeaf::default();

        let non_empty_leaf = BabyTransactionLeaf {
                hash:    random_bools,
                phantom: std::marker::PhantomData
        };

        // println!("Inserting a non-empty leaf");

        let slice_len = 1 << non_inclusion_level;

        tree.insert(slice_len, non_empty_leaf.clone());

        let root = tree.root_hash();
        // println!("Root = {}", root);

        // println!("Checking reference proofs");

        assert!(tree.verify_proof(slice_len, non_empty_leaf.clone(), tree.merkle_path(slice_len)));
        assert!(tree.verify_proof(start_of_slice, empty_leaf.clone(), tree.merkle_path(start_of_slice)));

        {
            let proof = tree.merkle_path(start_of_slice);
            let proof_as_some: Vec<Option<Fr>> = proof.into_iter().map(|e| Some(e.0)).collect();

            let block_witness: BlockWitness<Bn256> = BlockWitness {
                root: Some(root),
                proof: proof_as_some
            };

            witnesses.push(block_witness);
        }
    }

    println!("Using test constraint system to check the satisfiability");

    {
        let mut cs = TestConstraintSystem::<Bn256>::new();

        let instance = NonInclusion {
            params: params,
            number_of_blocks: NUMBER_OF_BLOCKS_TO_PROVE as usize,
            leaf_hash_length: 256,
            tree_depth: TREE_DEPTH as usize,
            interval_length: Some(interval_length),
            index: Some(index_as_field_element),
            witness: witnesses.clone(),
        };

        println!("Synthsizing a snark for {} block for {} tree depth", NUMBER_OF_BLOCKS_TO_PROVE, TREE_DEPTH);

        instance.synthesize(&mut cs).unwrap();

        println!("Looking for unconstrained variabled: {}", cs.find_unconstrained());

        println!("Number of constraints = {}", cs.num_constraints());
        // inputs are ONE, starting index, slice length + root * number of blocks 
        // assert_eq!(cs.num_inputs(), (1 + 1 + 1 + NUMBER_OF_BLOCKS_TO_PROVE) as usize);

        let err = cs.which_is_unsatisfied();
        if err.is_some() {
            panic!("ERROR satisfying in {}\n", err.unwrap());
        }
    }
    let empty_witness: BlockWitness<Bn256> = BlockWitness {
            root: None,
            proof: vec![None; TREE_DEPTH as usize]
        };

    let instance_for_generation = NonInclusion {
        params: params,
        number_of_blocks: NUMBER_OF_BLOCKS_TO_PROVE as usize,
        leaf_hash_length: 256,
        tree_depth: TREE_DEPTH as usize,
        interval_length: None,
        index: None,
        witness: vec![empty_witness; NUMBER_OF_BLOCKS_TO_PROVE as usize],
    };

    println!("generating setup...");
    let start = PreciseTime::now();
    let circuit_params = generate_random_parameters(instance_for_generation, rng).unwrap();
    println!("setup generated in {} s", start.to(PreciseTime::now()).num_milliseconds() as f64 / 1000.0);

    let instance_for_proof = NonInclusion {
        params: params,
        number_of_blocks: NUMBER_OF_BLOCKS_TO_PROVE as usize,
        leaf_hash_length: 256,
        tree_depth: TREE_DEPTH as usize,
        interval_length: Some(interval_length),
        index: Some(index_as_field_element),
        witness: witnesses.clone(),
    };

    let pvk = prepare_verifying_key(&circuit_params.vk);

    println!("creating proof...");
    let start = PreciseTime::now();
    let proof = create_random_proof(instance_for_proof, &circuit_params, rng).unwrap();
    println!("proof created in {} s", start.to(PreciseTime::now()).num_milliseconds() as f64 / 1000.0);

    let mut public_inputs = vec![];
    public_inputs.push(index_as_field_element);
    public_inputs.push(interval_length);
    public_inputs.extend(witnesses.into_iter().map(|e| e.root.clone().unwrap()));

    let success = verify_proof(&pvk, &proof, &public_inputs).unwrap();
    assert!(success);
    println!("Proof is valid");
}
