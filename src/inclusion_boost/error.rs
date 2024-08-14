use std::{num::ParseIntError, str::Utf8Error};

use alloy::transports::TransportErrorKind;
use cb_common::commit::error::SignerClientError;

use crate::lookahead::error::LookaheadError;

#[derive(Debug)]
pub enum InclusionListBoostError {
    GenericError(String),
    BeaconApiError(beacon_api_client::Error),
    Reqwest(reqwest::Error),
    SseError(mev_share_sse::client::SseError),
    AlloyRpcError(alloy::transports::RpcError<TransportErrorKind>),
    SignerClientError(SignerClientError),
    // Utf8Error(Utf8Error),
    LookaheadError(LookaheadError),
    ParseIntError(ParseIntError),
    Serde(serde_json::Error),
}

impl From<String> for InclusionListBoostError {
    fn from(value: String) -> Self {
        InclusionListBoostError::GenericError(value)
    }
}

impl From<beacon_api_client::Error> for InclusionListBoostError {
    fn from(value: beacon_api_client::Error) -> Self {
        InclusionListBoostError::BeaconApiError(value)
    }
}

impl From<reqwest::Error> for InclusionListBoostError {
    fn from(value: reqwest::Error) -> Self {
        InclusionListBoostError::Reqwest(value)
    }
}

impl From<mev_share_sse::client::SseError> for InclusionListBoostError {
    fn from(value: mev_share_sse::client::SseError) -> Self {
        InclusionListBoostError::SseError(value)
    }
}

impl From<alloy::transports::RpcError<TransportErrorKind>> for InclusionListBoostError {
    fn from(value: alloy::transports::RpcError<TransportErrorKind>) -> Self {
        InclusionListBoostError::AlloyRpcError(value)
    }
}

impl From<SignerClientError> for InclusionListBoostError {
    fn from(value: SignerClientError) -> Self {
        InclusionListBoostError::SignerClientError(value)
    }
}

impl From<LookaheadError> for InclusionListBoostError {
    fn from(value: LookaheadError) -> Self {
        InclusionListBoostError::LookaheadError(value)
    }
}


impl From<ParseIntError> for InclusionListBoostError {
    fn from(value: ParseIntError) -> Self {
        InclusionListBoostError::ParseIntError(value)
    }
}

impl From<serde_json::Error> for InclusionListBoostError {
    fn from(value: serde_json::Error) -> Self {
        InclusionListBoostError::Serde(value)
    }
}


// impl From<Utf8Error> for InclusionListBoostError {
//     fn from(value: Utf8Error) -> Self {
//         InclusionListBoostError::Utf8Error(value)
//     }
// }
