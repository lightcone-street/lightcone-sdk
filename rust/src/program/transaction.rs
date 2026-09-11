//! Solana v1 compilation, canonical encoding, and signing.
//!
//! Resources and the blockhash are part of the signed message. Recompile and
//! obtain fresh signatures to change either. Legacy and v0 imports are rejected.

use super::error::{SdkError, SdkResult};
use solana_hash::Hash;
use solana_instruction::Instruction;
use solana_message::{v1, VersionedMessage};
use solana_pubkey::Pubkey;
use solana_signature::Signature;
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;

fn invalid(error: impl std::fmt::Display) -> SdkError {
    SdkError::InvalidTransaction(error.to_string())
}

/// Explicit limits encoded inline in the v1 message. No implicit compute budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct V1ResourceConfig {
    /// Maximum compute units the transaction may consume (1..=1,400,000).
    pub compute_unit_limit: u32,
    /// Maximum loaded account data, in bytes (1..=64 MiB).
    pub loaded_accounts_data_size_limit: u32,
    /// Total transaction priority fee in lamports, not micro-lamports per CU.
    pub priority_fee_lamports: u64,
    /// Heap bytes, in 1-KiB increments from 32 to 256 KiB; None uses 32 KiB.
    pub heap_size: Option<u32>,
}

impl V1ResourceConfig {
    /// Runtime compute-unit ceiling.
    pub const MAX_COMPUTE_UNITS: u32 = 1_400_000;
    /// Runtime loaded-account data ceiling, in bytes.
    pub const MAX_LOADED_ACCOUNT_BYTES: u32 = 64 * 1024 * 1024;

    /// Reject resource values outside the supported runtime limits.
    pub fn validate(&self) -> SdkResult<()> {
        if !(1..=Self::MAX_COMPUTE_UNITS).contains(&self.compute_unit_limit) {
            return Err(invalid("compute limit must be 1..=1,400,000"));
        }
        if !(1..=Self::MAX_LOADED_ACCOUNT_BYTES).contains(&self.loaded_accounts_data_size_limit) {
            return Err(invalid(
                "loaded account data limit must be 1..=67,108,864 bytes",
            ));
        }
        if let Some(bytes) = self.heap_size {
            if !(v1::MIN_HEAP_SIZE..=v1::MAX_HEAP_SIZE).contains(&bytes) || bytes % 1024 != 0 {
                return Err(invalid("heap must be 32..=256 KiB in 1-KiB increments"));
            }
        }
        Ok(())
    }

    fn config(self) -> SdkResult<v1::TransactionConfig> {
        self.validate()?;
        Ok(v1::TransactionConfig {
            compute_unit_limit: Some(self.compute_unit_limit),
            loaded_accounts_data_size_limit: Some(self.loaded_accounts_data_size_limit),
            priority_fee: (self.priority_fee_lamports != 0).then_some(self.priority_fee_lamports),
            heap_size: self.heap_size,
        })
    }
}

/// A blockhash and its expiry, together with caller-selected transaction resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct V1TransactionContext {
    /// Recent confirmed blockhash, encoded in the signed message.
    pub blockhash: Hash,
    /// Expiry returned by the same getLatestBlockhash result.
    pub last_valid_block_height: u64,
    /// Caller-selected limits and total priority fee.
    pub resources: V1ResourceConfig,
}

/// A validated v1 transaction bound to its original blockhash expiry.
///
/// Fields are private so callers cannot bypass validation or retain signatures
/// after changing the message. Unsigned transactions contain every signature slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V1Transaction {
    transaction: VersionedTransaction,
    context: V1TransactionContext,
}

impl V1Transaction {
    /// Maximum canonical transaction bytes, including every signature.
    pub const MAX_TRANSACTION_SIZE: usize = v1::MAX_TRANSACTION_SIZE;
    /// Maximum distinct inline account addresses.
    pub const MAX_ADDRESSES: usize = v1::MAX_ADDRESSES as usize;

    /// Compile offline using inline account keys and explicit resources.
    pub fn compile(
        instructions: &[Instruction],
        payer: &Pubkey,
        context: &V1TransactionContext,
    ) -> SdkResult<Self> {
        let message = v1::Message::try_compile_with_config(
            payer,
            instructions,
            context.blockhash,
            context.resources.config()?,
        )
        .map_err(invalid)?;
        let transaction = VersionedTransaction {
            signatures: vec![
                Signature::default();
                usize::from(message.header.num_required_signatures)
            ],
            message: VersionedMessage::V1(message),
        };
        Self::from_versioned(transaction, context)
    }

    /// Import only a v1 transaction whose resources and blockhash match the context.
    /// This permits unsigned transactions; submission additionally verifies signatures.
    pub fn from_versioned(
        transaction: VersionedTransaction,
        context: &V1TransactionContext,
    ) -> SdkResult<Self> {
        let VersionedMessage::V1(message) = &transaction.message else {
            return Err(invalid("only Solana v1 transactions are supported"));
        };
        if context.blockhash == Hash::default() || context.last_valid_block_height == 0 {
            return Err(invalid(
                "a recent blockhash and its last valid block height are required",
            ));
        }
        if message.lifetime_specifier != context.blockhash
            || message.config != context.resources.config()?
        {
            return Err(invalid(
                "message does not match its blockhash/resource context",
            ));
        }
        message.validate().map_err(invalid)?;
        transaction.sanitize().map_err(invalid)?;
        if message.instructions.is_empty() {
            return Err(invalid("transaction has no instructions"));
        }
        if message.instructions.iter().any(|ix| {
            message.account_keys[usize::from(ix.program_id_index)]
                == solana_sdk_ids::compute_budget::ID
        }) {
            return Err(invalid(
                "configure v1 resources inline; ComputeBudget instructions are unsupported",
            ));
        }
        let result = Self {
            transaction,
            context: *context,
        };
        result.to_wire_bytes()?;
        Ok(result)
    }

    /// Decode canonical v1 wire bytes, rejecting trailing data and oversized input.
    pub fn from_wire_bytes(bytes: &[u8], context: &V1TransactionContext) -> SdkResult<Self> {
        if bytes.len() > Self::MAX_TRANSACTION_SIZE {
            return Err(invalid("transaction exceeds the v1 wire size limit"));
        }
        if bytes.first() != Some(&v1::V1_PREFIX) {
            return Err(invalid("only Solana v1 transactions are supported"));
        }
        let transaction = wincode::deserialize(bytes).map_err(invalid)?;
        let result = Self::from_versioned(transaction, context)?;
        if result.to_wire_bytes()? != bytes {
            return Err(invalid("transaction encoding is not canonical"));
        }
        Ok(result)
    }

    /// Inspect the immutable v1 message used for fees and signatures.
    pub fn message(&self) -> &v1::Message {
        match &self.transaction.message {
            VersionedMessage::V1(message) => message,
            _ => unreachable!("V1Transaction construction enforces v1"),
        }
    }

    /// Return the original blockhash expiry and resource configuration.
    pub fn context(&self) -> &V1TransactionContext {
        &self.context
    }

    /// Read-only access for RPC integrations. Import responses through validation.
    pub fn as_versioned(&self) -> &VersionedTransaction {
        &self.transaction
    }

    /// Return distinct required signing keys in signature order, starting with the payer.
    pub fn required_signers(&self) -> &[Pubkey] {
        &self.message().account_keys[..usize::from(self.message().header.num_required_signatures)]
    }

    /// Canonical version-prefixed message bytes for signing and getFeeForMessage.
    pub fn message_bytes(&self) -> SdkResult<Vec<u8>> {
        wincode::serialize(&self.transaction.message).map_err(invalid)
    }

    /// Canonical transaction bytes including all required signature slots.
    pub fn to_wire_bytes(&self) -> SdkResult<Vec<u8>> {
        let bytes = wincode::serialize(&self.transaction).map_err(invalid)?;
        if bytes.len() > Self::MAX_TRANSACTION_SIZE {
            return Err(invalid(format!(
                "transaction is {} bytes; v1 maximum is {}",
                bytes.len(),
                Self::MAX_TRANSACTION_SIZE
            )));
        }
        Ok(bytes)
    }

    /// Sign each distinct required key once; caller order and duplicate inputs do not matter.
    pub fn sign(&self, signers: &[&dyn Signer]) -> SdkResult<Self> {
        let mut supplied = std::collections::HashMap::new();
        for signer in signers {
            supplied
                .entry(signer.try_pubkey().map_err(invalid)?)
                .or_insert(*signer);
        }
        let ordered: Vec<_> = self
            .required_signers()
            .iter()
            .map(|key| {
                supplied
                    .get(key)
                    .copied()
                    .ok_or_else(|| invalid(format!("missing signer {key}")))
            })
            .collect::<SdkResult<_>>()?;
        let transaction = VersionedTransaction::try_new(self.transaction.message.clone(), &ordered)
            .map_err(invalid)?;
        let signed = Self::from_versioned(transaction, &self.context)?;
        signed.verify_signatures()?;
        Ok(signed)
    }

    /// Verify every required signature over this exact version-prefixed message.
    pub fn verify_signatures(&self) -> SdkResult<()> {
        self.transaction
            .verify_and_hash_message()
            .map_err(invalid)?;
        Ok(())
    }

    /// Accept wallet signatures only if every byte of the prepared message survives.
    pub fn accept_signed_bytes(&self, bytes: &[u8]) -> SdkResult<Self> {
        let signed = Self::from_wire_bytes(bytes, &self.context)?;
        if signed.transaction.message != self.transaction.message {
            return Err(invalid("wallet changed the prepared transaction message"));
        }
        signed.verify_signatures()?;
        Ok(signed)
    }
}

#[cfg(test)]
pub(crate) fn test_context() -> V1TransactionContext {
    V1TransactionContext {
        blockhash: Hash::new_from_array([7; 32]),
        last_valid_block_height: 100,
        resources: V1ResourceConfig {
            compute_unit_limit: 200_000,
            loaded_accounts_data_size_limit: 1024 * 1024,
            priority_fee_lamports: 123,
            heap_size: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_instruction::AccountMeta;
    use solana_keypair::Keypair;

    fn transfer(payer: &Pubkey) -> Instruction {
        solana_system_interface::instruction::transfer(payer, &Pubkey::new_unique(), 42)
    }

    #[test]
    fn canonical_v1_round_trip_covers_inline_resources_and_every_signature() {
        let payer = Keypair::new();
        let context = test_context();
        let unsigned =
            V1Transaction::compile(&[transfer(&payer.pubkey())], &payer.pubkey(), &context)
                .unwrap();
        let signed = unsigned.sign(&[&payer]).unwrap();
        let bytes = signed.to_wire_bytes().unwrap();
        assert_eq!(bytes[0], v1::V1_PREFIX);
        assert_eq!(unsigned.to_wire_bytes().unwrap().len(), bytes.len());
        assert_eq!(signed.message().config.priority_fee, Some(123));
        assert_eq!(signed.message().config.compute_unit_limit, Some(200_000));
        assert_eq!(
            signed.message().config.loaded_accounts_data_size_limit,
            Some(1024 * 1024)
        );
        assert_eq!(
            V1Transaction::from_wire_bytes(&bytes, &context).unwrap(),
            signed
        );
        assert_eq!(unsigned.accept_signed_bytes(&bytes).unwrap(), signed);
        assert_eq!(
            signed.message_bytes().unwrap(),
            wincode::serialize(&signed.as_versioned().message).unwrap()
        );
        let mut trailing = bytes;
        trailing.push(0);
        assert!(V1Transaction::from_wire_bytes(&trailing, &context).is_err());
    }

    #[test]
    fn rejects_legacy_and_v0_at_both_import_boundaries() {
        let payer = Keypair::new();
        let context = test_context();
        let instructions = [transfer(&payer.pubkey())];
        let messages = [
            VersionedMessage::Legacy(solana_message::Message::new_with_blockhash(
                &instructions,
                Some(&payer.pubkey()),
                &context.blockhash,
            )),
            VersionedMessage::V0(
                solana_message::v0::Message::try_compile(
                    &payer.pubkey(),
                    &instructions,
                    &[],
                    context.blockhash,
                )
                .unwrap(),
            ),
        ];
        for message in messages {
            let tx = VersionedTransaction::try_new(message, &[&payer]).unwrap();
            assert!(V1Transaction::from_versioned(tx.clone(), &context).is_err());
            assert!(
                V1Transaction::from_wire_bytes(&wincode::serialize(&tx).unwrap(), &context)
                    .is_err()
            );
        }
    }

    #[test]
    fn enforces_resources_and_forbids_compute_budget_instructions() {
        let context = test_context();
        for invalid in [
            V1ResourceConfig {
                compute_unit_limit: 0,
                ..context.resources
            },
            V1ResourceConfig {
                compute_unit_limit: 1_400_001,
                ..context.resources
            },
            V1ResourceConfig {
                loaded_accounts_data_size_limit: 0,
                ..context.resources
            },
            V1ResourceConfig {
                loaded_accounts_data_size_limit: 67_108_865,
                ..context.resources
            },
            V1ResourceConfig {
                heap_size: Some(32_769),
                ..context.resources
            },
            V1ResourceConfig {
                heap_size: Some(16_384),
                ..context.resources
            },
        ] {
            assert!(invalid.validate().is_err());
        }
        let payer = Pubkey::new_unique();
        let budget = Instruction {
            program_id: solana_sdk_ids::compute_budget::ID,
            accounts: vec![],
            data: vec![2, 1, 0, 0, 0],
        };
        assert!(V1Transaction::compile(&[budget], &payer, &context).is_err());
        assert!(V1Transaction::compile(&[], &payer, &context).is_err());
        assert!(V1Transaction::compile(
            &[transfer(&payer)],
            &payer,
            &V1TransactionContext {
                blockhash: Hash::default(),
                ..context
            }
        )
        .is_err());
    }

    #[test]
    fn size_limit_includes_all_signature_slots() {
        let payer = Pubkey::new_unique();
        let cosigner = Pubkey::new_unique();
        let mut ix = Instruction {
            program_id: Pubkey::new_unique(),
            accounts: vec![AccountMeta::new_readonly(cosigner, true)],
            data: vec![],
        };
        let context = test_context();
        let overhead = V1Transaction::compile(&[ix.clone()], &payer, &context)
            .unwrap()
            .to_wire_bytes()
            .unwrap()
            .len();
        ix.data.resize(v1::MAX_TRANSACTION_SIZE - overhead, 0);
        let boundary = V1Transaction::compile(&[ix.clone()], &payer, &context).unwrap();
        assert_eq!(
            boundary.to_wire_bytes().unwrap().len(),
            v1::MAX_TRANSACTION_SIZE
        );
        assert_eq!(boundary.required_signers().len(), 2);
        ix.data.push(0);
        assert!(V1Transaction::compile(&[ix], &payer, &context).is_err());
    }

    #[test]
    fn address_limit_counts_distinct_keys_and_signers_are_deduplicated() {
        let payer = Keypair::new();
        let other = Keypair::new();
        let context = test_context();
        let mut ix = Instruction {
            program_id: Pubkey::new_unique(),
            accounts: vec![AccountMeta::new_readonly(other.pubkey(), true); 3],
            data: vec![],
        };
        let tx = V1Transaction::compile(&[ix.clone()], &payer.pubkey(), &context).unwrap();
        assert_eq!(tx.message().account_keys.len(), 3);
        assert_eq!(tx.message().instructions[0].accounts.len(), 3);
        assert!(tx.sign(&[&payer]).is_err());
        assert_eq!(
            tx.sign(&[&other, &payer, &other]).unwrap(),
            tx.sign(&[&payer, &other]).unwrap()
        );
        ix.accounts = (0..62)
            .map(|_| AccountMeta::new_readonly(Pubkey::new_unique(), false))
            .collect();
        assert!(V1Transaction::compile(&[ix.clone()], &payer.pubkey(), &context).is_ok());
        ix.accounts
            .push(AccountMeta::new_readonly(Pubkey::new_unique(), false));
        assert!(V1Transaction::compile(&[ix], &payer.pubkey(), &context).is_err());
    }

    #[test]
    fn wallet_must_preserve_message_and_supply_valid_signatures() {
        let payer = Keypair::new();
        let context = test_context();
        let tx = V1Transaction::compile(&[transfer(&payer.pubkey())], &payer.pubkey(), &context)
            .unwrap();
        assert!(tx
            .accept_signed_bytes(&tx.to_wire_bytes().unwrap())
            .is_err());
        for field in 0..5 {
            let mut changed = tx.as_versioned().clone();
            let VersionedMessage::V1(message) = &mut changed.message else {
                unreachable!()
            };
            match field {
                0 => message.lifetime_specifier = Hash::new_unique(),
                1 => message.config.priority_fee = Some(999),
                2 => message.instructions[0].data[4] ^= 1,
                3 => message.account_keys[1] = Pubkey::new_unique(),
                _ => message.config.compute_unit_limit = Some(100_000),
            }
            // Even a valid signature over the changed message cannot authorize a different plan.
            let signed = VersionedTransaction::try_new(changed.message, &[&payer]).unwrap();
            assert!(tx
                .accept_signed_bytes(&wincode::serialize(&signed).unwrap())
                .is_err());
        }
        let mut corrupt = tx.sign(&[&payer]).unwrap().to_wire_bytes().unwrap();
        *corrupt.last_mut().unwrap() ^= 1;
        assert!(tx.accept_signed_bytes(&corrupt).is_err());
    }
}
