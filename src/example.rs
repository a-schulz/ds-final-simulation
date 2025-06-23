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

use std::time::Duration;
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Duration;
    use nexosim::ports::EventSlot;
    use nexosim::simulation::{Mailbox, SimInit};
    use nexosim::time::MonotonicTime;

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

    // The simulation is still at t0 so nothing is expected at the output of the
    // second delay gate.
    assert!(output_slot.next().is_none());

    // Advance simulation time until the next event and check the time and output.
    simu.step()?;
    assert_eq!(simu.time(), t0 + Duration::from_secs(1));
    assert_eq!(output_slot.next(), Some(84.0));

    // Get the answer to the ultimate question of life, the universe & everything.
    simu.step()?;
    assert_eq!(simu.time(), t0 + Duration::from_secs(2));
    assert_eq!(output_slot.next(), Some(42.0));

    Ok(())
}