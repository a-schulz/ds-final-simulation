// src/config.rs
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub simulation: SimulationConfig,
}

#[derive(Debug, Deserialize)]
pub struct SimulationConfig {
    pub arrival_min: f64,         // Minimum arrival time in minutes
    pub arrival_max: f64,         // Maximum arrival time in minutes
    pub max_swimmers: u64,        // Maximum capacity of the pool
    pub swim_time_min: f64,       // Minimum swimming time in minutes
    pub swim_time_max: f64,       // Maximum swimming time in minutes
    pub simulation_time: f64,     // Total simulation time in minutes
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}