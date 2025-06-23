// src/models.rs
use nexosim::model::{Context, Model};
use nexosim::ports::Output;
use rand::prelude::*;
use rand_distr::Uniform;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::time::Duration;

// The main person entity that enters and leaves the swimming pool
#[derive(Debug, Clone)]
pub struct Person {
    pub id: u64,
    pub arrival_time: u64,      // Time when person arrived
    pub entry_time: u64,        // Time when person entered the swimming pool
    pub exit_time: Option<u64>, // Time when person left the swimming pool
    pub wait_time: u64,         // Time spent waiting in queue
    pub swim_time: f64,         // Duration person will swim
}

impl Person {
    pub fn new(id: u64, arrival_time: u64, swim_time: f64) -> Self {
        Self {
            id,
            arrival_time,
            entry_time: 0,
            exit_time: None,
            wait_time: 0,
            swim_time,
        }
    }
}

// Person generator (creates new swimmers at random intervals)
pub struct PersonSource {
    pub output: Output<Person>,
    pub arrival_min: f64,
    pub arrival_max: f64,
    pub swim_time_min: f64,
    pub swim_time_max: f64,
    pub rng: StdRng,
    pub next_id: u64,
}

impl PersonSource {
    pub fn new(arrival_min: f64, arrival_max: f64, swim_time_min: f64, swim_time_max: f64, seed: u64) -> Self {
        Self {
            output: Output::default(),
            arrival_min,
            arrival_max,
            swim_time_min,
            swim_time_max,
            rng: StdRng::seed_from_u64(seed),
            next_id: 0,
        }
    }

    // Start generating people
    pub fn start_generation(&mut self, _: (), cx: &mut Context<Self>) {
        self.schedule_next_arrival(cx);
    }

    // Schedule the next person arrival
    fn schedule_next_arrival(&mut self, cx: &mut Context<Self>) {
        let arrival_dist = Uniform::new(self.arrival_min, self.arrival_max);
        let next_arrival_time = arrival_dist.sample(&mut self.rng);

        cx.schedule_event(
            Duration::from_secs_f64(next_arrival_time * 60.0),
            Self::generate_person,
            ()
        ).unwrap();
    }

    async fn generate_person(&mut self, _: (), cx: &mut Context<Self>) {
        // Generate swimming time for this person
        let swim_dist = Uniform::new(self.swim_time_min, self.swim_time_max);
        let swim_time = swim_dist.sample(&mut self.rng);

        // Create new person
        let current_time = cx.time().as_secs() as u64;
        let person = Person::new(self.next_id, current_time, swim_time);
        self.next_id += 1;

        // Send to swimming pool
        self.output.send(person).await;

        // Schedule next arrival
        self.schedule_next_arrival(cx);
    }
}

impl Model for PersonSource {}

// Swimming pool model
pub struct SwimmingPool {
    pub output: Output<Person>,
    pub max_capacity: u64,
    pub current_count: u64,
}

impl SwimmingPool {
    pub fn new(max_capacity: u64) -> Self {
        Self {
            output: Output::default(),
            max_capacity,
            current_count: 0,
        }
    }

    pub fn input(&mut self, mut person: Person, cx: &mut Context<Self>) {
        let current_time = cx.time().as_secs() as u64;

        // Calculate wait time
        person.wait_time = current_time.saturating_sub(person.arrival_time);
        person.entry_time = current_time;

        // Increment counter
        self.current_count += 1;
        
        println!("DEBUG: Person {} entered pool. Current count: {}/{}", 
            person.id, self.current_count, self.max_capacity);

        // Schedule person to leave after swim time
        cx.schedule_event(
            Duration::from_secs_f64(person.swim_time * 60.0),
            Self::person_exits,
            person
        ).unwrap();
    }

    async fn person_exits(&mut self, mut person: Person, cx: &mut Context<Self>) {
        // Set exit time and send to statistics
        let current_time = cx.time().as_secs() as u64;
        person.exit_time = Some(current_time);

        // Decrement counter
        self.current_count -= 1;
        
        println!("DEBUG: Person {} exited pool after {} minutes. Current count: {}/{}", 
            person.id, 
            (current_time - person.entry_time) as f64 / 60.0, 
            self.current_count, 
            self.max_capacity);

        // Send to statistics collector
        self.output.send(person).await;
    }
}

impl Model for SwimmingPool {}

// Waiting queue for when the pool is full
pub struct WaitingQueue {
    pub output: Output<Person>,
    pub pool_notification: Output<()>,
    pub queue: Vec<Person>,
}

impl WaitingQueue {
    pub fn new() -> Self {
        Self {
            output: Output::default(),
            pool_notification: Output::default(),
            queue: Vec::new(),
        }
    }

    pub async fn input(&mut self, person: Person) {
        // Add person to queue
        self.queue.push(person);
    }

    // Called when someone leaves the pool
    pub async fn pool_available(&mut self, _: ()) {
        // Send the next person to the pool if there's anyone waiting
        if !self.queue.is_empty() {
            let person = self.queue.remove(0);
            self.output.send(person).await;
        }
    }
}

impl Model for WaitingQueue {}

// Statistics collector
#[derive(Clone)]
pub struct StatisticsCollector {
    pub persons_processed: u64,
    pub total_wait_time: u64,
    pub total_swim_time: u64,
    pub max_queue_length: usize,
    pub max_wait_time: u64,
}

impl StatisticsCollector {
    pub fn new() -> Self {
        Self {
            persons_processed: 0,
            total_wait_time: 0,
            total_swim_time: 0,
            max_queue_length: 0,
            max_wait_time: 0,
        }
    }

    pub fn input(&mut self, person: Person, _: &mut Context<Self>) {
        // Process completed person
        self.persons_processed += 1;
        self.total_wait_time += person.wait_time;

        if let Some(exit_time) = person.exit_time {
            let actual_swim_time = exit_time - person.entry_time;
            self.total_swim_time += actual_swim_time;
        }

        self.max_wait_time = self.max_wait_time.max(person.wait_time);
    }

    pub fn update_queue_length(&mut self, length: usize) {
        self.max_queue_length = self.max_queue_length.max(length);
    }

    pub fn print_statistics(&self, simulation_time: f64) -> String {
        let avg_wait_time = if self.persons_processed > 0 {
            self.total_wait_time as f64 / self.persons_processed as f64 / 60.0 // in minutes
        } else {
            0.0
        };

        let avg_swim_time = if self.persons_processed > 0 {
            self.total_swim_time as f64 / self.persons_processed as f64 / 60.0 // in minutes
        } else {
            0.0
        };

        format!(
            "Swimming Pool Simulation Statistics:\n\
            - Total simulation time: {:.1} minutes\n\
            - People processed: {}\n\
            - Average wait time: {:.2} minutes\n\
            - Maximum wait time: {:.2} minutes\n\
            - Average swim time: {:.2} minutes\n\
            - Maximum queue length: {}",
            simulation_time,
            self.persons_processed,
            avg_wait_time,
            self.max_wait_time as f64 / 60.0,
            avg_swim_time,
            self.max_queue_length
        )
    }
}

impl Model for StatisticsCollector {}

// Pool controller that manages the swimming pool and waiting queue
pub struct PoolController {
    pub pool_output: Output<Person>,
    pub queue_output: Output<Person>,
    pub pool_notification: Output<()>, // New field for notifying when pool has space
    pub max_capacity: u64,
    pub current_count: u64,
}

impl PoolController {
    pub fn new(max_capacity: u64) -> Self {
        Self {
            pool_output: Output::default(),
            queue_output: Output::default(),
            pool_notification: Output::default(),
            max_capacity,
            current_count: 0,
        }
    }

    pub async fn input(&mut self, person: Person) {
        if self.current_count < self.max_capacity {
            // Pool has space - send directly to pool
            self.current_count += 1;
            println!("DEBUG: Controller - Sending person {} directly to pool. Current count: {}/{}", 
                person.id, self.current_count, self.max_capacity);
            self.pool_output.send(person).await;
        } else {
            // Pool is full - send to waiting queue
            println!("DEBUG: Controller - Pool full, sending person {} to queue. Current count: {}/{}", 
                person.id, self.current_count, self.max_capacity);
            self.queue_output.send(person).await;
        }
    }

    // When a person exits the pool
    pub async fn person_exited(&mut self, person: Person) {
        self.current_count -= 1;
        println!("DEBUG: Controller - Person {} exited, decremented count to {}/{}", 
            person.id, self.current_count, self.max_capacity);
        // Notify queue that space is available
        self.pool_notification.send(()).await;
    }
    
    pub fn person_notification(&mut self, _person: Person, _cx: &mut Context<Self>) {
        self.current_count -= 1;
    }

}

impl Model for PoolController {}