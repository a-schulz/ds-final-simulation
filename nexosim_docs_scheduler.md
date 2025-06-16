Struct Scheduler
pub struct Scheduler(/* private fields */);

A global simulation scheduler.

A Scheduler can be Cloned and sent to other threads.
Implementations
Source
impl Scheduler
Source
pub fn time(&self) -> MonotonicTime

Returns the current simulation time.
Examples

use nexosim::simulation::Scheduler;
use nexosim::time::MonotonicTime;

fn is_third_millenium(scheduler: &Scheduler) -> bool {
let time = scheduler.time();
time >= MonotonicTime::new(978307200, 0).unwrap()
&& time < MonotonicTime::new(32535216000, 0).unwrap()
}

Source
pub fn schedule(
&self,
deadline: impl Deadline,
action: Action,
) -> Result<(), SchedulingError>

Schedules an action at a future time.

An error is returned if the specified time is not in the future of the current simulation time.

If multiple actions send events at the same simulation time to the same model, these events are guaranteed to be processed according to the scheduling order of the actions.
Source
pub fn schedule_event<M, F, T, S>(
&self,
deadline: impl Deadline,
func: F,
arg: T,
address: impl Into<Address<M>>,
) -> Result<(), SchedulingError>
where
M: Model,
F: for<'a> InputFn<'a, M, T, S>,
T: Send + Clone + 'static,
S: Send + 'static,

Schedules an event at a future time.

An error is returned if the specified time is not in the future of the current simulation time.

Events scheduled for the same time and targeting the same model are guaranteed to be processed according to the scheduling order.
Source
pub fn schedule_keyed_event<M, F, T, S>(
&self,
deadline: impl Deadline,
func: F,
arg: T,
address: impl Into<Address<M>>,
) -> Result<ActionKey, SchedulingError>
where
M: Model,
F: for<'a> InputFn<'a, M, T, S>,
T: Send + Clone + 'static,
S: Send + 'static,

Schedules a cancellable event at a future time and returns an event key.

An error is returned if the specified time is not in the future of the current simulation time.

Events scheduled for the same time and targeting the same model are guaranteed to be processed according to the scheduling order.
Source
pub fn schedule_periodic_event<M, F, T, S>(
&self,
deadline: impl Deadline,
period: Duration,
func: F,
arg: T,
address: impl Into<Address<M>>,
) -> Result<(), SchedulingError>
where
M: Model,
F: for<'a> InputFn<'a, M, T, S> + Clone,
T: Send + Clone + 'static,
S: Send + 'static,

Schedules a periodically recurring event at a future time.

An error is returned if the specified time is not in the future of the current simulation time or if the specified period is null.

Events scheduled for the same time and targeting the same model are guaranteed to be processed according to the scheduling order.
Source
pub fn schedule_keyed_periodic_event<M, F, T, S>(
&self,
deadline: impl Deadline,
period: Duration,
func: F,
arg: T,
address: impl Into<Address<M>>,
) -> Result<ActionKey, SchedulingError>
where
M: Model,
F: for<'a> InputFn<'a, M, T, S> + Clone,
T: Send + Clone + 'static,
S: Send + 'static,

Schedules a cancellable, periodically recurring event at a future time and returns an event key.

An error is returned if the specified time is not in the future of the current simulation time or if the specified period is null.

Events scheduled for the same time and targeting the same model are guaranteed to be processed according to the scheduling order.
Source
pub fn halt(&mut self)

Requests the simulation to be interrupted at the earliest opportunity.

If a multi-step method such as Simulation::step_until or Simulation::step_unbounded is concurrently being executed, this will cause such method to return before it steps to next scheduler deadline (if any) with ExecutionError::Halted.

Otherwise, this will cause the next call to a Simulation::step* or Simulation::process* method to return immediately with ExecutionError::Halted.

In all cases, once ExecutionError::Halted is returned, the halt flag is cleared and the simulation can be resumed at any moment with another call to one of the Simulation::step* or Simulation::process* methods.
Trait Implementations
Source
impl Clone for Scheduler
Source
fn clone(&self) -> Scheduler
Returns a copy of the value. Read more
1.0.0 · Source
fn clone_from(&mut self, source: &Self)
Performs copy-assignment from source. Read more
Source
impl Debug for Scheduler
Source
fn fmt(&self, f: &mut Formatter<'_>) -> Result
Formats the value using the given formatter. Read more
Auto Trait Implementations
impl Freeze for Scheduler
impl RefUnwindSafe for Scheduler
impl Send for Scheduler
impl Sync for Scheduler
impl Unpin for Scheduler
impl UnwindSafe for Scheduler
Blanket Implementations
Source
impl<T> Any for T
where
T: 'static + ?Sized,
Source
impl<T> Borrow<T> for T
where
T: ?Sized,
Source
impl<T> BorrowMut<T> for T
where
T: ?Sized,
Source
impl<T> CloneToUninit for T
where
T: Clone,
Source
impl<T> DynClone for T
where
T: Clone,
Source
impl<T> From<T> for T

Source
impl<T> FromRef<T> for T
where
T: Clone,
Source
impl<T> Instrument for T
Source
impl<T, U> Into<U> for T
where
U: From<T>,

Source
impl<T> IntoRequest<T> for T
Source
impl<T> ToOwned for T
where
T: Clone,
Source
impl<T, U> TryFrom<U> for T
where
U: Into<T>,
Source
impl<T, U> TryInto<U> for T
where
U: TryFrom<T>,
Source
§
impl<T> WithSubscriber for T