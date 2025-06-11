/// BitcoinZ Overwinter Transaction Builder
/// 
/// This module builds Overwinter (v3) transactions for transparent-only transfers
/// to bypass the Sapling binding signature issue.

use blake2b_simd::Params;
use byteorder::{LittleEndian, WriteBytesExt};
use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
use std::io::Write;
use zcash_primitives::{
    consensus::{BlockHeight, BranchId, Parameters},
    legacy::{Script, TransparentAddress},
    transaction::{
        components::{Amount, OutPoint, TxOut},
    },
};

/// BitcoinZ Overwinter constants
const OVERWINTER_VERSION_GROUP_ID: u32 = 0x03C48270;  // Zcash Overwinter
const BITCOINZ_VERSION_GROUP_ID: u32 = 0x892f2085;    // BitcoinZ uses same for v3 and v4
const OVERWINTER_TX_VERSION: i32 = 3;
const SIGHASH_ALL: u32 = 1;

/// Personalization for BLAKE2b hashing in BitcoinZ
const ZCASH_PREVOUTS_HASH_PERSONALIZATION: &[u8; 16] = b"ZcashPrevoutHash";
const ZCASH_SEQUENCE_HASH_PERSONALIZATION: &[u8; 16] = b"ZcashSequencHash";
const ZCASH_OUTPUTS_HASH_PERSONALIZATION: &[u8; 16] = b"ZcashOutputsHash";
const ZCASH_SIGHASH_PERSONALIZATION_PREFIX: &[u8; 12] = b"ZcashSigHash";


/// Build a raw Overwinter transaction for BitcoinZ transparent-only transfers
pub fn build_overwinter_tx<P: Parameters>(
    params: &P,
    inputs: Vec<(OutPoint, TxOut, SecretKey)>,
    outputs: Vec<(TransparentAddress, Amount)>,
    height: BlockHeight,
) -> Result<Vec<u8>, String> {
    
    // First, build the unsigned transaction
    let unsigned_tx = build_unsigned_overwinter_tx(&inputs, &outputs, height)?;
    
    // Then sign all inputs
    let signed_tx = sign_overwinter_transaction(params, unsigned_tx, inputs, &outputs, height)?;
    
    
    Ok(signed_tx)
}

/// Build unsigned Overwinter transaction
fn build_unsigned_overwinter_tx(
    inputs: &[(OutPoint, TxOut, SecretKey)],
    outputs: &[(TransparentAddress, Amount)],
    height: BlockHeight,
) -> Result<Vec<u8>, String> {
    let mut tx_data = Vec::new();
    
    // Header (version with overwinter flag)
    let header = 0x80000000u32 | (OVERWINTER_TX_VERSION as u32);
    tx_data.write_u32::<LittleEndian>(header)
        .map_err(|e| format!("Failed to write header: {}", e))?;
    
    // Version group ID (BitcoinZ uses same ID for v3 and v4)
    tx_data.write_u32::<LittleEndian>(BITCOINZ_VERSION_GROUP_ID)
        .map_err(|e| format!("Failed to write version group ID: {}", e))?;
    
    // Input count
    write_compact_size(&mut tx_data, inputs.len() as u64)?;
    
    // Write inputs (with empty script sigs for now)
    for (outpoint, _, _) in inputs {
        // Previous output hash (32 bytes)
        tx_data.write_all(outpoint.hash())
            .map_err(|e| format!("Failed to write outpoint hash: {}", e))?;
        
        // Previous output index (4 bytes)
        tx_data.write_u32::<LittleEndian>(outpoint.n())
            .map_err(|e| format!("Failed to write outpoint index: {}", e))?;
        
        // Script sig placeholder (empty for unsigned)
        write_compact_size(&mut tx_data, 0)?;
        
        // Sequence
        tx_data.write_u32::<LittleEndian>(0xfffffffe)
            .map_err(|e| format!("Failed to write sequence: {}", e))?;
    }
    
    // Output count
    write_compact_size(&mut tx_data, outputs.len() as u64)?;
    
    // Write outputs
    for (addr, amount) in outputs {
        // Amount (8 bytes)
        tx_data.write_u64::<LittleEndian>(u64::from(*amount))
            .map_err(|e| format!("Failed to write amount: {}", e))?;
        
        // Script pubkey
        let script = addr.script();
        write_compact_size(&mut tx_data, script.0.len() as u64)?;
        tx_data.write_all(&script.0)
            .map_err(|e| format!("Failed to write script: {}", e))?;
    }
    
    // Lock time
    tx_data.write_u32::<LittleEndian>(0)
        .map_err(|e| format!("Failed to write lock time: {}", e))?;
    
    // Expiry height (Overwinter addition)
    let expiry_height = height + BlockHeight::from(10);
    tx_data.write_u32::<LittleEndian>(u32::from(expiry_height))
        .map_err(|e| format!("Failed to write expiry height: {}", e))?;
    
    // Note: BitcoinZ v3 doesn't include joinSplits in the unsigned transaction
    // They're added during signing
    
    Ok(tx_data)
}

/// Sign the Overwinter transaction
fn sign_overwinter_transaction<P: Parameters>(
    params: &P,
    unsigned_tx: Vec<u8>,
    inputs: Vec<(OutPoint, TxOut, SecretKey)>,
    outputs: &[(TransparentAddress, Amount)],
    height: BlockHeight,
) -> Result<Vec<u8>, String> {
    let secp = Secp256k1::new();
    let mut signatures = Vec::new();
    
    // Compute signatures for each input
    for (index, (_, txout, sk)) in inputs.iter().enumerate() {
        // Compute the sighash for this input
        let sighash = compute_overwinter_sighash(
            &unsigned_tx,
            &inputs,
            &outputs,
            index,
            &txout.script_pubkey,
            txout.value,
            SIGHASH_ALL,
            params,
            height,
        )?;
        
        // Sign the sighash
        let msg = Message::from_slice(&sighash)
            .map_err(|e| format!("Failed to create message: {}", e))?;
        let sig = secp.sign_ecdsa(&msg, &sk);
        
        // Create script sig
        let pk = PublicKey::from_secret_key(&secp, &sk);
        let mut script_sig = Vec::new();
        
        // Push signature with sighash type
        let mut sig_bytes = sig.serialize_der().to_vec();
        sig_bytes.push(SIGHASH_ALL as u8);
        script_sig.push(sig_bytes.len() as u8);
        script_sig.extend_from_slice(&sig_bytes);
        
        // Push public key
        let pk_bytes = pk.serialize();
        script_sig.push(pk_bytes.len() as u8);
        script_sig.extend_from_slice(&pk_bytes);
        
        signatures.push(script_sig);
    }
    
    // Now rebuild the transaction with signatures
    let mut signed_tx = Vec::new();
    
    // Copy header and version group ID (first 8 bytes)
    signed_tx.extend_from_slice(&unsigned_tx[0..8]);
    
    // Write inputs with signatures
    write_compact_size(&mut signed_tx, inputs.len() as u64)?;
    
    for (i, (outpoint, _, _)) in inputs.iter().enumerate() {
        // Previous output
        signed_tx.write_all(outpoint.hash())
            .map_err(|e| format!("Failed to write outpoint hash: {}", e))?;
        signed_tx.write_u32::<LittleEndian>(outpoint.n())
            .map_err(|e| format!("Failed to write outpoint index: {}", e))?;
        
        // Script sig with signature
        write_compact_size(&mut signed_tx, signatures[i].len() as u64)?;
        signed_tx.write_all(&signatures[i])
            .map_err(|e| format!("Failed to write script sig: {}", e))?;
        
        // Sequence
        signed_tx.write_u32::<LittleEndian>(0xfffffffe)
            .map_err(|e| format!("Failed to write sequence: {}", e))?;
    }
    
    // Find where outputs start in the unsigned transaction
    let mut cursor = 8; // After header and version group
    let (input_count, varint_size) = read_compact_size(&unsigned_tx[cursor..])?
        .ok_or("Failed to read input count")?;
    cursor += varint_size;
    
    // Skip all inputs in unsigned tx
    for _ in 0..input_count {
        cursor += 32 + 4; // outpoint
        let (script_len, varint_size) = read_compact_size(&unsigned_tx[cursor..])?
            .ok_or("Failed to read script length")?;
        cursor += varint_size + script_len as usize;
        cursor += 4; // sequence
    }
    
    // Copy the rest (outputs, locktime, expiry)
    signed_tx.extend_from_slice(&unsigned_tx[cursor..]);
    
    // Add joinsplit count (0 for transparent-only)
    write_compact_size(&mut signed_tx, 0)?;
    
    Ok(signed_tx)
}

/// Compute Overwinter sighash using BLAKE2b
fn compute_overwinter_sighash<P: Parameters>(
    _tx_data: &[u8],
    inputs: &[(OutPoint, TxOut, SecretKey)],
    outputs: &[(TransparentAddress, Amount)],
    input_index: usize,
    script_code: &Script,
    value: Amount,
    sighash_type: u32,
    params: &P,
    height: BlockHeight,
) -> Result<[u8; 32], String> {
    // This implements the Overwinter sighash algorithm
    // Reference: ZIP-143
    
    let mut data = Vec::new();
    
    // 1. Header with Overwinter flag
    let header = 0x80000000u32 | (OVERWINTER_TX_VERSION as u32);
    data.write_u32::<LittleEndian>(header)
        .map_err(|e| format!("Failed to write header: {}", e))?;
    
    // 2. Version group ID (BitcoinZ uses same for all versions)
    data.write_u32::<LittleEndian>(BITCOINZ_VERSION_GROUP_ID)
        .map_err(|e| format!("Failed to write version group ID: {}", e))?;
    
    // 3. Hash of all prevouts (if not ANYONECANPAY)
    if (sighash_type & 0x80) == 0 {
        let prevouts_hash = compute_prevouts_hash(inputs)?;
        data.write_all(&prevouts_hash)
            .map_err(|e| format!("Failed to write prevouts hash: {}", e))?;
    } else {
        data.write_all(&[0u8; 32])
            .map_err(|e| format!("Failed to write empty prevouts hash: {}", e))?;
    }
    
    // 4. Hash of all sequences (if not ANYONECANPAY, SINGLE, NONE)
    if (sighash_type & 0x80) == 0 && (sighash_type & 0x1f) != 2 && (sighash_type & 0x1f) != 3 {
        let sequences_hash = compute_sequences_hash(inputs)?;
        data.write_all(&sequences_hash)
            .map_err(|e| format!("Failed to write sequences hash: {}", e))?;
    } else {
        data.write_all(&[0u8; 32])
            .map_err(|e| format!("Failed to write empty sequences hash: {}", e))?;
    }
    
    // 5. Hash of all outputs (if not SINGLE or NONE)
    if (sighash_type & 0x1f) != 2 && (sighash_type & 0x1f) != 3 {
        let outputs_hash = compute_outputs_hash(outputs)?;
        data.write_all(&outputs_hash)
            .map_err(|e| format!("Failed to write outputs hash: {}", e))?;
    } else {
        data.write_all(&[0u8; 32])
            .map_err(|e| format!("Failed to write empty outputs hash: {}", e))?;
    }
    
    // 6. JoinSplits hash (empty for Overwinter transparent)
    data.write_all(&[0u8; 32])
        .map_err(|e| format!("Failed to write joinsplits hash: {}", e))?;
    
    // 7. Lock time (using 0)
    data.write_u32::<LittleEndian>(0)
        .map_err(|e| format!("Failed to write lock time: {}", e))?;
    
    // 8. Expiry height
    let expiry_height = height + BlockHeight::from(10);
    data.write_u32::<LittleEndian>(u32::from(expiry_height))
        .map_err(|e| format!("Failed to write expiry height: {}", e))?;
    
    // 9. Sighash type
    data.write_u32::<LittleEndian>(sighash_type)
        .map_err(|e| format!("Failed to write sighash type: {}", e))?;
    
    // If not ANYONECANPAY, add current input details
    if (sighash_type & 0x80) == 0 {
        // 10. Outpoint
        let (outpoint, _, _) = &inputs[input_index];
        data.write_all(outpoint.hash())
            .map_err(|e| format!("Failed to write outpoint hash: {}", e))?;
        data.write_u32::<LittleEndian>(outpoint.n())
            .map_err(|e| format!("Failed to write outpoint index: {}", e))?;
        
        // 11. Script code
        write_compact_size(&mut data, script_code.0.len() as u64)?;
        data.write_all(&script_code.0)
            .map_err(|e| format!("Failed to write script code: {}", e))?;
        
        // 12. Value
        data.write_u64::<LittleEndian>(u64::from(value))
            .map_err(|e| format!("Failed to write value: {}", e))?;
        
        // 13. Sequence
        data.write_u32::<LittleEndian>(0xfffffffe)
            .map_err(|e| format!("Failed to write sequence: {}", e))?;
    }
    
    // Create personalization with consensus branch ID
    let mut personalization = [0u8; 16];
    personalization[..12].copy_from_slice(ZCASH_SIGHASH_PERSONALIZATION_PREFIX);
    let branch_id: u32 = match BranchId::for_height(params, height) {
        BranchId::Sprout => 0x00000000,
        BranchId::Overwinter => 0x5ba81b19,
        BranchId::Sapling => 0x76b809bb,
        BranchId::Blossom => 0x2bb40e60,
        BranchId::Heartwood => 0xf5b9230b,
        BranchId::Canopy => 0xe9ff75a6,
        BranchId::Nu5 => 0xc2d6d0b4,
        #[cfg(zcash_unstable = "zfuture")]
        BranchId::ZFuture => 0xffffffff,
    };
    personalization[12..16].copy_from_slice(&branch_id.to_le_bytes());
    
    // Compute BLAKE2b hash
    let hash = Params::new()
        .hash_length(32)
        .personal(&personalization)
        .to_state()
        .update(&data)
        .finalize();
    
    let mut result = [0u8; 32];
    result.copy_from_slice(hash.as_bytes());
    
    Ok(result)
}

/// Compute hash of all prevouts
fn compute_prevouts_hash(inputs: &[(OutPoint, TxOut, SecretKey)]) -> Result<[u8; 32], String> {
    let mut data = Vec::new();
    
    for (outpoint, _, _) in inputs {
        data.write_all(outpoint.hash())
            .map_err(|e| format!("Failed to write outpoint hash: {}", e))?;
        data.write_u32::<LittleEndian>(outpoint.n())
            .map_err(|e| format!("Failed to write outpoint index: {}", e))?;
    }
    
    let hash = Params::new()
        .hash_length(32)
        .personal(ZCASH_PREVOUTS_HASH_PERSONALIZATION)
        .to_state()
        .update(&data)
        .finalize();
    
    let mut result = [0u8; 32];
    result.copy_from_slice(hash.as_bytes());
    
    Ok(result)
}

/// Compute hash of all sequences
fn compute_sequences_hash(inputs: &[(OutPoint, TxOut, SecretKey)]) -> Result<[u8; 32], String> {
    let mut data = Vec::new();
    
    for _ in inputs {
        data.write_u32::<LittleEndian>(0xfffffffe)
            .map_err(|e| format!("Failed to write sequence: {}", e))?;
    }
    
    let hash = Params::new()
        .hash_length(32)
        .personal(ZCASH_SEQUENCE_HASH_PERSONALIZATION)
        .to_state()
        .update(&data)
        .finalize();
    
    let mut result = [0u8; 32];
    result.copy_from_slice(hash.as_bytes());
    
    Ok(result)
}

/// Compute hash of all outputs
fn compute_outputs_hash(outputs: &[(TransparentAddress, Amount)]) -> Result<[u8; 32], String> {
    let mut data = Vec::new();
    
    for (addr, amount) in outputs {
        // Write amount (8 bytes)
        data.write_u64::<LittleEndian>(u64::from(*amount))
            .map_err(|e| format!("Failed to write amount: {}", e))?;
        
        // Write script pubkey
        let script = addr.script();
        write_compact_size(&mut data, script.0.len() as u64)?;
        data.write_all(&script.0)
            .map_err(|e| format!("Failed to write script: {}", e))?;
    }
    
    let hash = Params::new()
        .hash_length(32)
        .personal(ZCASH_OUTPUTS_HASH_PERSONALIZATION)
        .to_state()
        .update(&data)
        .finalize();
    
    let mut result = [0u8; 32];
    result.copy_from_slice(hash.as_bytes());
    
    Ok(result)
}

/// Write a variable-length integer
fn write_compact_size(writer: &mut Vec<u8>, size: u64) -> Result<(), String> {
    if size < 0xfd {
        writer.push(size as u8);
    } else if size <= 0xffff {
        writer.push(0xfd);
        writer.write_u16::<LittleEndian>(size as u16)
            .map_err(|e| format!("Failed to write compact size: {}", e))?;
    } else if size <= 0xffffffff {
        writer.push(0xfe);
        writer.write_u32::<LittleEndian>(size as u32)
            .map_err(|e| format!("Failed to write compact size: {}", e))?;
    } else {
        writer.push(0xff);
        writer.write_u64::<LittleEndian>(size)
            .map_err(|e| format!("Failed to write compact size: {}", e))?;
    }
    Ok(())
}

/// Read a variable-length integer
fn read_compact_size(data: &[u8]) -> Result<Option<(u64, usize)>, String> {
    if data.is_empty() {
        return Ok(None);
    }
    
    let first = data[0];
    match first {
        0..=0xfc => Ok(Some((first as u64, 1))),
        0xfd => {
            if data.len() < 3 {
                return Ok(None);
            }
            let val = u16::from_le_bytes([data[1], data[2]]) as u64;
            Ok(Some((val, 3)))
        }
        0xfe => {
            if data.len() < 5 {
                return Ok(None);
            }
            let val = u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as u64;
            Ok(Some((val, 5)))
        }
        0xff => {
            if data.len() < 9 {
                return Ok(None);
            }
            let val = u64::from_le_bytes([
                data[1], data[2], data[3], data[4],
                data[5], data[6], data[7], data[8]
            ]);
            Ok(Some((val, 9)))
        }
    }
}

/// Check if we should use Overwinter for this transaction
pub fn should_use_overwinter(
    transparent_inputs: usize,
    shielded_inputs: usize,
    transparent_outputs: usize,
    shielded_outputs: usize,
) -> bool {
    // Only use Overwinter for pure transparent transactions
    shielded_inputs == 0 && shielded_outputs == 0 && 
    transparent_inputs > 0 && transparent_outputs > 0
}