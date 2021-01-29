//! Instruction types

use crate::error::AudiusError;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use std::mem::size_of;

/// Signature with message to validate
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct Signature {
    /// Secp256k1 serialized signature
    pub signature: Vec<u8>,
    /// Ethereum signature recovery ID
    pub recovery_id: u8,
    /// Keccak256 message hash
    pub message: [u8; 32],
}

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
    ///
    ///   0. `[w]` Initialized valid signer to remove
    ///   1. `[]` Signer group to remove from
    ///   2. `[s]` SignerGroup's owner
    ClearValidSigner,
    ///   Validate signature issued by valid signer
    ///
    ///   0. `[]` Initialized valid signer
    ///   1. `[]` Signer group signer belongs to
    ValidateSignature(Signature),
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
            2 => Self::ClearValidSigner,
            3 => {
                let signature: &Signature = unpack_reference(rest)?;
                Self::ValidateSignature(signature.clone())
            }
            _ => return Err(AudiusError::InvalidInstruction.into()),
        })
    }

    /// Packs a [AudiusInstruction]() into a byte buffer.
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = vec![0u8; size_of::<AudiusInstruction>()];
        match self {
            Self::InitSignerGroup => buf[0] = 0,
            Self::InitValidSigner(eth_pubkey) => {
                buf[0] = 1;
                #[allow(clippy::cast_ptr_alignment)]
                let packed_pubkey = unsafe { &mut *(&mut buf[1] as *mut u8 as *mut [u8; 20]) };
                *packed_pubkey = *eth_pubkey;
            }
            Self::ClearValidSigner => buf[0] = 2,
            Self::ValidateSignature(signature) => {
                buf[0] = 3;
                #[allow(clippy::cast_ptr_alignment)]
                let packed_signature = unsafe { &mut *(&mut buf[1] as *mut u8 as *mut Signature) };
                *packed_signature = signature.clone();
            }
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
    let val: &T = unsafe { &*(&input[0] as *const u8 as *const T) };
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
        AccountMeta::new_readonly(*groups_owner, true),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

/// Creates `ClearValidSigner` instruction
pub fn clear_valid_signer(
    program_id: &Pubkey,
    valid_signer_account: &Pubkey,
    signer_group: &Pubkey,
    groups_owner: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new(*valid_signer_account, false),
        AccountMeta::new_readonly(*signer_group, false),
        AccountMeta::new_readonly(*groups_owner, true),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data: AudiusInstruction::ClearValidSigner.pack(),
    })
}

/// Creates `ValidateSignature` instruction
pub fn validate_signature(
    program_id: &Pubkey,
    valid_signer_account: &Pubkey,
    signer_group: &Pubkey,
    signature: Signature,
) -> Result<Instruction, ProgramError> {
    let args = AudiusInstruction::ValidateSignature(signature);
    let data = args.pack();

    let accounts = vec![
        AccountMeta::new_readonly(*valid_signer_account, false),
        AccountMeta::new_readonly(*signer_group, false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}
