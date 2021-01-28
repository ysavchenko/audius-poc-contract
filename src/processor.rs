//! Program state processor

use crate::error::AudiusError;
use crate::instruction::AudiusInstruction;
use crate::state::{SignerGroup, ValidSigner};
use num_traits::FromPrimitive;
use solana_program::decode_error::DecodeError;
use solana_program::program_error::PrintProgramError;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

/// Program state handler
pub struct Processor {}
impl Processor {
    /// SignerGroup version indicating group initialization
    pub const SIGNER_GROUP_VERSION: u8 = 1;

    /// ValidSigner version indicating signer initialization
    pub const VALID_SIGNER_VERSION: u8 = 1;

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

        signer_group.version = Self::SIGNER_GROUP_VERSION;
        signer_group.owner = *group_owner_info.key;

        signer_group.serialize(&mut signer_group_info.data.borrow_mut())?;
        Ok(())
    }

    /// Process [InitValidSigner]().
    pub fn process_init_valid_signer(
        accounts: &[AccountInfo],
        eth_pubkey: [u8; 20],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        // uninitialized valid signer account
        let valid_signer_info = next_account_info(account_info_iter)?;
        // signer group account
        let signer_group_info = next_account_info(account_info_iter)?;
        // signer group's owner
        let signer_groups_owner_info = next_account_info(account_info_iter)?;

        let signer_group = SignerGroup::deserialize(&signer_group_info.data.borrow())?;

        if !signer_group.is_initialized() {
            return Err(AudiusError::UninitializedSignerGroup.into());
        }

        let mut valid_signer = ValidSigner::deserialize(&valid_signer_info.data.borrow())?;

        if valid_signer.is_initialized() {
            return Err(AudiusError::SignerAlreadyInitialized.into());
        }

        signer_group.check_owner(&signer_groups_owner_info)?;

        // TODO: check if ethereum public key is valid

        valid_signer.version = Self::VALID_SIGNER_VERSION;
        valid_signer.signer_group = *signer_group_info.key;
        valid_signer.public_key = eth_pubkey;

        valid_signer.serialize(&mut valid_signer_info.data.borrow_mut())?;
        Ok(())
    }

    /// Process an [Instruction]().
    pub fn process(_program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = AudiusInstruction::unpack(input)?;

        match instruction {
            AudiusInstruction::InitSignerGroup => {
                msg!("Instruction: InitSignerGroup");
                Self::process_init_signer_group(accounts)
            }
            AudiusInstruction::InitValidSigner(eth_pubkey) => {
                msg!("Instruction: InitValidSigner");
                Self::process_init_valid_signer(accounts, eth_pubkey)
            }
            _ => Err(AudiusError::InvalidInstruction.into()), // TODO: remove when cover all the instructions
        }
    }
}

impl PrintProgramError for AudiusError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            AudiusError::InvalidInstruction => msg!("Invalid instruction"),
            AudiusError::SignerGroupAlreadyInitialized => msg!("Signer group already initialized"),
            AudiusError::UninitializedSignerGroup => msg!("Uninitialized signer group"),
            AudiusError::SignerAlreadyInitialized => msg!("Signer is already initialized"),
            AudiusError::WrongOwner => msg!("Wrong owner"),
            AudiusError::SignatureMissing => msg!("Signature missing"),
        }
    }
}
