use std::fs;
use std::io::Write;
use std::path::PathBuf;

use clap::Parser;
use concordium_rwa_events_listener::txn_processor;
use log::info;

#[derive(Parser, Debug, Clone)]
/// Configuration struct for OpenAPI.
pub struct Config {
    /// Output file path for the generated OpenAPI specs
    #[clap(long)]
    pub output: PathBuf,
}

fn main() {
    env_logger::init();
    let config = Config::parse();

    let api_service = txn_processor::create_service();
    let spec_json = api_service.spec();
    let mut file = fs::File::create(&config.output).expect("Error creating file");
    file.write_all(spec_json.as_bytes())
        .expect("Error writing to file");
    info!("OpenAPI specs generated at {}", config.output.display());
}
