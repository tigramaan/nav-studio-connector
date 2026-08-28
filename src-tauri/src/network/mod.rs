mod discovery;
mod inspection;

pub use discovery::{discover_robots, DISCOVERY_SERVICE};
pub use inspection::{check_health, inspect_endpoint};
