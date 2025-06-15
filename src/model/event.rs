use crate::model::part::Part;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Event {
    SMDFinished(Part),
    LotbadBatchFinished(Vec<Part>),
    MontageFinished(Part),
    QCFinished(Part),
    SimulationEnd,
}

use ordered_float::OrderedFloat;

pub struct EventQueue {
    events: std::collections::BinaryHeap<(std::cmp::Reverse<OrderedFloat<f64>>, Event)>,
}

impl EventQueue {
    pub fn new() -> Self {
        Self { events: std::collections::BinaryHeap::new() }
    }

    pub fn add_event(&mut self, time: f64, event: Event) {
        self.events.push((std::cmp::Reverse(OrderedFloat(time)), event));
    }

    pub fn next_event(&mut self) -> Option<(f64, Event)> {
        if let Some((std::cmp::Reverse(time), event)) = self.events.pop() {
            Some((time.into_inner(), event))
        } else {
            None
        }
    }
}