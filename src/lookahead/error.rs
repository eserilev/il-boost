use std::num::ParseIntError;

#[derive(Debug)]
pub enum LookaheadError {
    BeaconApiClientError(beacon_api_client::Error),
    Reqwest(reqwest::Error),
    Serde(serde_json::Error),
    ParseIntError(ParseIntError),
    FailedLookahead,
}

impl From<beacon_api_client::Error> for LookaheadError {
    fn from(value: beacon_api_client::Error) -> Self {
        LookaheadError::BeaconApiClientError(value)
    }
}

impl From<reqwest::Error> for LookaheadError {
    fn from(value: reqwest::Error) -> Self {
        LookaheadError::Reqwest(value)
    }
}

impl From<serde_json::Error> for LookaheadError {
    fn from(value: serde_json::Error) -> Self {
        LookaheadError::Serde(value)
    }
}

impl From<ParseIntError> for LookaheadError {
    fn from(value: ParseIntError) -> Self {
        LookaheadError::ParseIntError(value)
    }
}
