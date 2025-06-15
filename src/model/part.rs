use std::cmp::Ordering;
use rand::distributions::{Distribution, Uniform};
// Note: Use rand_distr crate for Normal distribution
use rand_distr::Normal;
use std::collections::VecDeque;
use rand::rngs::ThreadRng;
use crate::SimulationConfig;

#[derive(Clone, Debug)]
pub struct Part {
    pub id: usize,
    pub entry_time: f64,
    pub stage_times: Vec<f64>,
}

impl PartialEq for Part {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Part {}

// The rest of your Part implementation

// Stationen im Produktionsprozess
enum ProcessStage {
    SMDAutomat,
    Lotbad,
    Montage,
    QualityControl,
    Finished,
}

// Simulationsmodell
struct ProductionModel {
    config: SimulationConfig,
    rng: ThreadRng,
    current_time: f64,
    next_id: usize,

    // Puffer zwischen den Stationen
    buffer_p1: VecDeque<Part>, // Nach SMD, vor Lotbad
    buffer_p2: VecDeque<Part>, // Nach Lotbad, vor Montage
    buffer_p3: VecDeque<Part>, // Nach Montage, vor QC

    // Ressourcen
    montage_stations_available: usize,
    test_stations_available: usize,

    // Statistiken
    parts_completed: Vec<Part>,
    lotbad_batch_time: f64,
    lotbad_current_batch: Vec<Part>,
}