use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SimulationConfig {
    // Stationszeiten
    pub smd_time_minutes: f64,
    pub lotbad_batch_size: usize,
    pub lotbad_batches_per_hour: usize,
    pub montage_min_minutes: f64,
    pub montage_max_minutes: f64,
    pub qc_mean_minutes: f64,
    pub qc_std_dev_minutes: f64,
    pub qc_min_minutes: f64,

    // Kapazitäten
    pub buffer_capacity: usize,
    pub montage_stations: usize,
    pub test_stations: usize,

    // Simulationsparameter
    pub simulation_duration_hours: f64,
    pub warm_up_period_hours: f64,
    pub replications: usize,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            smd_time_minutes: 1.0,
            lotbad_batch_size: 6,
            lotbad_batches_per_hour: 10,
            montage_min_minutes: 3.0,
            montage_max_minutes: 5.0,
            qc_mean_minutes: 6.0,
            qc_std_dev_minutes: 4.0,
            qc_min_minutes: 3.0,
            buffer_capacity: 15,
            montage_stations: 2,
            test_stations: 2,
            simulation_duration_hours: 8.0,
            warm_up_period_hours: 1.0,
            replications: 5,
        }
    }
}