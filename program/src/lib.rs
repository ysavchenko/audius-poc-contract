#![deny(missing_docs)]

//! A program signature service for the Audius

pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

/// Current program version
pub const PROGRAM_VERSION: u8 = 1;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;
// 3QqhXLvBgPZ4DCV3YjyzpiQWfeR4Lf2bSKqSnj5c8wkE
solana_program::declare_id!("77xRWv8Z3kaQpD9K3Den7YWJ7sxsf1KTnw5MdcM7Gtnw");
