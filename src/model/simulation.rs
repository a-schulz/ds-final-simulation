use std::collections::VecDeque;
use rand::{thread_rng, Rng};
use rand::distributions::{Distribution, Uniform};
use rand_distr::Normal;
use crate::config::SimulationConfig;
use crate::model::event::{Event, EventQueue};
use crate::model::part::Part;

pub struct ProductionModel {
    pub rng: rand::rngs::ThreadRng,
    pub current_time: f64,
    pub next_id: usize,
    pub buffer_p1: VecDeque<Part>,
    pub buffer_p2: VecDeque<Part>,
    pub buffer_p3: VecDeque<Part>,
    pub montage_stations_available: usize,
    pub test_stations_available: usize,
    pub parts_completed: Vec<Part>,
    pub lotbad_batch_time: f64,
    pub lotbad_current_batch: Vec<Part>,
    pub config: SimulationConfig,
}

impl ProductionModel {
    pub fn new(config: SimulationConfig) -> Self {
        Self {
            rng: thread_rng(),
            current_time: 0.0,
            next_id: 0,
            buffer_p1: VecDeque::with_capacity(config.buffer_capacity),
            buffer_p2: VecDeque::with_capacity(config.buffer_capacity),
            buffer_p3: VecDeque::with_capacity(config.buffer_capacity),
            montage_stations_available: config.montage_stations,
            test_stations_available: config.test_stations,
            parts_completed: Vec::new(),
            lotbad_batch_time: 0.0,
            lotbad_current_batch: Vec::new(),
            config,
        }
    }

    fn start_smd_process(&mut self, event_queue: &mut EventQueue) {
        // Neue Bauteile starten, wenn möglich
        if self.buffer_p1.len() < self.config.buffer_capacity {
            let part = Part {
                id: self.next_id,
                entry_time: self.current_time,
                stage_times: Vec::new(),
            };
            self.next_id += 1;

            // SMD-Zeit ist konstant
            let completion_time = self.current_time + self.config.smd_time_minutes;
            event_queue.add(completion_time, Event::SMDFinished(part));
        }
    }

    fn process_lotbad(&mut self, event_queue: &mut EventQueue) {
        // Starte Lotbad-Prozess, wenn genügend Teile vorhanden
        if self.lotbad_current_batch.is_empty() && !self.buffer_p1.is_empty() {
            // Zeit für einen Durchlauf (6 Minuten entsprechend 10 Durchläufe/Stunde)
            let batch_time = 60.0 / (self.config.lotbad_batches_per_hour as f64);

            // Sammle Teile für den Batch
            while self.lotbad_current_batch.len() < self.config.lotbad_batch_size && !self.buffer_p1.is_empty() {
                if let Some(part) = self.buffer_p1.pop_front() {
                    self.lotbad_current_batch.push(part);
                }
            }

            // Plane Fertigstellung
            event_queue.add(
                self.current_time + batch_time,
                Event::LotbadBatchFinished(self.lotbad_current_batch.clone())
            );
            self.lotbad_current_batch.clear(); // Reset für nächsten Batch
        }
    }

    fn start_montage(&mut self, event_queue: &mut EventQueue) {
        while self.montage_stations_available > 0 && !self.buffer_p2.is_empty() {
            if let Some(part) = self.buffer_p2.pop_front() {
                self.montage_stations_available -= 1;

                // Gleichverteilte Zeit zwischen 3 und 5 Minuten
                let dist = Uniform::new(self.config.montage_min_minutes, self.config.montage_max_minutes);
                let montage_time = dist.sample(&mut self.rng);

                event_queue.add(
                    self.current_time + montage_time,
                    Event::MontageFinished(part)
                );
            }
        }
    }

    fn start_qc(&mut self, event_queue: &mut EventQueue) {
        while self.test_stations_available > 0 && !self.buffer_p3.is_empty() {
            if let Some(part) = self.buffer_p3.pop_front() {
                self.test_stations_available -= 1;

                // Normalverteilte Zeit mit Minimum
                let normal = Normal::new(self.config.qc_mean_minutes, self.config.qc_std_dev_minutes).unwrap();
                let mut qc_time = normal.sample(&mut self.rng);

                // Mindestens 3 Minuten
                if qc_time < self.config.qc_min_minutes {
                    qc_time = self.config.qc_min_minutes;
                }

                event_queue.add(
                    self.current_time + qc_time,
                    Event::QCFinished(part)
                );
            }
        }
    }

    pub fn run_simulation(&mut self) {
        let mut event_queue = EventQueue::new();

        // Ende der Simulation planen
        let end_time = self.config.simulation_duration_hours * 60.0;
        event_queue.add(end_time, Event::SimulationEnd);

        // Starte den ersten SMD-Prozess
        self.start_smd_process(&mut event_queue);

        // Hauptschleife der Simulation
        while let Some((time, event)) = event_queue.next_event() {
            self.current_time = time;

            match event {
                Event::SMDFinished(part) => {
                    // Teil zum Puffer P1 hinzufügen
                    self.buffer_p1.push_back(part);
                    // Starte nächsten SMD-Prozess
                    self.start_smd_process(&mut event_queue);
                    // Prüfe, ob Lotbad-Prozess starten kann
                    self.process_lotbad(&mut event_queue);
                },

                Event::LotbadBatchFinished(parts) => {
                    // Teile zum Puffer P2 hinzufügen
                    for part in parts {
                        if self.buffer_p2.len() < self.config.buffer_capacity {
                            self.buffer_p2.push_back(part);
                        }
                    }
                    // Starte Montage-Prozesse, falls möglich
                    self.start_montage(&mut event_queue);
                    // Prüfe, ob weiterer Lotbad-Prozess starten kann
                    self.process_lotbad(&mut event_queue);
                },

                Event::MontageFinished(part) => {
                    // Station wieder verfügbar
                    self.montage_stations_available += 1;
                    // Teil zum Puffer P3 hinzufügen
                    self.buffer_p3.push_back(part);
                    // Starte weitere Montage-Prozesse, falls möglich
                    self.start_montage(&mut event_queue);
                    // Starte QC-Prozesse, falls möglich
                    self.start_qc(&mut event_queue);
                },

                Event::QCFinished(part) => {
                    // Station wieder verfügbar
                    self.test_stations_available += 1;
                    // Teil als fertig markieren
                    self.parts_completed.push(part);
                    // Starte weitere QC-Prozesse, falls möglich
                    self.start_qc(&mut event_queue);
                },

                Event::SimulationEnd => {
                    break;
                }
            }
        }
    }

    pub fn print_statistics(&self) {
        let total_parts = self.parts_completed.len();
        println!("Simulation abgeschlossen.");
        println!("Insgesamt fertiggestellte Teile: {}", total_parts);

        if !self.parts_completed.is_empty() {
            let total_throughput_time: f64 = self.parts_completed.iter()
                .map(|p| self.current_time - p.entry_time)
                .sum();
            let avg_throughput_time = total_throughput_time / total_parts as f64;
            println!("Durchschnittliche Durchlaufzeit: {:.2} Minuten", avg_throughput_time);
        }

        println!("Teile in Puffer P1: {}", self.buffer_p1.len());
        println!("Teile in Puffer P2: {}", self.buffer_p2.len());
        println!("Teile in Puffer P3: {}", self.buffer_p3.len());
    }
}