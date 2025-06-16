use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub process_times: ProcessTimes,
    pub resources: Resources,
    pub buffer_capacities: BufferCapacities,
    pub simulation: SimulationConfig,
}

#[derive(Debug, Deserialize)]
pub struct ProcessTimes {
    pub smd_placement_time: f64,
    pub lot_bath_capacity: usize,
    pub lot_bath_batches_per_hour: usize,
    pub assembly_time_min: f64,
    pub assembly_time_max: f64,
    pub quality_control_mean: f64,
    pub quality_control_std_dev: f64,
    pub quality_control_min_time: f64,
}

#[derive(Debug, Deserialize)]
pub struct Resources {
    pub smd_machines: usize,
    pub lot_bath_machines: usize,
    pub assembly_workstations: usize,
    pub test_stations: usize,
}

#[derive(Debug, Deserialize)]
pub struct BufferCapacities {
    pub pre_lot_bath_buffer: usize,
    pub pre_assembly_buffer: usize,
    pub pre_test_buffer: usize,
}

#[derive(Debug, Deserialize)]
pub struct SimulationConfig {
    pub simulation_time: u64,
    pub arrival_rate: f64,
    pub seed: u64,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}
