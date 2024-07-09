
use config::InclusionListConfig;
use cb_common::{config::load_module_config, utils::initialize_tracing_log};

mod config;
mod client;




#[tokio::main]
async fn main() -> Result<(), ()> {
    initialize_tracing_log();

    let config = load_module_config::<InclusionListConfig>().expect("failed to load config");

    Ok(())
}

