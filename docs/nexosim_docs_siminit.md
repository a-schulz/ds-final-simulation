Struct SimInit
Settings
Help
Source

pub struct SimInit { /* private fields */ }

Builder for a multi-threaded, discrete-event simulation.
Implementations
Source
impl SimInit
Source
pub fn new() -> Self

Creates a builder for a multithreaded simulation running on all available logical threads.
Source
pub fn with_num_threads(num_threads: usize) -> Self

Creates a builder for a simulation running on the specified number of threads.

Note that the number of worker threads is automatically constrained to be between 1 and usize::BITS (inclusive). It is always set to 1 on wasm targets.
Source
pub fn add_model<P: ProtoModel>(
self,
model: P,
mailbox: Mailbox<P::Model>,
name: impl Into<String>,
) -> Self

Adds a model and its mailbox to the simulation bench.

The name argument needs not be unique. The use of the dot character in the name is possible but discouraged as it can cause confusion with the fully qualified name of a submodel. If an empty string is provided, it is replaced by the string <unknown>.
Source
pub fn set_clock(self, clock: impl Clock + 'static) -> Self

Synchronizes the simulation with the provided Clock.

If the clock isn’t explicitly set then the default NoClock is used, resulting in the simulation running as fast as possible.
Source
pub fn set_clock_tolerance(self, tolerance: Duration) -> Self

Specifies a tolerance for clock synchronization.

When a clock synchronization tolerance is set, then any report of synchronization loss by Clock::synchronize that exceeds the specified tolerance will trigger an ExecutionError::OutOfSync error.
Source
pub fn set_timeout(self, timeout: Duration) -> Self
Available on non-target_family="wasm" only.

Sets a timeout for the call to SimInit::init and for any subsequent simulation step.

The timeout corresponds to the maximum wall clock time allocated for the completion of a single simulation step before an ExecutionError::Timeout error is raised.

A null duration disables the timeout, which is the default behavior.

See also Simulation::set_timeout.
Source
pub fn init(
self,
start_time: MonotonicTime,
) -> Result<(Simulation, Scheduler), ExecutionError>

Builds a simulation initialized at the specified simulation time, executing the Model::init method on all model initializers.

The simulation object and its associated scheduler are returned upon success.
Trait Implementations
Source
impl Debug for SimInit
Source
fn fmt(&self, f: &mut Formatter<'_>) -> Result
Formats the value using the given formatter. Read more
Source
impl Default for SimInit
Source
fn default() -> Self
Returns the “default value” for a type. Read more
Auto Trait Implementations
impl Freeze for SimInit
impl !RefUnwindSafe for SimInit
impl Send for SimInit
impl !Sync for SimInit
impl Unpin for SimInit
impl !UnwindSafe for SimInit
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
impl<T> WithSubscriber for T
