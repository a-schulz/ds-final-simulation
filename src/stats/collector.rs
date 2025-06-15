use serde::{Serialize, Deserialize};
use crate::model::simulation::ProductionModel;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SimulationStats {
    pub total_parts_completed: usize,
    pub avg_throughput_time: f64,
    pub max_throughput_time: f64,
    pub min_throughput_time: f64,
    pub avg_p1_utilization: f64,
    pub avg_p2_utilization: f64,
    pub avg_p3_utilization: f64,
    pub avg_montage_utilization: f64,
    pub avg_qc_utilization: f64,
    pub runs_completed: usize,
}

impl SimulationStats {
    pub fn new() -> Self {
        Self {
            total_parts_completed: 0,
            avg_throughput_time: 0.0,
            max_throughput_time: 0.0,
            min_throughput_time: f64::MAX,
            avg_p1_utilization: 0.0,
            avg_p2_utilization: 0.0,
            avg_p3_utilization: 0.0,
            avg_montage_utilization: 0.0,
            avg_qc_utilization: 0.0,
            runs_completed: 0,
        }
    }

    pub fn add_run_results(&mut self, model: ProductionModel) {
        self.runs_completed += 1;
        self.total_parts_completed += model.parts_completed.len();

        // Berechnung der Durchlaufzeiten
        if !model.parts_completed.is_empty() {
            let n = model.parts_completed.len() as f64;
            let total_throughput: f64 = model.parts_completed.iter()
                .map(|p| model.current_time - p.entry_time)
                .sum();

            let max_time = model.parts_completed.iter()
                .map(|p| model.current_time - p.entry_time)
                .fold(0.0, f64::max);

            let min_time = model.parts_completed.iter()
                .map(|p| model.current_time - p.entry_time)
                .fold(f64::MAX, f64::min);

            // Update der Statistiken
            self.avg_throughput_time = (self.avg_throughput_time * (self.runs_completed - 1) as f64
                + total_throughput / n) / self.runs_completed as f64;

            self.max_throughput_time = self.max_throughput_time.max(max_time);
            self.min_throughput_time = self.min_throughput_time.min(min_time);
        }

        // Berechnung der Auslastung der Puffer
        let buffer_p1_util = model.buffer_p1.len() as f64 / model.config.buffer_capacity as f64;
        let buffer_p2_util = model.buffer_p2.len() as f64 / model.config.buffer_capacity as f64;
        let buffer_p3_util = model.buffer_p3.len() as f64 / model.config.buffer_capacity as f64;

        self.avg_p1_utilization = (self.avg_p1_utilization * (self.runs_completed - 1) as f64
            + buffer_p1_util) / self.runs_completed as f64;
        self.avg_p2_utilization = (self.avg_p2_utilization * (self.runs_completed - 1) as f64
            + buffer_p2_util) / self.runs_completed as f64;
        self.avg_p3_utilization = (self.avg_p3_utilization * (self.runs_completed - 1) as f64
            + buffer_p3_util) / self.runs_completed as f64;

        // Berechnung der Auslastung der Stationen
        let montage_util = (model.config.montage_stations - model.montage_stations_available) as f64
            / model.config.montage_stations as f64;
        let qc_util = (model.config.test_stations - model.test_stations_available) as f64
            / model.config.test_stations as f64;

        self.avg_montage_utilization = (self.avg_montage_utilization * (self.runs_completed - 1) as f64
            + montage_util) / self.runs_completed as f64;
        self.avg_qc_utilization = (self.avg_qc_utilization * (self.runs_completed - 1) as f64
            + qc_util) / self.runs_completed as f64;
    }
}