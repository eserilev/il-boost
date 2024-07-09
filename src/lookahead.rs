use beacon_api_client::{mainnet::Client, Error, ProposerDuty};
use reqwest::Url;

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

    /// Get the proposer duties for UPCOMING slots.
    pub async fn get_current_lookahead(&self) -> Result<Vec<ProposerDuty>, Error> {
        tracing::info!("Getting current lookahead duties");

        let current_slot = get_slot(&self.url, "head")
            .await
            .expect("failed to get slot")
            .unwrap();

        let epoch = current_slot / 32;
        tracing::info!("Getting proposer duties for epoch: {}", epoch);

        let (_, duties) = self.client.get_proposer_duties(epoch).await?;

        Ok(duties
            .into_iter()
            .filter(|d| d.slot > current_slot)
            .collect::<Vec<_>>())
    }
}

async fn get_slot(beacon_url: &str, slot: &str) -> Result<Option<u64>, ()> {
    let url = format!("{}/eth/v1/beacon/headers/{}", beacon_url, slot);

    let res = reqwest::get(url).await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&res.text().await.unwrap()).unwrap();

    if let Some(slot) = json.pointer("/data/header/message/slot") {

        let slot_str = slot.as_str().unwrap();
        return Ok(Some(slot_str.parse::<u64>().unwrap()));
    }

    Ok(None)
}