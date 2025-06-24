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
}

impl SwimmingPool {
    pub fn new() -> Self {
        Self {
            output: Output::default(),
        }
    }

    pub fn input(&mut self, mut person: Person, cx: &mut Context<Self>) {
        let current_time = cx.time().as_secs() as u64;

        // Calculate wait time
        person.wait_time = current_time.saturating_sub(person.arrival_time);
        person.entry_time = current_time;

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

        self.output.send(person).await;
    }
}

impl Model for SwimmingPool {}

// Waiting queue for when the pool is full
pub struct WaitingQueue {
    pub output: Output<Person>,
    pub queue: Vec<Person>,
}

impl WaitingQueue {
    pub fn new() -> Self {
        Self {
            output: Output::default(),
            queue: Vec::new(),
        }
    }

    pub async fn input(&mut self, person: Person) {
        // Add person to queue
        let person_id = person.id;
        self.queue.push(person);
        println!("DEBUG: WaitingQueue - Person {} added to queue. Queue length: {}", person_id, self.queue.len());
    }

    // Called when someone leaves the pool
    pub async fn pool_available(&mut self, _: ()) {
        // Send the next person to the pool if there's anyone waiting
        if !self.queue.is_empty() {
            let person = self.queue.remove(0);
            println!("DEBUG: WaitingQueue - Person {} left queue. Queue length: {}", person.id, self.queue.len());
            self.output.send(person).await;
        }
    }
}

impl Model for WaitingQueue {}

// Pool controller that manages the swimming pool and waiting queue
pub struct PoolController {
    pub pool_output: Output<Person>,
    pub queue_output: Output<Person>,
    pub pool_notification: Output<()>,
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
            println!("DEBUG: Controller - Sending person {} to pool. Current count: {}/{}",
                person.id, self.current_count, self.max_capacity);
            self.pool_output.send(person).await;
        } else {
            // Pool is full - send to waiting queue
            println!("DEBUG: Controller - Pool full, sending person {} to queue. Current count: {}/{}",
                person.id, self.current_count, self.max_capacity);
            self.queue_output.send(person).await;
        }
    }

    // When a person exits the pool -> notify the queue that space is available
    pub async fn person_exited(&mut self, person: Person) {
        self.current_count -= 1;
        println!("DEBUG: Controller - Person {} exited, decremented count to {}/{}",
                 person.id, self.current_count, self.max_capacity);
        // Notify queue that space is available
        self.pool_notification.send(()).await;
    }
}

impl Model for PoolController {}