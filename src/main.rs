mod config;
mod models;

use std::time::Duration;
use clap::Parser;
use nexosim::simulation::{Address, Mailbox, SimInit};
use nexosim::time::MonotonicTime;

use crate::config::Config;
use crate::models::{
    SmdMachine, Buffer, LotBath, AssemblyStation, QualityControl,
    ProductSource, StatisticsCollector
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

    // Calculate lot bath process time from batches per hour
    let lot_bath_process_time = 60.0 / config.process_times.lot_bath_batches_per_hour as f64;
    
    // Initialize models
    let mut smd_machine = SmdMachine::new(config.process_times.smd_placement_time);
    
    let mut lot_bath_buffer = Buffer::new(config.buffer_capacities.pre_lot_bath_buffer);
    let mut lot_bath = LotBath::new(
        config.process_times.lot_bath_capacity,
        lot_bath_process_time
    );
    
    let mut assembly_buffer = Buffer::new(config.buffer_capacities.pre_assembly_buffer);
    let mut assembly_stations = Vec::new();
    for i in 0..config.resources.assembly_workstations {
        assembly_stations.push(AssemblyStation::new(
            config.process_times.assembly_time_min,
            config.process_times.assembly_time_max,
            config.simulation.seed + i as u64
        ));
    }
    
    let mut test_buffer = Buffer::new(config.buffer_capacities.pre_test_buffer);
    let mut test_stations = Vec::new();
    for i in 0..config.resources.test_stations {
        test_stations.push(QualityControl::new(
            config.process_times.quality_control_mean,
            config.process_times.quality_control_std_dev,
            config.process_times.quality_control_min_time,
            config.simulation.seed + config.resources.assembly_workstations as u64 + i as u64
        ));
    }
    
    let mut source = ProductSource::new(
        config.simulation.arrival_rate,
        config.simulation.seed
    );
    
    // let mut statistics = StatisticsCollector::new(config.simulation.warmup_period);


    // ###################################################
    // Connect models (assembling simulation benches)
    // ###################################################

    // Create mailboxes
    let source_mbox = Mailbox::new();
    let smd_mbox = Mailbox::new();
    let lot_bath_buffer_mbox = Mailbox::new();
    let lot_bath_mbox = Mailbox::new();
    let assembly_buffer_mbox = Mailbox::new();
    
    let mut assembly_mboxes = Vec::new();
    for _ in 0..assembly_stations.len() {
        assembly_mboxes.push(Mailbox::new());
    }
    
    let test_buffer_mbox = Mailbox::new();
    
    let mut test_mboxes = Vec::new();
    for _ in 0..test_stations.len() {
        test_mboxes.push(Mailbox::new());
    }
    
    let stats_mbox = Mailbox::new();
    
    // Connect models
    // Source -> SMD Machine
    source.output.connect(SmdMachine::input, &smd_mbox);
    
    // SMD Machine -> Lot Bath Buffer
    smd_machine.output.connect(Buffer::input, &lot_bath_buffer_mbox);
    
    // Lot Bath Buffer -> Lot Bath
    lot_bath_buffer.output.connect(LotBath::input, &lot_bath_mbox);
    
    // Lot Bath -> Assembly Buffer
    lot_bath.output.connect(Buffer::input, &assembly_buffer_mbox);
    
    // Assembly Buffer -> Assembly Stations (round-robin)
    for assembly_mbox in assembly_mboxes.iter() {
        assembly_buffer.output.connect(AssemblyStation::input, assembly_mbox);
    }
    
    // Assembly Stations -> Test Buffer
    for assembly_station in assembly_stations.iter_mut() {
        assembly_station.output.connect(Buffer::input, &test_buffer_mbox);
    }
    
    // Test Buffer -> Test Stations (round-robin)
    for test_mbox in test_mboxes.iter() {
        test_buffer.output.connect(QualityControl::input, test_mbox);
    }
    
    // Test Stations -> Statistics Collector
    for test_station in test_stations.iter_mut() {
        test_station.output.connect(StatisticsCollector::input, &stats_mbox);
    }
    
    // Create simulation
    let t0 = MonotonicTime::EPOCH;
    let mut sim_init = SimInit::new();
    
    // Add models to simulation and store addresses for later use
    // Note that add_model returns the modified SimInit, not an address
    sim_init = sim_init.add_model(source, source_mbox, "source");
    // Store the name for later use with scheduler
    let source_name = "source";

    sim_init = sim_init.add_model(smd_machine, smd_mbox, "smd_machine");
    sim_init = sim_init.add_model(lot_bath_buffer, lot_bath_buffer_mbox, "lot_bath_buffer");
    sim_init = sim_init.add_model(lot_bath, lot_bath_mbox, "lot_bath");
    sim_init = sim_init.add_model(assembly_buffer, assembly_buffer_mbox, "assembly_buffer");
    
    for (i, (assembly_station, assembly_mbox)) in assembly_stations.into_iter().zip(assembly_mboxes).enumerate() {
        sim_init = sim_init.add_model(assembly_station, assembly_mbox, &format!("assembly_station_{}", i));
    }
    
    sim_init = sim_init.add_model(test_buffer, test_buffer_mbox, "test_buffer");
    
    for (i, (test_station, test_mbox)) in test_stations.into_iter().zip(test_mboxes).enumerate() {
        sim_init = sim_init.add_model(test_station, test_mbox, &format!("test_station_{}", i));
    }
    
    // sim_init = sim_init.add_model(statistics, stats_mbox, "statistics");
    // Store the statistics model name for later
    // let stats_name = "statistics";

    // ###################################################
    // Running simulation
    // ###################################################

    // Initialize and run simulation
    let (mut simulation, scheduler) = sim_init.init(t0)?;
    
    // First step the simulation to trigger model initialization
    simulation.step()?;
    
    // Schedule first arrival manually using the scheduler
    let source_address = Address::from_name(source_name);
    scheduler.schedule_event(
        Duration::ZERO,
        ProductSource::start_generation,
        (),
        source_address
    )?;
    
    println!("Starting simulation for {} minutes...", config.simulation.simulation_time);
    
    // Run the simulation until the specified time
    simulation.step_until(t0 + Duration::from_secs(config.simulation.simulation_time * 60))?;
    
    // Print statistics
    // Access the statistics model using process_query
    // We'll construct an address from the name
    // let stats_address = Address::from_name(stats_name);

    // Process a query to the statistics model to get the data
    // let stats_result = simulation.process_query(
    //     StatisticsCollector::get_statistics,
    //     (),
    //     stats_address
    // )?;

    // Print statistics
    println!("Simulation statistics:");
    println!("{:?}", stats_result);

    println!("Simulation completed successfully");
    
    Ok(())
}
