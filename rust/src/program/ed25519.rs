//! Ed25519 signature verification instruction helpers.
//!
//! This module provides functions to create Ed25519 verification instructions
//! for order matching. Three strategies are supported:
//!
//! 1. **Individual Instructions** - One instruction per signature (simplest)
//! 2. **Batch Verification** - One instruction for multiple signatures (more efficient)
//! 3. **Cross-Instruction References** - References data in match instruction (most efficient)

use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
};

use crate::program::constants::ED25519_PROGRAM_ID;
use crate::program::orders::FullOrder;

// ============================================================================
// Ed25519 Verification Parameters
// ============================================================================

/// Parameters for creating an Ed25519 verification instruction.
#[derive(Debug, Clone)]
pub struct Ed25519VerifyParams {
    /// The public key that signed the message
    pub pubkey: Pubkey,
    /// The message that was signed (32-byte order hash)
    pub message: [u8; 32],
    /// The Ed25519 signature (64 bytes)
    pub signature: [u8; 64],
}

impl Ed25519VerifyParams {
    /// Create verify params from a full order.
    pub fn from_order(order: &FullOrder) -> Self {
        Self {
            pubkey: order.maker,
            message: order.hash(),
            signature: order.signature,
        }
    }
}

// ============================================================================
// Strategy 1: Individual Ed25519 Instructions
// ============================================================================

/// Create an Ed25519 signature verification instruction.
///
/// This instruction must precede the matchOrdersMulti instruction in the transaction.
/// The Solana Ed25519 program will verify the signature and the matchOrdersMulti
/// instruction will read the verification result from the instructions sysvar.
///
/// Ed25519 instruction data format (144 bytes):
/// - Header (16 bytes):
///   - num_signatures (u8): 1
///   - padding (u8): 0
///   - signature_offset (u16): 16
///   - signature_instruction_index (u16): 0xFFFF (same instruction)
///   - public_key_offset (u16): 80 (16 + 64)
///   - public_key_instruction_index (u16): 0xFFFF
///   - message_data_offset (u16): 112 (16 + 64 + 32)
///   - message_data_size (u16): 32
///   - message_instruction_index (u16): 0xFFFF
/// - Signature (64 bytes)
/// - Public Key (32 bytes)
/// - Message (32 bytes)
pub fn create_ed25519_verify_instruction(params: &Ed25519VerifyParams) -> Instruction {
    let mut data = vec![0u8; 144];

    // Header
    data[0] = 1; // num_signatures
    data[1] = 0; // padding

    // signature_offset (u16 LE) - starts at byte 16
    data[2..4].copy_from_slice(&16u16.to_le_bytes());
    // signature_instruction_index (u16 LE) - 0xFFFF = same instruction
    data[4..6].copy_from_slice(&0xFFFFu16.to_le_bytes());

    // public_key_offset (u16 LE) - starts at byte 80 (16 + 64)
    data[6..8].copy_from_slice(&80u16.to_le_bytes());
    // public_key_instruction_index (u16 LE)
    data[8..10].copy_from_slice(&0xFFFFu16.to_le_bytes());

    // message_data_offset (u16 LE) - starts at byte 112 (16 + 64 + 32)
    data[10..12].copy_from_slice(&112u16.to_le_bytes());
    // message_data_size (u16 LE)
    data[12..14].copy_from_slice(&32u16.to_le_bytes());
    // message_instruction_index (u16 LE)
    data[14..16].copy_from_slice(&0xFFFFu16.to_le_bytes());

    // Signature (64 bytes at offset 16)
    data[16..80].copy_from_slice(&params.signature);

    // Public Key (32 bytes at offset 80)
    data[80..112].copy_from_slice(params.pubkey.as_ref());

    // Message (32 bytes at offset 112)
    data[112..144].copy_from_slice(&params.message);

    Instruction {
        program_id: *ED25519_PROGRAM_ID,
        accounts: vec![],
        data,
    }
}

/// Create Ed25519 verification instructions for multiple signatures.
pub fn create_ed25519_verify_instructions(params: &[Ed25519VerifyParams]) -> Vec<Instruction> {
    params.iter().map(create_ed25519_verify_instruction).collect()
}

/// Create an Ed25519 verification instruction for an order.
pub fn create_order_verify_instruction(order: &FullOrder) -> Instruction {
    let params = Ed25519VerifyParams::from_order(order);
    create_ed25519_verify_instruction(&params)
}

// ============================================================================
// Strategy 2: Batch Ed25519 Verification
// ============================================================================

/// Create a single Ed25519 instruction that verifies multiple signatures.
///
/// This is more efficient than multiple single-signature instructions.
/// Each additional signature adds 14 bytes header + 64 sig + 32 pubkey + 32 msg = 142 bytes.
pub fn create_batch_ed25519_verify_instruction(params: &[Ed25519VerifyParams]) -> Instruction {
    assert!(!params.is_empty(), "At least one signature is required");

    let num_signatures = params.len();

    // Calculate header size: 2 bytes base + 14 bytes per signature
    let header_size = 2 + num_signatures * 14;

    // Data layout after header: [sig0, pubkey0, msg0, sig1, pubkey1, msg1, ...]
    // Each entry: 64 + 32 + 32 = 128 bytes
    let entry_size = 64 + 32 + 32;
    let total_size = header_size + num_signatures * entry_size;

    let mut data = vec![0u8; total_size];

    // num_signatures (u8)
    data[0] = num_signatures as u8;
    // padding (u8)
    data[1] = 0;

    // Per-signature header entries
    let mut header_offset = 2;
    for i in 0..num_signatures {
        let data_start = header_size + i * entry_size;

        // signature_offset (u16 LE)
        data[header_offset..header_offset + 2].copy_from_slice(&(data_start as u16).to_le_bytes());
        header_offset += 2;

        // signature_instruction_index (u16 LE)
        data[header_offset..header_offset + 2].copy_from_slice(&0xFFFFu16.to_le_bytes());
        header_offset += 2;

        // public_key_offset (u16 LE)
        data[header_offset..header_offset + 2]
            .copy_from_slice(&((data_start + 64) as u16).to_le_bytes());
        header_offset += 2;

        // public_key_instruction_index (u16 LE)
        data[header_offset..header_offset + 2].copy_from_slice(&0xFFFFu16.to_le_bytes());
        header_offset += 2;

        // message_data_offset (u16 LE)
        data[header_offset..header_offset + 2]
            .copy_from_slice(&((data_start + 64 + 32) as u16).to_le_bytes());
        header_offset += 2;

        // message_data_size (u16 LE)
        data[header_offset..header_offset + 2].copy_from_slice(&32u16.to_le_bytes());
        header_offset += 2;

        // message_instruction_index (u16 LE)
        data[header_offset..header_offset + 2].copy_from_slice(&0xFFFFu16.to_le_bytes());
        header_offset += 2;
    }

    // Data section
    for (i, p) in params.iter().enumerate() {
        let data_start = header_size + i * entry_size;
        data[data_start..data_start + 64].copy_from_slice(&p.signature);
        data[data_start + 64..data_start + 96].copy_from_slice(p.pubkey.as_ref());
        data[data_start + 96..data_start + 128].copy_from_slice(&p.message);
    }

    Instruction {
        program_id: *ED25519_PROGRAM_ID,
        accounts: vec![],
        data,
    }
}

// ============================================================================
// Strategy 3: Cross-Instruction Reference Ed25519 Verification
// ============================================================================

/// Match instruction data layout offsets (for cross-instruction references).
///
/// Layout for single maker (332 bytes):
/// - `[0]`:        discriminator (1 byte)
/// - `[1..33]`:    taker_hash (32 bytes)        <- taker message
/// - `[33..98]`:   taker_compact (65 bytes)     <- taker pubkey at offset 33+8=41
/// - `[98..162]`:  taker_signature (64 bytes)   <- taker signature
/// - `[162]`:      num_makers (1 byte)
/// - `[163..195]`: maker_hash (32 bytes)        <- maker message
/// - `[195..260]`: maker_compact (65 bytes)     <- maker pubkey at offset 195+8=203
/// - `[260..324]`: maker_signature (64 bytes)   <- maker signature
/// - `[324..332]`: maker_fill_amount (8 bytes)
pub struct MatchIxOffsets;

impl MatchIxOffsets {
    /// Taker hash (message) starts at offset 1
    pub const TAKER_MESSAGE: u16 = 1;
    /// Taker compact.maker (pubkey) at 33+8
    pub const TAKER_PUBKEY: u16 = 41;
    /// Taker signature at offset 98
    pub const TAKER_SIGNATURE: u16 = 98;
    /// num_makers at offset 162
    pub const NUM_MAKERS: u16 = 162;

    /// Calculate offsets for a maker order in the match instruction data.
    /// Each maker entry is: hash(32) + compact(65) + sig(64) + fill(8) = 169 bytes
    pub fn maker_offsets(maker_index: usize) -> MakerOffsets {
        let base = 163 + maker_index * 169;
        MakerOffsets {
            message: base as u16,
            pubkey: (base + 32 + 8) as u16,
            signature: (base + 32 + 65) as u16,
        }
    }
}

/// Offsets for a maker order within the match instruction data.
pub struct MakerOffsets {
    /// Offset of maker hash (message)
    pub message: u16,
    /// Offset of maker pubkey
    pub pubkey: u16,
    /// Offset of maker signature
    pub signature: u16,
}

/// Parameters for cross-instruction Ed25519 verification.
#[derive(Debug, Clone)]
pub struct CrossRefEd25519Params {
    pub signature_offset: u16,
    pub signature_ix_index: u16,
    pub pubkey_offset: u16,
    pub pubkey_ix_index: u16,
    pub message_offset: u16,
    pub message_size: u16,
    pub message_ix_index: u16,
}

/// Create a single Ed25519 instruction that references data in another instruction.
///
/// This is only 16 bytes (just header with offsets) - no embedded signature/pubkey/message data.
pub fn create_cross_ref_ed25519_instruction(params: &CrossRefEd25519Params) -> Instruction {
    let mut data = vec![0u8; 16];

    // num_signatures (u8)
    data[0] = 1;
    // padding (u8)
    data[1] = 0;

    // signature_offset (u16 LE)
    data[2..4].copy_from_slice(&params.signature_offset.to_le_bytes());
    // signature_instruction_index (u16 LE)
    data[4..6].copy_from_slice(&params.signature_ix_index.to_le_bytes());

    // public_key_offset (u16 LE)
    data[6..8].copy_from_slice(&params.pubkey_offset.to_le_bytes());
    // public_key_instruction_index (u16 LE)
    data[8..10].copy_from_slice(&params.pubkey_ix_index.to_le_bytes());

    // message_data_offset (u16 LE)
    data[10..12].copy_from_slice(&params.message_offset.to_le_bytes());
    // message_data_size (u16 LE)
    data[12..14].copy_from_slice(&params.message_size.to_le_bytes());
    // message_instruction_index (u16 LE)
    data[14..16].copy_from_slice(&params.message_ix_index.to_le_bytes());

    Instruction {
        program_id: *ED25519_PROGRAM_ID,
        accounts: vec![],
        data,
    }
}

/// Create Ed25519 verification instructions using cross-instruction references.
///
/// Instead of embedding signature/pubkey/message data in the Ed25519 instruction,
/// this points to offsets within the matchOrdersMulti instruction data.
///
/// This saves ~128 bytes per order (64 sig + 32 pubkey + 32 msg).
///
/// # Arguments
/// * `num_makers` - Number of maker orders
/// * `match_ix_index` - Index of the matchOrdersMulti instruction in the transaction
///
/// # Returns
/// Array of Ed25519 verification instructions (taker first, then makers)
pub fn create_cross_ref_ed25519_instructions(
    num_makers: usize,
    match_ix_index: u16,
) -> Vec<Instruction> {
    let mut instructions = Vec::with_capacity(1 + num_makers);

    // Taker verification instruction
    let taker_ix = create_cross_ref_ed25519_instruction(&CrossRefEd25519Params {
        signature_offset: MatchIxOffsets::TAKER_SIGNATURE,
        signature_ix_index: match_ix_index,
        pubkey_offset: MatchIxOffsets::TAKER_PUBKEY,
        pubkey_ix_index: match_ix_index,
        message_offset: MatchIxOffsets::TAKER_MESSAGE,
        message_size: 32,
        message_ix_index: match_ix_index,
    });
    instructions.push(taker_ix);

    // Maker verification instructions
    for i in 0..num_makers {
        let offsets = MatchIxOffsets::maker_offsets(i);
        let maker_ix = create_cross_ref_ed25519_instruction(&CrossRefEd25519Params {
            signature_offset: offsets.signature,
            signature_ix_index: match_ix_index,
            pubkey_offset: offsets.pubkey,
            pubkey_ix_index: match_ix_index,
            message_offset: offsets.message,
            message_size: 32,
            message_ix_index: match_ix_index,
        });
        instructions.push(maker_ix);
    }

    instructions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ed25519_verify_instruction_size() {
        let params = Ed25519VerifyParams {
            pubkey: Pubkey::new_unique(),
            message: [42u8; 32],
            signature: [0u8; 64],
        };

        let ix = create_ed25519_verify_instruction(&params);
        assert_eq!(ix.data.len(), 144);
        assert_eq!(ix.program_id, *ED25519_PROGRAM_ID);
        assert!(ix.accounts.is_empty());
    }

    #[test]
    fn test_batch_ed25519_verify_instruction_size() {
        let params = vec![
            Ed25519VerifyParams {
                pubkey: Pubkey::new_unique(),
                message: [1u8; 32],
                signature: [0u8; 64],
            },
            Ed25519VerifyParams {
                pubkey: Pubkey::new_unique(),
                message: [2u8; 32],
                signature: [0u8; 64],
            },
        ];

        let ix = create_batch_ed25519_verify_instruction(&params);

        // Header: 2 + 2*14 = 30 bytes
        // Data: 2 * 128 = 256 bytes
        // Total: 286 bytes
        assert_eq!(ix.data.len(), 30 + 256);
    }

    #[test]
    fn test_cross_ref_ed25519_instruction_size() {
        let params = CrossRefEd25519Params {
            signature_offset: 98,
            signature_ix_index: 2,
            pubkey_offset: 41,
            pubkey_ix_index: 2,
            message_offset: 1,
            message_size: 32,
            message_ix_index: 2,
        };

        let ix = create_cross_ref_ed25519_instruction(&params);
        assert_eq!(ix.data.len(), 16);
    }

    #[test]
    fn test_cross_ref_instructions_count() {
        let instructions = create_cross_ref_ed25519_instructions(2, 3);
        // 1 taker + 2 makers = 3 instructions
        assert_eq!(instructions.len(), 3);
    }

    #[test]
    fn test_maker_offsets() {
        // Maker 0
        let offsets0 = MatchIxOffsets::maker_offsets(0);
        assert_eq!(offsets0.message, 163);
        assert_eq!(offsets0.pubkey, 163 + 32 + 8);
        assert_eq!(offsets0.signature, 163 + 32 + 65);

        // Maker 1 (169 bytes later)
        let offsets1 = MatchIxOffsets::maker_offsets(1);
        assert_eq!(offsets1.message, 163 + 169);
    }
}
