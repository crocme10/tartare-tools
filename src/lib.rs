pub mod improve_stop_positions;
pub mod poi;
pub mod read_shapes;
pub mod report;
pub mod runner;

pub type Error = failure::Error;

/// The corresponding result type used by the crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;
