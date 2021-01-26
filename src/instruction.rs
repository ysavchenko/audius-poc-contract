//! Instruction types

use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    program_option::COption,
    pubkey::Pubkey,
    sysvar,
};

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
    pub fn unpack(input: &[u8]) -> Result<(), ProgramError> {
        // TODO return Self
        Ok(())
    }

    /// Packs a [AudiusInstruction]() into a byte buffer.
    pub fn pack(&self) -> Vec<u8> {
        Vec::new()
    }
}
