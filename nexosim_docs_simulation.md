Struct Simulation
Settings
Help
Source

pub struct Simulation { /* private fields */ }

Simulation environment.

A Simulation is created by calling SimInit::init on a simulation initializer. It contains an asynchronous executor that runs all simulation models added beforehand to SimInit.

A Simulation object also manages an event scheduling queue and simulation time. The scheduling queue can be accessed from the simulation itself, but also from models via the optional &mut Context argument of input and replier port methods. Likewise, simulation time can be accessed with the Simulation::time method, or from models with the Context::time method.

Events and queries can be scheduled immediately, i.e. for the current simulation time, using process_event and send_query. Calling these methods will block until all computations triggered by such event or query have completed. In the case of queries, the response is returned.

Events can also be scheduled at a future simulation time using one of the schedule_* method. These methods queue an event without blocking.

Finally, the Simulation instance manages simulation time. A call to step will:

    increment simulation time until that of the next scheduled event in chronological order, then
    call Clock::synchronize which, unless the simulation is configured to run as fast as possible, blocks until the desired wall clock time, and finally
    run all computations scheduled for the new simulation time.

The step_until method operates similarly but iterates until the target simulation time has been reached.
Implementations
Source
impl Simulation
Source
pub fn set_timeout(&mut self, timeout: Duration)
Available on non-target_family="wasm" only.

Sets a timeout for each simulation step.

The timeout corresponds to the maximum wall clock time allocated for the completion of a single simulation step before an ExecutionError::Timeout error is raised.

A null duration disables the timeout, which is the default behavior.

See also SimInit::set_timeout.
Source
pub fn time(&self) -> MonotonicTime

Returns the current simulation time.
Source
pub fn reset_clock(&mut self, clock: impl Clock + 'static)

Reinitializes the simulation clock.

This can in particular be used to resume a simulation driven by a real-time clock after it was halted, using a new clock with an update time reference.
Source
pub fn step(&mut self) -> Result<(), ExecutionError>

Advances simulation time to that of the next scheduled event, processing that event as well as all other events scheduled for the same time.

Processing is gated by a (possibly blocking) call to Clock::synchronize on the configured simulation clock. This method blocks until all newly processed events have completed.
Source
pub fn step_until(
&mut self,
deadline: impl Deadline,
) -> Result<(), ExecutionError>

Iteratively advances the simulation time until the specified deadline, as if by calling Simulation::step repeatedly.

This method blocks until all events scheduled up to the specified target time have completed. The simulation time upon completion is equal to the specified target time, whether or not an event was scheduled for that time.
Source
pub fn step_unbounded(&mut self) -> Result<(), ExecutionError>

Iteratively advances the simulation time, as if by calling Simulation::step repeatedly.

This method blocks until the simulation is halted or all scheduled events have completed.
Source
pub fn process(&mut self, action: Action) -> Result<(), ExecutionError>

Processes an action immediately, blocking until completion.

Simulation time remains unchanged. The periodicity of the action, if any, is ignored.
Source
pub fn process_event<M, F, T, S>(
&mut self,
func: F,
arg: T,
address: impl Into<Address<M>>,
) -> Result<(), ExecutionError>
where
M: Model,
F: for<'a> InputFn<'a, M, T, S>,
T: Send + Clone + 'static,

Processes an event immediately, blocking until completion.

Simulation time remains unchanged.
Source
pub fn process_query<M, F, T, R, S>(
&mut self,
func: F,
arg: T,
address: impl Into<Address<M>>,
) -> Result<R, ExecutionError>
where
M: Model,
F: for<'a> ReplierFn<'a, M, T, R, S>,
T: Send + Clone + 'static,
R: Send + 'static,

Processes a query immediately, blocking until completion.

Simulation time remains unchanged. If the mailbox targeted by the query was not found in the simulation, an ExecutionError::BadQuery is returned.
Trait Implementations
Source
impl Debug for Simulation
Source
fn fmt(&self, f: &mut Formatter<'_>) -> Result
Formats the value using the given formatter. Read more
Auto Trait Implementations
impl Freeze for Simulation
impl !RefUnwindSafe for Simulation
impl Send for Simulation
impl !Sync for Simulation
impl Unpin for Simulation
impl !UnwindSafe for Simulation
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
impl<T> From<T> for T

Source
impl<T> Instrument for T
Source
impl<T, U> Into<U> for T
where
U: From<T>,

Source
impl<T> IntoRequest<T> for T
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