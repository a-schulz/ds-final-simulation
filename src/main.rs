use std::collections::VecDeque;
use std::thread;
use std::time::{Duration, Instant};
use clap::{Parser, ValueEnum};
use colored::*;
use crossterm::{cursor, terminal, ExecutableCommand};
use std::io::{stdout, Write};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
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

// CLI Arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Simulation speed factor (higher is faster)
    #[arg(short, long, default_value_t = 1.0)]
    speed: f64,

    /// Number of parts to simulate
    #[arg(short, long, default_value_t = 50)]
    parts: usize,

    /// Display mode
    #[arg(short, long, default_value = "animated")]
    mode: DisplayMode,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, ValueEnum)]
enum DisplayMode {
    Animated,
    Summary,
}

// Simulation components
#[derive(Clone)]
struct Part {
    id: usize,
    entry_time: Instant,
}

struct SMDMachine {
    processing_time: Duration, // 1 minute constant
    current_part: Option<(Part, Instant)>,
}

struct LotBad {
    batch_size: usize,         // 6 pieces per batch
    batch_time: Duration,      // 6 minutes per batch
    current_batch: Vec<Part>,
    batch_start_time: Option<Instant>,
}

struct Buffer {
    name: String,
    capacity: usize,
    parts: VecDeque<Part>,
}

struct Simulation {
    parts_created: usize,
    parts_completed: usize,
    total_parts: usize,
    storage_to_smd_buffer: Buffer,
    smd_machine: SMDMachine,
    smd_to_lotbad_buffer: Buffer,
    lotbad: LotBad,
    speed_factor: f64,
    display_mode: DisplayMode,
    start_time: Instant,
    config: SimulationConfig,
}

impl SMDMachine {
    fn new(config: &SimulationConfig) -> Self {
        Self {
            processing_time: Duration::from_secs_f64(config.smd_time_minutes * 60.0),
            current_part: None,
        }
    }

    fn is_available(&self) -> bool {
        self.current_part.is_none()
    }

    fn start_processing(&mut self, part: Part) {
        self.current_part = Some((part, Instant::now()));
    }

    fn check_completed(&mut self, speed_factor: f64) -> Option<Part> {
        if let Some((part, start_time)) = &self.current_part {
            let adjusted_time = Duration::from_secs_f64(
                self.processing_time.as_secs_f64() / speed_factor
            );

            if start_time.elapsed() >= adjusted_time {
                let completed_part = part.clone();
                self.current_part = None;
                return Some(completed_part);
            }
        }
        None
    }
}

impl Part {
    fn new(id: usize) -> Self {
        Self {
            id,
            entry_time: Instant::now(),
        }
    }
}

impl Buffer {
    fn new(name: &str, capacity: usize) -> Self {
        Self {
            name: name.to_string(),
            capacity,
            parts: VecDeque::new(),
        }
    }

    fn add_part(&mut self, part: Part) -> bool {
        if self.parts.len() < self.capacity {
            self.parts.push_back(part);
            true
        } else {
            false
        }
    }

    fn take_part(&mut self) -> Option<Part> {
        self.parts.pop_front()
    }

    fn is_full(&self) -> bool {
        self.parts.len() >= self.capacity
    }

    fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    fn size(&self) -> usize {
        self.parts.len()
    }
}

impl LotBad {
    fn new(config: &SimulationConfig) -> Self {
        // Calculate batch time: 60 minutes / batches_per_hour
        let batch_minutes = 60.0 / config.lotbad_batches_per_hour as f64;

        Self {
            batch_size: config.lotbad_batch_size,
            batch_time: Duration::from_secs_f64(batch_minutes * 60.0), // Convert to seconds
            current_batch: Vec::new(),
            batch_start_time: None,
        }
    }

    fn add_part(&mut self, part: Part) -> bool {
        if self.current_batch.len() < self.batch_size {
            self.current_batch.push(part);

            // Start the batch timer when the first part is added
            if self.current_batch.len() == 1 {
                self.batch_start_time = Some(Instant::now());
            }

            true
        } else {
            false
        }
    }

    fn is_batch_ready(&self) -> bool {
        self.current_batch.len() >= self.batch_size
    }

    fn is_batch_complete(&self, speed_factor: f64) -> bool {
        if let Some(start_time) = self.batch_start_time {
            let adjusted_time = Duration::from_secs_f64(
                self.batch_time.as_secs_f64() / speed_factor
            );

            if self.is_batch_ready() && start_time.elapsed() >= adjusted_time {
                return true;
            }
        }
        false
    }

    fn complete_batch(&mut self) -> Vec<Part> {
        let completed_batch = std::mem::take(&mut self.current_batch);
        self.batch_start_time = None;
        completed_batch
    }

    fn batch_progress(&self, speed_factor: f64) -> f64 {
        if let Some(start_time) = self.batch_start_time {
            let adjusted_time = Duration::from_secs_f64(
                self.batch_time.as_secs_f64() / speed_factor
            );

            let elapsed = start_time.elapsed();
            (elapsed.as_secs_f64() / adjusted_time.as_secs_f64()).min(1.0)
        } else {
            0.0
        }
    }
}

impl Simulation {
    fn new(args: &Args) -> Self {
        let config = SimulationConfig::default();

        Self {
            parts_created: 0,
            parts_completed: 0,
            total_parts: args.parts,
            storage_to_smd_buffer: Buffer::new("Storage to SMD", config.buffer_capacity),
            smd_machine: SMDMachine::new(&config),
            smd_to_lotbad_buffer: Buffer::new("SMD to LotBad", config.buffer_capacity),
            lotbad: LotBad::new(&config),
            speed_factor: args.speed,
            display_mode: args.mode,
            start_time: Instant::now(),
            config,
        }
    }

    fn run(&mut self) {
        let mut stdout = stdout();
        if self.display_mode == DisplayMode::Animated {
            terminal::enable_raw_mode().expect("Could not enable raw mode");
            stdout.execute(terminal::Clear(terminal::ClearType::All)).unwrap();
            stdout.execute(cursor::Hide).unwrap();
        }

        loop {
            // Process one simulation tick
            self.tick();

            // Display the current state
            if self.display_mode == DisplayMode::Animated {
                self.display_animated(&mut stdout);
                thread::sleep(Duration::from_millis(100));
            }

            // Check if simulation is complete
            if self.parts_completed >= self.total_parts &&
                self.storage_to_smd_buffer.is_empty() &&
                self.smd_machine.current_part.is_none() &&
                self.smd_to_lotbad_buffer.is_empty() &&
                self.lotbad.current_batch.is_empty() {
                break;
            }
        }

        if self.display_mode == DisplayMode::Animated {
            stdout.execute(cursor::Show).unwrap();
            terminal::disable_raw_mode().expect("Could not disable raw mode");
        }

        // Print final statistics
        self.display_summary();
    }

    fn tick(&mut self) {
        // 1. Check if lotbad batch is complete and process it
        if self.lotbad.is_batch_complete(self.speed_factor) {
            let completed_parts = self.lotbad.complete_batch();
            self.parts_completed += completed_parts.len();
        }

        // 2. Move parts from SMD to LotBad buffer if possible
        if let Some(completed_part) = self.smd_machine.check_completed(self.speed_factor) {
            if self.smd_to_lotbad_buffer.add_part(completed_part.clone()) {
                // Part moved to buffer
            } else {
                // Buffer is full, put the part back in the machine
                // (This shouldn't happen in a well-balanced system)
                self.smd_machine.current_part = Some((completed_part, Instant::now()));
            }
        }

        // 3. Fill lotbad with parts from buffer when possible
        if !self.lotbad.is_batch_ready() && !self.smd_to_lotbad_buffer.is_empty() {
            if let Some(part) = self.smd_to_lotbad_buffer.take_part() {
                self.lotbad.add_part(part);
            }
        }

        // 4. Start SMD processing if machine is available and there are parts in buffer
        if self.smd_machine.is_available() && !self.storage_to_smd_buffer.is_empty() {
            if let Some(part) = self.storage_to_smd_buffer.take_part() {
                self.smd_machine.start_processing(part);
            }
        }

        // 5. Add new parts to the system if we haven't created all parts yet
        if self.parts_created < self.total_parts && !self.storage_to_smd_buffer.is_full() {
            let new_part = Part::new(self.parts_created + 1);
            if self.storage_to_smd_buffer.add_part(new_part) {
                self.parts_created += 1;
            }
        }
    }

    fn display_animated(&self, stdout: &mut std::io::Stdout) {
        stdout.execute(cursor::MoveTo(0, 0)).unwrap();

        println!("{}", "SMD Manufacturing Process Simulation".green().bold());
        println!("----------------------------------------");
        println!("Time elapsed: {:.1} minutes", self.start_time.elapsed().as_secs_f64() / 60.0);
        println!("Speed factor: {:.1}x", self.speed_factor);
        println!("Parts created: {}/{}", self.parts_created, self.total_parts);
        println!("Parts completed: {}/{}", self.parts_completed, self.total_parts);
        println!();

        // Storage to SMD Buffer
        let buffer_fill = (self.storage_to_smd_buffer.size() as f64 / self.storage_to_smd_buffer.capacity as f64 * 20.0).round() as usize;
        println!("Storage → SMD Buffer [{}{}] {}/{}",
                 "█".repeat(buffer_fill),
                 " ".repeat(20 - buffer_fill),
                 self.storage_to_smd_buffer.size(),
                 self.storage_to_smd_buffer.capacity
        );

        // SMD Machine
        if let Some((part, _)) = &self.smd_machine.current_part {
            println!("SMD Machine: Processing Part #{} 🔄", part.id);
        } else {
            println!("SMD Machine: Idle ⏸");
        }

        // SMD to LotBad Buffer
        let buffer_fill = (self.smd_to_lotbad_buffer.size() as f64 / self.smd_to_lotbad_buffer.capacity as f64 * 20.0).round() as usize;
        println!("SMD → LotBad Buffer [{}{}] {}/{}",
                 "█".repeat(buffer_fill),
                 " ".repeat(20 - buffer_fill),
                 self.smd_to_lotbad_buffer.size(),
                 self.smd_to_lotbad_buffer.capacity
        );

        // LotBad
        let batch_progress = (self.lotbad.batch_progress(self.speed_factor) * 20.0).round() as usize;
        println!("LotBad: {} parts in batch [{}/{}]",
                 self.lotbad.current_batch.len(),
                 self.lotbad.current_batch.len(),
                 self.lotbad.batch_size
        );

        if !self.lotbad.current_batch.is_empty() {
            println!("Batch Progress: [{}{}] {:.1}%",
                     "█".repeat(batch_progress),
                     " ".repeat(20 - batch_progress),
                     self.lotbad.batch_progress(self.speed_factor) * 100.0
            );
        }

        println!("\nPress Ctrl+C to exit");

        // Clear the rest of the screen to avoid artifacts
        for _ in 0..10 {
            println!("{}", " ".repeat(80));
        }

        stdout.flush().unwrap();
    }

    fn display_summary(&self) {
        println!("{}", "\nSimulation Complete".green().bold());
        println!("----------------------------------------");
        println!("Total simulation time: {:.1} minutes", self.start_time.elapsed().as_secs_f64() / 60.0);
        println!("Parts processed: {}", self.total_parts);
        println!("Average processing time per part: {:.2} minutes",
                 self.start_time.elapsed().as_secs_f64() / 60.0 / self.total_parts as f64);
    }
}

fn main() {
    let args = Args::parse();

    // Create and run the simulation
    println!("Starting SMD manufacturing simulation with {} parts at {:.1}x speed...",
             args.parts, args.speed);

    let mut simulation = Simulation::new(&args);
    simulation.run();
}
