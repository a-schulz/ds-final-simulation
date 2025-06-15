enum Event {
    SMDFinished(Part),
    LotbadBatchFinished(Vec<Part>),
    MontageFinished(Part),
    QCFinished(Part),
    SimulationEnd,
}

struct EventQueue {
    events: Vec<(f64, Event)>,
}

impl EventQueue {
    fn new() -> Self {
        Self { events: Vec::new() }
    }

    fn schedule(&mut self, time: f64, event: Event) {
        let position = self.events.binary_search_by(|(t, _)| t.partial_cmp(&time).unwrap())
            .unwrap_or_else(|pos| pos);
        self.events.insert(position, (time, event));
    }

    fn next_event(&mut self) -> Option<(f64, Event)> {
        self.events.pop()
    }
}