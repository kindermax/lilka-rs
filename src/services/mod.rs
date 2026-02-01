pub mod clock;
pub mod network;

pub use clock::ClockService;
pub use network::{network_task, NetworkService};
