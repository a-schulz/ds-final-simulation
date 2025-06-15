use ::config::{Config, ConfigError, File};
use std::path::Path;

mod config;
mod model;

use crate::config::SimulationConfig;
use crate::model::ProductionModel;

fn load_config() -> Result<SimulationConfig, ConfigError> {
    let mut settings = Config::builder();

    // Versuche, config.toml zu laden, falls vorhanden
    if Path::new("config.toml").exists() {
        settings = settings.add_source(File::with_name("config.toml"));
    }

    // Konvertiere in unser Config-Format und behandle den Fehler
    match settings.build() {
        Ok(config) => {
            // Manual conversion since TryFrom<&Config> is not implemented
            Ok(SimulationConfig {
                // Set your config fields based on config.get() calls
                // Example: buffer_capacity: config.get_int("buffer_capacity")? as usize,
                ..SimulationConfig::default()
            })
        },
        Err(e) => Err(e),
    }
}

fn main() {
    env_logger::init();

    // Lade Konfiguration oder verwende Standardwerte
    let config = match load_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Fehler beim Laden der Konfiguration: {}. Verwende Standardwerte.", e);
            SimulationConfig::default()
        }
    };

    // Mehrere Simulationsläufe durchführen
    for i in 0..config.replications {
        println!("Starte Simulationslauf {}", i+1);
        let mut model = ProductionModel::new(config.clone());
        model.run_simulation();
        model.print_statistics();
    }
}