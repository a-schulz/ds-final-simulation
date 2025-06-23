// src/main.rs
mod config;
mod models;

use std::time::{Duration, Instant};
use clap::Parser;
use nexosim::simulation::{Mailbox, SimInit};
use nexosim::time::MonotonicTime;

use crate::config::Config;
use crate::models::{
    PersonSource, SwimmingPool, WaitingQueue,
    StatisticsCollector, PoolController
};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Path to the configuration file
    #[clap(short, long, default_value = "config.toml")]
    config_file: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse();

    // Load configuration
    let config = Config::from_file(&args.config_file)?;
    println!("Loaded configuration from {}", args.config_file);

    // Initialize models
    let mut person_source = PersonSource::new(
        config.simulation.arrival_min,
        config.simulation.arrival_max,
        config.simulation.swim_time_min,
        config.simulation.swim_time_max,
        42 // seed
    );

    let mut swimming_pool = SwimmingPool::new(config.simulation.max_swimmers);
    let mut waiting_queue = WaitingQueue::new();
    let mut pool_controller = PoolController::new(config.simulation.max_swimmers);
    let mut statistics = StatisticsCollector::new();

    // Create mailboxes
    let source_mbox = Mailbox::new();
    let source_address = source_mbox.address();
    let pool_mbox = Mailbox::new();
    let queue_mbox = Mailbox::new();
    let controller_mbox = Mailbox::new();
    let stats_mbox = Mailbox::new();

    // Connect models
    // Person Source -> Pool Controller
    person_source.output.connect(PoolController::input, &controller_mbox);

    // Pool Controller -> Swimming Pool (when there's space)
    pool_controller.pool_output.connect(SwimmingPool::input, &pool_mbox);

    // Pool Controller -> Waiting Queue (when pool is full)
    pool_controller.queue_output.connect(WaitingQueue::input, &queue_mbox);

    // Pool Controller -> Waiting Queue notification (when space becomes available)
    pool_controller.pool_notification.connect(WaitingQueue::pool_available, &queue_mbox);

    // Swimming Pool -> Statistics (when person exits)
    swimming_pool.output.connect(StatisticsCollector::input, &stats_mbox);
    // Swimming Pool -> Pool Controller (when person exits)
    swimming_pool.output.connect(PoolController::person_exited, &controller_mbox);


    // Waiting Queue -> Swimming Pool (when space becomes available)
    waiting_queue.output.connect(SwimmingPool::input, &pool_mbox);

    // Initialize simulation
    let t0 = MonotonicTime::EPOCH;
    let mut sim_init = SimInit::new();

    // Keep a clone of the statistics object for reporting
    let stats_clone = statistics.clone();

    // Add models to simulation
    sim_init = sim_init.add_model(person_source, source_mbox, "person_source");
    sim_init = sim_init.add_model(swimming_pool, pool_mbox, "swimming_pool");
    sim_init = sim_init.add_model(waiting_queue, queue_mbox, "waiting_queue");
    sim_init = sim_init.add_model(pool_controller, controller_mbox, "pool_controller");
    sim_init = sim_init.add_model(statistics, stats_mbox, "statistics");

    let real_start_time = Instant::now();
    // Initialize and run simulation
    let (mut simulation, _scheduler) = sim_init.init(t0)?;

    // Start person generation
    simulation.process_event(PersonSource::start_generation, (), &source_address)?;

    println!("Starting simulation for {} minutes...", config.simulation.simulation_time);

    // Run simulation for the configured time
    simulation.step_until(t0 + Duration::from_secs_f64(config.simulation.simulation_time * 60.0))?;

    // Print statistics
    println!("Wall clock execution time: {:?}", real_start_time.elapsed());
    /*let stats_output = stats_clone.print_statistics(config.simulation.simulation_time);
    println!("{}", stats_output);*/
    println!("Simulation completed successfully");

    Ok(())
}