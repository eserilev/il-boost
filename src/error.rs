
#[derive(Debug)]
pub enum CommitBoostError {
    Reqwest(reqwest::Error),
}

#[derive(Debug)]
pub enum InclusionListBoostError {
    GenericError(String),
    BeaconApiError(beacon_api_client::Error),
    Reqwest(reqwest::Error),
    SseError(mev_share_sse::client::SseError),
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