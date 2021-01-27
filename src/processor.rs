//! Program state processor

use crate::error::AudiusError;
use crate::instruction::AudiusInstruction;
use crate::state::{SignerGroup, ValidSigner};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

/// Program state handler
pub struct Processor {}
impl Processor {
    /// Process [InitSignerGroup]().
    pub fn process_init_signer_group(accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        // signer group account
        let signer_group_info = next_account_info(account_info_iter)?;
        // signer group owner account
        let group_owner_info = next_account_info(account_info_iter)?;

        let mut signer_group = SignerGroup::deserialize(&signer_group_info.data.borrow())?;

        if signer_group.is_initialized() {
            return Err(AudiusError::SignerGroupAlreadyInitialized.into());
        }

        signer_group.version = 1;
        signer_group.owner = *group_owner_info.key;

        signer_group.serialize(&mut signer_group_info.data.borrow_mut())
    }

    /// Process an [Instruction]().
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = AudiusInstruction::unpack(input)?;

        match instruction {
            AudiusInstruction::InitSignerGroup => Self::process_init_signer_group(accounts),
            _ => Err(AudiusError::InvalidInstruction.into()), // TODO: remove when cover all the instructions
        }
    }
}
