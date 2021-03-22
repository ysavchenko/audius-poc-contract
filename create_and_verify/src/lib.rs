//! A minimal Solana program template
#![deny(missing_docs)]

pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

/// Current program version
pub const PROGRAM_VERSION: u8 = 1;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;

solana_program::declare_id!("HYRYHqZwCQG6Xn1V7HsW7rf6eHZP2HTM6zvtNSRZyyrB");
