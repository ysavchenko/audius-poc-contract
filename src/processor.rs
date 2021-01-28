//! Program state processor

use crate::error::AudiusError;
use crate::instruction::AudiusInstruction;
use crate::state::{SignerGroup, ValidSigner};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    program_error::PrintProgramError,
    msg,
    decode_error::DecodeError,
};
use num_traits::FromPrimitive;

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
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = AudiusInstruction::unpack(input)?;

        match instruction {
            AudiusInstruction::InitSignerGroup => Self::process_init_signer_group(accounts),
            AudiusInstruction::InitValidSigner(eth_pubkey) => {
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

#[cfg(test)]
mod test {
    use crate::*;
    use solana_program::{hash::Hash, pubkey::Pubkey, system_instruction};
    use solana_program_test::*;
    use solana_sdk::{
        account::Account,
        signature::{Keypair, Signer},
        transaction::Transaction,
        transport::TransportError,
    };

    pub fn program_test() -> ProgramTest {
        ProgramTest::new("audius", id(), processor!(processor::Processor::process),)
    }

    async fn create_account(
        banks_client: &mut BanksClient,
        payer: &Keypair,
        recent_blockhash: &Hash,
        account: &Keypair,
        struct_size: usize,
    ) -> Result<(), TransportError> {
        let rent = banks_client.get_rent().await.unwrap();
        let account_rent = rent.minimum_balance(struct_size);

        let mut transaction = Transaction::new_with_payer(
            &[system_instruction::create_account(
                &payer.pubkey(),
                &account.pubkey(),
                account_rent,
                struct_size as u64,
                &id(),
            )],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[payer, account], *recent_blockhash);
        banks_client.process_transaction(transaction).await?;
        Ok(())
    }

    async fn get_account(banks_client: &mut BanksClient, pubkey: &Pubkey) -> Account {
        banks_client
            .get_account(*pubkey)
            .await
            .expect("account not found")
            .expect("account empty")
    }

    #[tokio::test]
    async fn init_signer_group() {
        let (mut banks_client, payer, recent_blockhash) = program_test().start().await;

        let signer_group = Keypair::new();
        let group_owner = Keypair::new();

        create_account(
            &mut banks_client,
            &payer,
            &recent_blockhash,
            &signer_group,
            state::SignerGroup::LEN,
        )
        .await
        .unwrap();

        let mut transaction =
            Transaction::new_with_payer(
                &[instruction::init_signer_group(
                    &id(),
                    &signer_group.pubkey(),
                    &group_owner.pubkey(),
                )
                .unwrap()],
                Some(&payer.pubkey()),
            );
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        let signer_group_account = get_account(&mut banks_client, &signer_group.pubkey()).await;

        assert_eq!(signer_group_account.data.len(), state::SignerGroup::LEN);
        assert_eq!(signer_group_account.owner, id());

        let signer_group_data =
            state::SignerGroup::deserialize(&signer_group_account.data.as_slice()).unwrap();

        assert!(signer_group_data.is_initialized());
        assert_eq!(signer_group_data.owner, group_owner.pubkey());
    }

    #[tokio::test]
    async fn init_valid_signer() {
        let (mut banks_client, payer, recent_blockhash) = program_test().start().await;

        let signer_group = Keypair::new();
        let group_owner = Keypair::new();

        create_account(
            &mut banks_client,
            &payer,
            &recent_blockhash,
            &signer_group,
            state::SignerGroup::LEN,
        )
        .await
        .unwrap();

        let mut transaction =
            Transaction::new_with_payer(
                &[instruction::init_signer_group(
                    &id(),
                    &signer_group.pubkey(),
                    &group_owner.pubkey(),
                )
                .unwrap()],
                Some(&payer.pubkey()),
            );
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        let valid_signer = Keypair::new();

        create_account(&mut banks_client, &payer, &recent_blockhash, &valid_signer, state::ValidSigner::LEN)
            .await
            .unwrap();

        let eth_pub_key = [1u8; 20];
        let latest_blockhash = banks_client.get_recent_blockhash().await.unwrap();
        let mut transaction = Transaction::new_with_payer(
            &[instruction::init_valid_signer(
                &id(),
                &valid_signer.pubkey(),
                &signer_group.pubkey(),
                &group_owner.pubkey(),
                eth_pub_key,
            )
            .unwrap()],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer, &group_owner], latest_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();
    }
}
