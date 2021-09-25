// TODO(#43): include these
#![allow(clippy::missing_panics_doc)]

// Macro-exporting modules
#[macro_use]
mod util;

// Public modules
pub mod load_file;

// Re-exports
pub use load_file::Warrior;
