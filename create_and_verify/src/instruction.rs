//! Instruction types

use crate::error::ProgramTemplateError;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar,
};

/// Instruction definition
#[repr(C)]
#[derive(Clone)]
pub enum TemplateInstruction {
    ///   Example
    ///
    ///   1. [] Valid signer account
    ///   2. [] Signer group
    ///   3. [] Audius program account
    ///   4. [] Sysvar instruction account
    ExampleInstruction(audius::instruction::SignatureData),
}
impl TemplateInstruction {
    /// Unpacks
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&tag, rest) = input
            .split_first()
            .ok_or(ProgramTemplateError::ExampleError)?;
        Ok(match tag {
            0 => {
                let mut signature: [u8; 64] = [0u8; 64];
                signature.copy_from_slice(&rest[0..64]);
                let signature_data = audius::instruction::SignatureData {
                    signature,
                    recovery_id: rest[64],
                    message: rest[64 + 1..].to_vec(),
                };
                Self::ExampleInstruction(signature_data)
            }
            _ => return Err(ProgramTemplateError::ExampleError.into()),
        })
    }

    /// Packs
    pub fn pack(&self) -> Vec<u8> {
        let mut buf;
        match self {
            Self::ExampleInstruction(signature_data) => {
                buf = vec![];
                buf.push(0);
                buf.extend_from_slice(&signature_data.signature);
                buf.push(signature_data.recovery_id);
                buf.extend_from_slice(&signature_data.message);
            }
        };
        buf
    }
}

/// Create `Example` instruction
pub fn init(
    program_id: &Pubkey,
    valid_signer_account: &Pubkey,
    signer_group: &Pubkey,
    signature_data: audius::instruction::SignatureData,
) -> Result<Instruction, ProgramError> {
    let init_data = TemplateInstruction::ExampleInstruction(signature_data);
    let data = init_data.pack();
    let accounts = vec![
        AccountMeta::new_readonly(*valid_signer_account, false),
        AccountMeta::new_readonly(*signer_group, false),
        AccountMeta::new_readonly(audius::id(), false),
        AccountMeta::new_readonly(sysvar::instructions::id(), false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}
