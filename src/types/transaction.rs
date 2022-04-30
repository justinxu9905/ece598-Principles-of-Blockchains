use serde::{Serialize, Deserialize};
use ring::signature::{Ed25519KeyPair, Signature, KeyPair, VerificationAlgorithm, EdDSAParameters, UnparsedPublicKey, ED25519};
use rand::{Rng, thread_rng};
use ring::digest::{digest, SHA256};
use crate::types::hash::{Hashable, H256};
use std::collections::HashMap;
use crate::types::address::Address;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
    pub sender: Address,
    pub acc_nonce: u32,
    pub receiver: Address,
    pub value: u32,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SignedTransaction {
    pub transaction: Transaction,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
}


impl Hashable for SignedTransaction {
    fn hash(&self) -> H256 {
        digest(&SHA256, &bincode::serialize(&self).unwrap()).into()
    }
}


/// Create digital signature of a transaction
pub fn sign(t: &Transaction, key: &Ed25519KeyPair) -> Signature {
    let converted_t = bincode::serialize(t).unwrap();
    let sig = key.sign(&converted_t);
    sig
}

/// Verify digital signature of a transaction, using public key instead of secret key
pub fn verify(t: &Transaction, public_key: &[u8], signature: &[u8]) -> bool {
    let converted_t = bincode::serialize(t).unwrap();
    let peer_public_key =
    UnparsedPublicKey::new(&ED25519, public_key);
    let verification = peer_public_key.verify(&converted_t, signature);
    if verification.is_ok() {
        return true;
    }
    false
}










