//! Utility functions for the Lightcone Pinocchio SDK.
//!
//! This module provides helper functions for ATA derivation, validation, and string serialization.

use solana_pubkey::Pubkey;

use crate::program::constants::{MAX_OUTCOMES, MIN_OUTCOMES};
use crate::program::error::{SdkError, SdkResult};

// ============================================================================
// Associated Token Account Helpers
// ============================================================================

/// Get the Associated Token Address for a wallet and mint.
///
/// Uses the standard Solana ATA derivation.
pub fn get_associated_token_address(
    wallet: &Pubkey,
    mint: &Pubkey,
    token_program_id: &Pubkey,
) -> Pubkey {
    let ata_program_id = spl_associated_token_account::id();

    Pubkey::find_program_address(
        &[wallet.as_ref(), token_program_id.as_ref(), mint.as_ref()],
        &ata_program_id,
    )
    .0
}

/// Get the ATA for a conditional token (using SPL Token).
pub fn get_conditional_token_ata(wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
    get_associated_token_address(wallet, mint, &spl_token::id())
}

/// Get the ATA for a deposit token (using SPL Token).
pub fn get_deposit_token_ata(wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
    get_associated_token_address(wallet, mint, &spl_token::id())
}

// ============================================================================
// Validation Helpers
// ============================================================================

/// Validate that the number of outcomes is within the allowed range.
pub fn validate_outcome_count(num_outcomes: u8) -> SdkResult<()> {
    if !(MIN_OUTCOMES..=MAX_OUTCOMES).contains(&num_outcomes) {
        return Err(SdkError::InvalidOutcomeCount {
            count: num_outcomes,
        });
    }
    Ok(())
}

/// Validate that an outcome index is valid for the given number of outcomes.
pub fn validate_outcome_index(outcome_index: u8, num_outcomes: u8) -> SdkResult<()> {
    if outcome_index >= num_outcomes {
        return Err(SdkError::InvalidOutcomeIndex {
            index: outcome_index,
            max: num_outcomes.saturating_sub(1),
        });
    }
    Ok(())
}

/// Validate that a buffer is exactly 32 bytes.
pub fn validate_32_bytes(buffer: &[u8]) -> SdkResult<()> {
    if buffer.len() != 32 {
        return Err(SdkError::InvalidDataLength {
            expected: 32,
            actual: buffer.len(),
        });
    }
    Ok(())
}

/// Validate a maker/taker fee pair against program rules.
pub fn validate_fee_pair(maker_fee_bps: i16, taker_fee_bps: i16) -> SdkResult<()> {
    if !(-500..=500).contains(&maker_fee_bps) || !(-500..=500).contains(&taker_fee_bps) {
        return Err(SdkError::InvalidFeeRange);
    }
    if i32::from(maker_fee_bps) + i32::from(taker_fee_bps) < 0 {
        return Err(SdkError::InvalidFeeSum);
    }
    Ok(())
}

// ============================================================================
// String Serialization
// ============================================================================

/// Serialize a string with a u16 length prefix.
///
/// Format: [length (2 bytes LE)][utf-8 bytes]
pub fn serialize_string(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    let len = bytes.len() as u16;
    let mut result = Vec::with_capacity(2 + bytes.len());
    result.extend_from_slice(&len.to_le_bytes());
    result.extend_from_slice(bytes);
    result
}

/// Serialize a string with a u32 little-endian length prefix.
pub fn serialize_string_u32(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    let len = bytes.len() as u32;
    let mut result = Vec::with_capacity(4 + bytes.len());
    result.extend_from_slice(&len.to_le_bytes());
    result.extend_from_slice(bytes);
    result
}

/// Deserialize a string with a u16 length prefix.
///
/// Returns the string and the number of bytes consumed.
pub fn deserialize_string(data: &[u8]) -> SdkResult<(String, usize)> {
    if data.len() < 2 {
        return Err(SdkError::InvalidDataLength {
            expected: 2,
            actual: data.len(),
        });
    }

    let len = u16::from_le_bytes([data[0], data[1]]) as usize;

    if data.len() < 2 + len {
        return Err(SdkError::InvalidDataLength {
            expected: 2 + len,
            actual: data.len(),
        });
    }

    let s = String::from_utf8(data[2..2 + len].to_vec())
        .map_err(|e| SdkError::Serialization(e.to_string()))?;

    Ok((s, 2 + len))
}

/// Serialize conditional metadata instruction strings.
///
/// Format: `[name_len u32][name][symbol_len u32][symbol][uri_len u32][uri]`.
pub fn serialize_conditional_metadata(name: &str, symbol: &str, uri: &str) -> SdkResult<Vec<u8>> {
    validate_metadata_string("name", name, 32)?;
    validate_metadata_string("symbol", symbol, 10)?;
    validate_metadata_string("uri", uri, 200)?;

    let mut result = Vec::with_capacity(12 + name.len() + symbol.len() + uri.len());
    result.extend(serialize_string_u32(name));
    result.extend(serialize_string_u32(symbol));
    result.extend(serialize_string_u32(uri));
    Ok(result)
}

fn validate_metadata_string(field: &'static str, value: &str, max_bytes: usize) -> SdkResult<()> {
    if value.as_bytes().len() > max_bytes {
        return Err(SdkError::InvalidDataLength {
            expected: max_bytes,
            actual: value.as_bytes().len(),
        });
    }
    if value.as_bytes().len() > u32::MAX as usize {
        return Err(SdkError::Serialization(format!(
            "{field} length exceeds u32 bounds"
        )));
    }
    Ok(())
}

// ============================================================================
// Checked Arithmetic
// ============================================================================

/// Multiply two u64 values and check for overflow.
pub fn checked_mul_u64(a: u64, b: u64) -> SdkResult<u64> {
    a.checked_mul(b).ok_or(SdkError::Overflow)
}

/// Divide two u64 values and check for division by zero.
pub fn checked_div_u64(a: u64, b: u64) -> SdkResult<u64> {
    if b == 0 {
        return Err(SdkError::Overflow);
    }
    Ok(a / b)
}

/// Add two u64 values and check for overflow.
pub fn checked_add_u64(a: u64, b: u64) -> SdkResult<u64> {
    a.checked_add(b).ok_or(SdkError::Overflow)
}

/// Subtract two u64 values and check for underflow.
pub fn checked_sub_u64(a: u64, b: u64) -> SdkResult<u64> {
    a.checked_sub(b).ok_or(SdkError::Overflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_outcome_count() {
        assert!(validate_outcome_count(2).is_ok());
        assert!(validate_outcome_count(3).is_ok());
        assert!(validate_outcome_count(6).is_ok());
        assert!(validate_outcome_count(1).is_err());
        assert!(validate_outcome_count(7).is_err());
        assert!(validate_outcome_count(0).is_err());
    }

    #[test]
    fn test_validate_outcome_index() {
        assert!(validate_outcome_index(0, 3).is_ok());
        assert!(validate_outcome_index(1, 3).is_ok());
        assert!(validate_outcome_index(2, 3).is_ok());
        assert!(validate_outcome_index(3, 3).is_err());
        assert!(validate_outcome_index(4, 3).is_err());
    }

    #[test]
    fn test_validate_fee_pair() {
        assert!(validate_fee_pair(0, 0).is_ok());
        assert!(validate_fee_pair(-10, 25).is_ok());
        assert!(matches!(
            validate_fee_pair(501, 0),
            Err(SdkError::InvalidFeeRange)
        ));
        assert!(matches!(
            validate_fee_pair(-50, 25),
            Err(SdkError::InvalidFeeSum)
        ));
    }

    #[test]
    fn test_string_serialization_roundtrip() {
        let original = "Hello, World!";
        let serialized = serialize_string(original);
        let (deserialized, consumed) = deserialize_string(&serialized).unwrap();

        assert_eq!(original, deserialized);
        assert_eq!(consumed, serialized.len());
    }

    #[test]
    fn test_string_serialization_empty() {
        let original = "";
        let serialized = serialize_string(original);
        let (deserialized, consumed) = deserialize_string(&serialized).unwrap();

        assert_eq!(original, deserialized);
        assert_eq!(consumed, 2); // Just the length prefix
    }

    #[test]
    fn test_string_serialization_unicode() {
        let original = "Hello, 世界! 🌍";
        let serialized = serialize_string(original);
        let (deserialized, consumed) = deserialize_string(&serialized).unwrap();

        assert_eq!(original, deserialized);
        assert_eq!(consumed, serialized.len());
    }

    #[test]
    fn test_checked_arithmetic() {
        assert_eq!(checked_mul_u64(100, 200).unwrap(), 20000);
        assert_eq!(checked_div_u64(200, 100).unwrap(), 2);
        assert_eq!(checked_add_u64(100, 200).unwrap(), 300);
        assert_eq!(checked_sub_u64(200, 100).unwrap(), 100);

        // Overflow cases
        assert!(checked_mul_u64(u64::MAX, 2).is_err());
        assert!(checked_div_u64(100, 0).is_err());
        assert!(checked_add_u64(u64::MAX, 1).is_err());
        assert!(checked_sub_u64(0, 1).is_err());
    }

    #[test]
    fn test_ata_derivation() {
        let wallet = Pubkey::new_unique();
        let mint = Pubkey::new_unique();

        // Should not panic and should return a valid pubkey
        let ata = get_conditional_token_ata(&wallet, &mint);
        assert_ne!(ata, Pubkey::default());

        let ata2 = get_deposit_token_ata(&wallet, &mint);
        assert_ne!(ata2, Pubkey::default());

        // Conditional tokens and deposit tokens both use the SPL Token program.
        assert_eq!(ata, ata2);
    }
}
