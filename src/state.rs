//! State transition types

use crate::error::AudiusError;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};
use std::mem::size_of;

/// Signer group data
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SignerGroup {
    /// Groups version
    pub version: u8,
    /// Pubkey of the account authorized to add/remove valid signers
    pub owner: Pubkey,
}

/// Valid signer data
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ValidSigner {
    /// Signer version
    pub version: u8,
    /// SignerGroup this ValidSigner belongs to
    pub signer_group: Pubkey,
    /// Ethereum public key used for signing messages
    pub public_key: [u8; 20],
}

/// Secp256k1 signature offsets data
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SecpSignatureOffsets {
    /// Offset of 64+1 bytes
    pub signature_offset: u16,
    /// Index of signature instruction in buffer
    pub signature_instruction_index: u8,
    /// Offset to eth_address of 20 bytes
    pub eth_address_offset: u16,
    /// Index of eth address instruction in buffer
    pub eth_address_instruction_index: u8,
    /// Offset to start of message data
    pub message_data_offset: u16,
    /// Size of message data
    pub message_data_size: u16,
    /// Index on message instruction in buffer
    pub message_instruction_index: u8,
}

impl SignerGroup {
    /// Length of SignerGroup when serialized
    pub const LEN: usize = size_of::<SignerGroup>();

    /// Deserialize a byte buffer into SignerGroup
    pub fn deserialize(input: &[u8]) -> Result<Self, ProgramError> {
        if input.len() < Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        #[allow(clippy::cast_ptr_alignment)]
        let signer_group: &SignerGroup =
            unsafe { &*(&input[0] as *const u8 as *const SignerGroup) };
        Ok(*signer_group)
    }

    /// Serialize a SignerGroup struct into byte buffer
    pub fn serialize(&self, output: &mut [u8]) -> ProgramResult {
        if output.len() < Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        #[allow(clippy::cast_ptr_alignment)]
        let value = unsafe { &mut *(&mut output[0] as *mut u8 as *mut SignerGroup) };
        *value = *self;
        Ok(())
    }

    /// Check if SignerGroup is initialized
    pub fn is_initialized(&self) -> bool {
        self.version != 0
    }

    /// Check owner validity and signature
    pub fn check_owner(&self, owner_info: &AccountInfo) -> Result<(), ProgramError> {
        if *owner_info.key != self.owner {
            return Err(AudiusError::WrongOwner.into());
        }
        if !owner_info.is_signer {
            return Err(AudiusError::SignatureMissing.into());
        }
        Ok(())
    }
}

impl ValidSigner {
    /// Length of ValidSigner when serialized
    pub const LEN: usize = size_of::<ValidSigner>();

    /// Deserialize a byte buffer into ValidSigner
    pub fn deserialize(input: &[u8]) -> Result<Self, ProgramError> {
        if input.len() < Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        #[allow(clippy::cast_ptr_alignment)]
        let valid_signer: &ValidSigner =
            unsafe { &*(&input[0] as *const u8 as *const ValidSigner) };
        Ok(valid_signer.clone())
    }

    /// Serialize a ValidSigner struct into byte buffer
    pub fn serialize(&self, output: &mut [u8]) -> ProgramResult {
        if output.len() < Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        #[allow(clippy::cast_ptr_alignment)]
        let value = unsafe { &mut *(&mut output[0] as *mut u8 as *mut ValidSigner) };
        *value = self.clone();
        Ok(())
    }

    /// Check if ValidSigner is initialized
    pub fn is_initialized(&self) -> bool {
        self.version != 0
    }
}

impl SecpSignatureOffsets {
    /// Max value can be hold in one byte
    pub const MAX_VALUE_ONE_BYTE: u16 = 256;

    /// Size of serialized Secp256k1 signature
    pub const SIGNATURE_OFFSETS_SERIALIZED_SIZE: usize = 11;

    /// Serialize [SecpSignatureOffsets]().
    pub fn pack(&self) -> Vec<u8> {
        let mut packed_offsets = vec![0u8; Self::SIGNATURE_OFFSETS_SERIALIZED_SIZE];

        self.euclidean_division(
            self.signature_offset,
            &mut packed_offsets,
            0 as usize,
            1 as usize,
        );

        packed_offsets[2] = self.signature_instruction_index;

        self.euclidean_division(
            self.eth_address_offset,
            &mut packed_offsets,
            3 as usize,
            4 as usize,
        );

        packed_offsets[5] = self.eth_address_instruction_index;

        self.euclidean_division(
            self.message_data_offset,
            &mut packed_offsets,
            6 as usize,
            7 as usize,
        );

        self.euclidean_division(
            self.message_data_size,
            &mut packed_offsets,
            8 as usize,
            9 as usize,
        );

        packed_offsets[10] = self.message_instruction_index;

        packed_offsets
    }

    fn euclidean_division(
        &self,
        dividend: u16,
        buffer: &mut Vec<u8>,
        first_index: usize,
        second_index: usize,
    ) {
        if dividend >= Self::MAX_VALUE_ONE_BYTE {
            let quotient: u8 = (dividend / Self::MAX_VALUE_ONE_BYTE) as u8;
            let remainder: u8 = (dividend % Self::MAX_VALUE_ONE_BYTE) as u8;
            buffer[first_index] = remainder;
            buffer[second_index] = quotient;
        } else {
            buffer[first_index] = dividend as u8;
            buffer[second_index] = 0;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_signer_group() {
        let signer_group = SignerGroup {
            version: 0,
            owner: Pubkey::new_from_array([1; 32]),
        };

        let mut buffer: [u8; SignerGroup::LEN] = [0; SignerGroup::LEN];
        signer_group.serialize(&mut buffer).unwrap();

        let deserialized: SignerGroup = SignerGroup::deserialize(&buffer).unwrap();

        assert_eq!(signer_group, deserialized);

        assert_eq!(signer_group.is_initialized(), false);
    }

    #[test]
    fn test_valid_signer() {
        let valid_signer = ValidSigner {
            version: 1,
            signer_group: Pubkey::new_from_array([1; 32]),
            public_key: [7; 20],
        };

        let mut buffer: [u8; ValidSigner::LEN] = [0; ValidSigner::LEN];
        valid_signer.serialize(&mut buffer).unwrap();

        let deserialized: ValidSigner = ValidSigner::deserialize(&buffer).unwrap();

        assert_eq!(valid_signer, deserialized);

        assert_eq!(valid_signer.is_initialized(), true);
    }
}
