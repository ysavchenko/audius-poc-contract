//! Instruction types

use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    program_option::COption,
    pubkey::Pubkey,
    sysvar,
};
use std::mem::size_of;
use crate::error::AudiusError;

/// Instructions supported by the Audius program
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum AudiusInstruction {
    /// Create new signer group account
    InitSignerGroup,
    /// Create new valid signer account
    InitValidSigner,
    /// Remove valid signer from the group
    ClearValidSigner,
    /// Validate signature issued by valid signer
    ValidateSignature,
}
impl AudiusInstruction {
    /// Unpacks a byte buffer into a [AudiusInstruction]().
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&tag, rest) = input.split_first().ok_or(AudiusError::InvalidInstruction)?;
        Ok(match tag {
            0 => Self::InitSignerGroup,
            1 => Self::InitValidSigner,
            _ => return Err(AudiusError::InvalidInstruction.into()),
        })
    }

    /// Packs a [AudiusInstruction]() into a byte buffer.
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            Self::InitSignerGroup => buf.push(0),
            Self::InitValidSigner => buf.push(1),
            Self::ClearValidSigner => buf.push(2),
            Self::ValidateSignature => buf.push(3),  // TODO: add parameters
        };
        buf
    }
}
