//! Program state processor

use crate::error::AudiusError;
use crate::instruction::AudiusInstruction;
use crate::state::SignerGroup;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

/// Program state handler
pub struct Processor {}
impl Processor {

    /// SignerGroup version indicating group initialization
    pub const SIGNER_GROUP_VERSION: u8 = 1;

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
        ProgramTest::new("audius", id(), processor!(processor::Processor::process))
    }

    async fn create_signer_group_info_account(
        banks_client: &mut BanksClient,
        payer: &Keypair,
        recent_blockhash: &Hash,
        signer_group_acc: &Keypair,
    ) -> Result<(), TransportError> {
        let rent = banks_client.get_rent().await.unwrap();
        let signer_group_rent = rent.minimum_balance(state::SignerGroup::LEN);

        let mut transaction = Transaction::new_with_payer(
            &[system_instruction::create_account(
                &payer.pubkey(),
                &signer_group_acc.pubkey(),
                signer_group_rent,
                state::SignerGroup::LEN as u64,
                &id(),
            )],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[payer, signer_group_acc], *recent_blockhash);
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

        create_signer_group_info_account(
            &mut banks_client,
            &payer,
            &recent_blockhash,
            &signer_group,
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
}
