mod config;
mod model;
mod web;
mod stats;

use config::SimulationConfig;
use std::path::Path;
use web::server::start_server;

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Lade Konfiguration oder verwende Standardwerte
    let config = match config::load_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Fehler beim Laden der Konfiguration: {}. Verwende Standardwerte.", e);
            SimulationConfig::default()
        }
    };

    // Starte den Web-Server
    println!("Starte Web-Server auf http://localhost:8000");
    start_server(config).await?;

    Ok(())
}