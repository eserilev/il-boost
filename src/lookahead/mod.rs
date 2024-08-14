use beacon_api_client::{mainnet::Client, Error, ProposerDuty};
use error::LookaheadError;
use reqwest::Url;

pub mod error;

pub struct LookaheadProvider {
    client: Client,
    url: String,
}

impl LookaheadProvider {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            client: Client::new(Url::parse(url).unwrap()),
        }
    }

    /// Get proposer duties for the current epoch
    pub async fn get_current_lookahead(&self) -> Result<Vec<ProposerDuty>, LookaheadError> {
        tracing::info!("Getting current lookahead duties");

        let current_slot = get_slot(&self.url, "head")
            .await?
            .ok_or(LookaheadError::FailedLookahead)?;

        let epoch = current_slot / 32;
        tracing::info!("Getting proposer duties for epoch: {}", epoch);

        let (_, duties) = self.client.get_proposer_duties(epoch).await?;

        Ok(duties
            .into_iter()
            .filter(|d| d.slot > current_slot)
            .collect::<Vec<_>>())
    }
}



async fn get_slot(beacon_url: &str, slot: &str) -> Result<Option<u64>, LookaheadError> {
    let url = format!("{}/eth/v1/beacon/headers/{}", beacon_url, slot);
    let res = reqwest::get(url).await?;
    let json: serde_json::Value = serde_json::from_str(&res.text().await?)?;
    
    let Some(slot) = json.pointer("/data/header/message/slot") else {
        return Ok(None);
    };
    let Some(slot_str) = slot.as_str() else {
        return Ok(None);
    };
    Ok(Some(slot_str.parse::<u64>()?))
}
