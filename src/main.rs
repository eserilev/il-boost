use std::{str::FromStr, sync::Arc};

use alloy::{primitives::FixedBytes, rpc::types::beacon::BlsPublicKey};
use config::InclusionListConfig;
use cb_common::{commit::request::SignRequest, config::{load_module_config, StartModuleConfig}, utils::initialize_tracing_log};
use mock_relay::MockRelay;
use tree_hash::Hash256;
use types::{BeaconBlock, InclusionListSummary, InclusionProof, SignedBeaconBlock};

mod config;
mod mock_relay;
mod types;




#[tokio::main]
async fn main() -> Result<(), ()> {
    initialize_tracing_log();

    let config = load_module_config::<InclusionListConfig>().expect("failed to load config");

    // a beacon node delegates its proposer rights to commit-boost
    let pubkey = BlsPublicKey::default();

    let relay = MockRelay::new();

    // the il module fetches the inclusion list
    // TODO expect()
    let inclusion_list = get_inclusion_list(u64::default(), u64::default()).expect("should get inclusion list");

    // call get header with proof
    // this returns a execution payload header with an associated inclusion proof
    // TODO unwrap()
    let header_with_proof = relay.get_header_with_proof(u64::default(), Hash256::default(), pubkey).unwrap();

    // TODO expect()
    if verify_inclusion_proof(header_with_proof.proof, inclusion_list).expect("should return a bool") {
        // generate the beacon block w/ the execution payload header and sign it
        // TODO unwrap()
        let signature = sign_beacon_block(config, pubkey, u64::default(), todo!()).await.unwrap();
        let signed_beacon_block = SignedBeaconBlock { beacon_block: todo!(), signature };
        relay.submit_signed_block(signed_beacon_block);
    }
   
    Ok(())
}

fn get_inclusion_list(slot: u64, proposer_index: u64,) -> Result<InclusionListSummary, ()>  {
    // this should return a list of censored transactions
    // if the txn is in the mempool, pays the base fee,
    // has a non-zero tip, and there is gas remaining in the block, 
    // it is being censored
    Ok(InclusionListSummary {
        slot,
        proposer_index,
        summary: vec![],
    })
}

fn verify_inclusion_proof(inclusion_proof: InclusionProof, inclusion_list: InclusionListSummary) -> Result<bool, ()> {
    todo!()
}

fn sign_execution_payload_header() {
    todo!()
}

async fn sign_beacon_block(config: StartModuleConfig<InclusionListConfig>, pubkey: BlsPublicKey, slot: u64, beacon_block: BeaconBlock) -> Result<FixedBytes<96>, ()> {

    let request = SignRequest::builder(&config.id, pubkey).with_msg(&beacon_block);

    // TODO unwrap()
    let signature  = config.signer_client.request_signature(&request).await.unwrap();

    Ok(signature)
}