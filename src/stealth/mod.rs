pub mod patcher;
pub mod process;

pub use patcher::BinaryPatcher;
pub use process::{spawn_detached, ProcessRegistry};

