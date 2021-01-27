//! State transition types

use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};
use std::mem::size_of;

/// Signer group data
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SignerGroup {
    /// Groups verstion
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
