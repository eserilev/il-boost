use alloy::rpc::types::beacon::BlsPublicKey;
use tree_hash::Hash256;

use crate::types::{ExecutionPayloadHeaderWithProof, SignedBeaconBlock};

pub struct MockRelay {

}


impl MockRelay {
    pub fn new() -> Self {
        Self{

        }
    }

    pub fn send_constraints(&self) {
        todo!()
    }

    // call /eth/v1/builder/header_with_proofs/{slot}/{parent_hash}/{pubkey}
    pub fn get_header_with_proof(&self, slot: u64, parent_hash: Hash256, pubkey: BlsPublicKey)-> Result<ExecutionPayloadHeaderWithProof, ()> {
        todo!()
    }

    // call ​/eth​/v1​/builder​/blinded_blocks
    pub fn submit_signed_block(&self, signed_blinded_block: SignedBeaconBlock) {
        todo!()
    }


}