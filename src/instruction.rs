//! Instruction types

use crate::error::AudiusError;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use std::mem::size_of;

/// Instructions supported by the Audius program
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum AudiusInstruction {
    ///   Create new signer group account
    ///
    ///   0. `[w]` New SignerGroup to create
    ///   1. `[]` SignerGroup's owner
    InitSignerGroup,
    ///   Create new valid signer account
    ///
    ///   0. `[w]` Uninitialized valid signer account
    ///   1. `[]` Group for Valid Signer to join with
    ///   2. `[s]` SignerGroup's owner
    InitValidSigner([u8; 20]),
    ///   Remove valid signer from the group
    ClearValidSigner,
    ///   Validate signature issued by valid signer
    ValidateSignature,
}
impl AudiusInstruction {
    /// Unpacks a byte buffer into a [AudiusInstruction]().
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&tag, rest) = input.split_first().ok_or(AudiusError::InvalidInstruction)?;
        Ok(match tag {
            0 => Self::InitSignerGroup,
            1 => {
                let eth_pubkey: &[u8; 20] = unpack_reference(rest)?;
                Self::InitValidSigner(*eth_pubkey)
            }
            _ => return Err(AudiusError::InvalidInstruction.into()),
        })
    }

    /// Packs a [AudiusInstruction]() into a byte buffer.
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            Self::InitSignerGroup => buf.push(0),
            Self::InitValidSigner(eth_pubkey) => {
                buf.push(1);
                let packed_pubkey = unsafe { &mut *(&mut buf[1] as *mut u8 as *mut [u8; 20]) };
                *packed_pubkey = *eth_pubkey;
            }
            Self::ClearValidSigner => buf.push(2),
            Self::ValidateSignature => buf.push(3), // TODO: add parameters
        };
        buf
    }
}

/// Unpacks a reference from a bytes buffer.
pub fn unpack_reference<T>(input: &[u8]) -> Result<&T, ProgramError> {
    if input.len() < size_of::<u8>() + size_of::<T>() {
        return Err(ProgramError::InvalidAccountData);
    }
    #[allow(clippy::cast_ptr_alignment)]
    let val: &T = unsafe { &*(&input[1] as *const u8 as *const T) };
    Ok(val)
}

/// Creates `InitSignerGroup` instruction
pub fn init_signer_group(
    program_id: &Pubkey,
    signer_group: &Pubkey,
    owner: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new(*signer_group, false),
        AccountMeta::new_readonly(*owner, false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data: AudiusInstruction::InitSignerGroup.pack(),
    })
}

/// Creates `InitValidSigner` instruction
pub fn init_valid_signer(
    program_id: &Pubkey,
    valid_signer_account: &Pubkey,
    signer_group: &Pubkey,
    groups_owner: &Pubkey,
    eth_pubkey: [u8; 20],
) -> Result<Instruction, ProgramError> {
    let args = AudiusInstruction::InitValidSigner(eth_pubkey);
    let data = args.pack();

    let accounts = vec![
        AccountMeta::new(*valid_signer_account, false),
        AccountMeta::new_readonly(*signer_group, false),
        AccountMeta::new(*groups_owner, true),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}
