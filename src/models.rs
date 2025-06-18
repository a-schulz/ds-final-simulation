use nexosim::model::{Context, Model};
use nexosim::ports::Output;
use rand::prelude::*;
use rand_distr::{Normal, Uniform};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::time::Duration;

// The main product that flows through the production system
#[derive(Debug, Clone)]
pub struct Telix1 {
    pub id: u64,
    pub entry_time: u64,
    pub completion_time: Option<u64>,
}

// SMD Placement machine model
#[derive(Default)]
pub struct SmdMachine {
    pub output: Output<Telix1>,
    pub process_time: f64,
    pub busy: bool,
}

impl SmdMachine {
    pub fn new(process_time: f64) -> Self {
        Self {
            output: Output::default(),
            process_time,
            busy: false,
        }
    }

    pub async fn input(&mut self, part: Telix1, cx: &mut Context<Self>) {
        self.busy = true;
        // Schedule completion after process time
        cx.schedule_event(Duration::from_secs_f64(self.process_time * 60.0), Self::complete, part)
            .unwrap();
    }

    async fn complete(&mut self, part: Telix1) {
        self.busy = false;
        self.output.send(part).await;
    }
}

impl Model for SmdMachine {}

// Buffer model for intermediate storage
#[derive(Default)]
pub struct Buffer {
    pub output: Output<Telix1>,
    pub capacity: usize,
    pub queue: Vec<Telix1>,
}

impl Buffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            output: Output::default(),
            capacity,
            queue: Vec::new(),
        }
    }

    pub async fn input(&mut self, part: Telix1) {
        if self.queue.len() < self.capacity {
            self.queue.push(part);
            // If this is the first item, send it immediately
            if self.queue.len() == 1 {
                let part = self.queue[0].clone();
                self.output.send(part).await;
            }
        }
    }

    // Called when downstream processing is complete
    pub async fn release(&mut self) {
        // Remove the first item
        if !self.queue.is_empty() {
            self.queue.remove(0);
        }

        // Send the next item if available
        if !self.queue.is_empty() {
            let part = self.queue[0].clone();
            self.output.send(part).await;
        }
    }
}

impl Model for Buffer {}

// Lot Bath model that processes items in batches
#[derive(Default)]
pub struct LotBath {
    pub output: Output<Telix1>,
    pub batch_size: usize,
    pub process_time: f64,
    pub current_batch: Vec<Telix1>,
    pub busy: bool,
}

impl LotBath {
    pub fn new(batch_size: usize, process_time: f64) -> Self {
        Self {
            output: Output::default(),
            batch_size,
            process_time,
            current_batch: Vec::new(),
            busy: false,
        }
    }

    pub async fn input(&mut self, part: Telix1, cx: &mut Context<Self>) {
        self.current_batch.push(part);

        // If batch is full, start processing
        if self.current_batch.len() >= self.batch_size || self.busy {
            if !self.busy {
                self.busy = true;
                // Schedule batch completion
                cx.schedule_event(
                    Duration::from_secs_f64(self.process_time * 60.0),
                    Self::complete_batch,
                    (),
                ).unwrap();
            }
        }
    }

    async fn complete_batch(&mut self, _: ()) {
        self.busy = false;

        // Send all parts from the batch
        for part in self.current_batch.drain(..) {
            self.output.send(part).await;
        }
    }
}

impl Model for LotBath {}

// Assembly workstation with uniformly distributed processing time
pub struct AssemblyStation {
    pub output: Output<Telix1>,
    pub min_time: f64,
    pub max_time: f64,
    pub busy: bool,
    pub rng: StdRng,
}

impl AssemblyStation {
    pub fn new(min_time: f64, max_time: f64, seed: u64) -> Self {
        Self {
            output: Output::default(),
            min_time,
            max_time,
            busy: false,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    pub async fn input(&mut self, part: Telix1, cx: &mut Context<Self>) {
        self.busy = true;

        // Generate random processing time with uniform distribution
        let distr = Uniform::new(self.min_time, self.max_time);
        let process_time = distr.sample(&mut self.rng);

        // Schedule completion
        cx.schedule_event(
            Duration::from_secs_f64(process_time * 60.0),
            Self::complete,
            part,
        ).unwrap();
    }

    async fn complete(&mut self, part: Telix1) {
        self.busy = false;
        self.output.send(part).await;
    }
}

impl Model for AssemblyStation {}

// Quality Control with normally distributed processing time and minimum time constraint
pub struct QualityControl {
    pub output: Output<Telix1>,
    pub mean_time: f64,
    pub std_dev: f64,
    pub min_time: f64,
    pub busy: bool,
    pub rng: StdRng,
}

impl QualityControl {
    pub fn new(mean_time: f64, std_dev: f64, min_time: f64, seed: u64) -> Self {
        Self {
            output: Output::default(),
            mean_time,
            std_dev,
            min_time,
            busy: false,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    pub async fn input(&mut self, part: Telix1, cx: &mut Context<Self>) {
        self.busy = true;

        // Generate random processing time with normal distribution
        let normal = Normal::new(self.mean_time, self.std_dev).unwrap();
        let mut process_time = normal.sample(&mut self.rng);

        // Apply minimum time constraint
        if process_time < self.min_time {
            process_time = self.min_time;
        }

        // Schedule completion
        cx.schedule_event(
            Duration::from_secs_f64(process_time * 60.0),
            Self::complete,
            part,
        ).unwrap();
    }

    async fn complete(&mut self, mut part: Telix1, cx: &mut Context<Self>) {
        self.busy = false;
        // Record completion time
        part.completion_time = Some((cx.time().as_secs() / 60).try_into().unwrap());  // Time in minutes
        self.output.send(part).await;
    }
}

impl Model for QualityControl {}

// Product source that generates new Telix1 products
pub struct ProductSource {
    pub output: Output<Telix1>,
    pub arrival_rate: f64,  // Mean arrivals per minute
    pub next_id: u64,
    pub rng: StdRng,
}

impl ProductSource {
    pub fn new(arrival_rate: f64, seed: u64) -> Self {
        Self {
            output: Output::default(),
            arrival_rate,
            next_id: 0,
            rng: StdRng::seed_from_u64(seed),
        }
    }
    
    // Public method that can be called by the scheduler
    pub async fn start_generation(&mut self, _: (), cx: &mut Context<Self>) {
        // Delegate to the private generate_product method
        self.generate_product((), cx).await;
    }

    fn schedule_next_arrival(&mut self, cx: &mut Context<Self>) {
        // Exponentially distributed inter-arrival times (Poisson process)
        let interval = -self.arrival_rate.recip() * self.rng.gen::<f64>().ln();

        cx.schedule_event(
            Duration::from_secs_f64(interval * 60.0),
            Self::generate_product,
            (),
        ).unwrap();
    }

    async fn generate_product(&mut self, _: (), cx: &mut Context<Self>) {
        // Create new product
        let product = Telix1 {
            id: self.next_id,
            entry_time: (cx.time().as_secs() / 60).try_into().unwrap(),  // Time in minutes
            completion_time: None,
        };
        self.next_id += 1;

        // Send to output
        self.output.send(product).await;

        // Schedule next arrival
        self.schedule_next_arrival(cx);
    }
}

impl Model for ProductSource {}

// Statistics collector for gathering performance metrics
pub struct StatisticsCollector {
    pub throughput_count: u64,
    pub cycle_times: Vec<f64>,
}

impl StatisticsCollector {
    pub fn new() -> Self {
        Self {
            throughput_count: 0,
            cycle_times: Vec::new(),
        }
    }

    pub async fn input(&mut self, part: Telix1) {
        if let Some(completion_time) = part.completion_time {
            let cycle_time = (completion_time as f64) - (part.entry_time as f64);
            self.cycle_times.push(cycle_time);
            self.throughput_count += 1;
        }
    }

    pub fn print_statistics(&self, total_simulation_time: f64) -> String {
        let mut result = String::from("\n=== Simulation Statistics ===\n");
        result.push_str(&format!("Total parts completed: {}\n", self.throughput_count));

        if !self.cycle_times.is_empty() {
            let avg_cycle_time: f64 = self.cycle_times.iter().sum::<f64>() / self.cycle_times.len() as f64;
            let max_cycle_time = self.cycle_times.iter().fold(f64::MIN, |a, &b| a.max(b));
            let min_cycle_time = self.cycle_times.iter().fold(f64::MAX, |a, &b| a.min(b));

            result.push_str(&format!("Average cycle time: {:.2} minutes\n", avg_cycle_time));
            result.push_str(&format!("Minimum cycle time: {:.2} minutes\n", min_cycle_time));
            result.push_str(&format!("Maximum cycle time: {:.2} minutes\n", max_cycle_time));

            // Calculate throughput per hour
            let throughput_per_hour = (self.throughput_count as f64 / total_simulation_time) * 60.0;
            result.push_str(&format!("Throughput rate: {:.2} parts per hour\n", throughput_per_hour));
        }

        result
    }
}

impl Model for StatisticsCollector {}
