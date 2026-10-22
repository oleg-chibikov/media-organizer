//! Serializable payloads shared with the frontend (`src/lib/types.ts`).

mod destination;
mod gps;
mod history_operation;
mod history_run;
mod metadata;
mod move_execution;
mod places;
mod plan;
mod scan;

pub use destination::*;
pub use gps::*;
pub use history_operation::*;
pub use history_run::*;
pub use metadata::*;
pub use move_execution::*;
pub use places::*;
pub use plan::*;
pub use scan::*;
