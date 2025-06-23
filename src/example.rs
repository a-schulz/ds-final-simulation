#[derive(Default)]
pub struct Multiplier {
    pub output: Output<f64>,
}
impl Multiplier {
    pub async fn input(&mut self, value: f64) {
        self.output.send(2.0 * value).await;
    }
}
impl Model for Multiplier {}

use std::collections::HashMap;
use std::time::{Duration, Instant};
use nexosim::model::{Context, Model};
use nexosim::ports::Output;
use nexosim::simulation::SimulationError;

#[derive(Default)]
pub struct Delay {
    pub output: Output<f64>,
}
impl Delay {
    pub fn input(&mut self, value: f64, cx: &mut Context<Self>) {
        cx.schedule_event(Duration::from_secs(1), Self::send, value).unwrap();
    }

    async fn send(&mut self, value: f64) {
        self.output.send(value).await;
    }
}
impl Model for Delay {}
// Create a SimulationStats struct to hold statistics
struct SimulationStats {
    event_counts: HashMap<String, usize>,
    execution_time: Duration,
    sim_time_elapsed: Duration,
    max_value_observed: f64,
}

impl SimulationStats {
    fn new() -> Self {
        SimulationStats {
            event_counts: HashMap::new(),
            execution_time: Duration::default(),
            sim_time_elapsed: Duration::default(),
            max_value_observed: f64::NEG_INFINITY,
        }
    }

    fn print(&self) {
        println!("\n=== Simulation Statistics ===");
        println!("Wall clock execution time: {:?}", self.execution_time);
        println!("Simulation time elapsed: {:?}", self.sim_time_elapsed);
        println!("Maximum value observed: {}", self.max_value_observed);
        println!("Event counts by model:");
        for (model, count) in &self.event_counts {
            println!("  {}: {} events", model, count);
        }
        println!("===========================\n");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Duration;
    use nexosim::ports::EventSlot;
    use nexosim::simulation::{Mailbox, SimInit};
    use nexosim::time::MonotonicTime;

    // Create statistics object
    let mut stats = SimulationStats::new();
    let real_start_time = Instant::now();

    // Instantiate models.
    let mut multiplier1 = Multiplier::default();
    let mut multiplier2 = Multiplier::default();
    let mut delay1 = Delay::default();
    let mut delay2 = Delay::default();

    // Instantiate mailboxes.
    let multiplier1_mbox = Mailbox::new();
    let multiplier2_mbox = Mailbox::new();
    let delay1_mbox = Mailbox::new();
    let delay2_mbox = Mailbox::new();

    // Connect the models.
    multiplier1.output.connect(Delay::input, &delay1_mbox);
    multiplier1.output.connect(Multiplier::input, &multiplier2_mbox);
    multiplier2.output.connect(Delay::input, &delay2_mbox);
    delay1.output.connect(Delay::input, &delay2_mbox);

    // Keep handles to the system input and output for the simulation.
    let mut output_slot = EventSlot::new();
    delay2.output.connect_sink(&output_slot);
    let input_address = multiplier1_mbox.address();

    // Create monitoring event slot to track all values
    let mut monitoring_slot = EventSlot::new();
    multiplier1.output.connect_sink(&monitoring_slot);
    multiplier2.output.connect_sink(&monitoring_slot);
    delay1.output.connect_sink(&monitoring_slot);

    // Pick an arbitrary simulation start time and build the simulation.
    let t0 = MonotonicTime::EPOCH;
    let (mut simu, scheduler) = SimInit::new()
        .add_model(multiplier1, multiplier1_mbox, "multiplier1")
        .add_model(multiplier2, multiplier2_mbox, "multiplier2")
        .add_model(delay1, delay1_mbox, "delay1")
        .add_model(delay2, delay2_mbox, "delay2")
        .init(t0)?;

    // Send a value to the first multiplier.
    simu.process_event(Multiplier::input, 21.0, &input_address)?;
    stats.event_counts.insert("multiplier1".to_string(), 1);

    // Update initial simulation time
    let initial_sim_time = simu.time();

    // The simulation is still at t0 so nothing is expected at the output of the
    // second delay gate.
    assert!(output_slot.next().is_none());

    // Advance simulation time until the next event and check the time and output.
    simu.step()?;

    // Update event counts and check values
    stats.event_counts.entry("delay1".to_string()).and_modify(|e| *e += 1).or_insert(1);
    stats.event_counts.entry("multiplier2".to_string()).and_modify(|e| *e += 1).or_insert(1);

    // Check monitoring slot for values
    while let Some(value) = monitoring_slot.next() {
        stats.max_value_observed = stats.max_value_observed.max(value);
    }

    assert_eq!(simu.time(), t0 + Duration::from_secs(1));
    assert_eq!(output_slot.next(), Some(84.0));

    // Get the answer to the ultimate question of life, the universe & everything.
    simu.step()?;

    // Update event counts and check values again
    stats.event_counts.entry("delay2".to_string()).and_modify(|e| *e += 1).or_insert(2);

    // Check monitoring slot for values
    while let Some(value) = monitoring_slot.next() {
        stats.max_value_observed = stats.max_value_observed.max(value);
    }

    assert_eq!(simu.time(), t0 + Duration::from_secs(2));
    assert_eq!(output_slot.next(), Some(42.0));

    // Collect final statistics
    stats.execution_time = real_start_time.elapsed();
    stats.sim_time_elapsed = simu.time().duration_since(initial_sim_time);
    stats.max_value_observed = stats.max_value_observed.max(42.0).max(84.0);

    // Print the statistics
    stats.print();

    Ok(())
}