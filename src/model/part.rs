use std::collections::VecDeque;
use rand::distributions::{Distribution, Uniform, Normal};
use rand::rngs::ThreadRng;

// Bauteil, das durch das System läuft
#[derive(Debug)]
struct Part {
    id: usize,
    entry_time: f64,
    stage_times: Vec<f64>,
}

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