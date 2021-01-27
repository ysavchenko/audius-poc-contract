//! Error types

use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

/// Errors that may be returned by the Audius program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum AudiusError {
    /// Test error
    #[error("TestError")]
    TestError,
}
impl From<AudiusError> for ProgramError {
    fn from(e: AudiusError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for AudiusError {
    fn type_of() -> &'static str {
        "Audius Error"
    }
}