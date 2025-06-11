/// BitcoinZ v4 Transaction Builder (No Binding Signature)
/// 
/// This module builds v4 Sapling transactions without binding signatures
/// for transparent-only transfers, matching what BitcoinZ expects.

use byteorder::{LittleEndian, WriteBytesExt};
use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
use blake2b_simd::{Params};
use std::io::Write;
use zcash_primitives::{
    consensus::{BlockHeight, Parameters},
    legacy::{Script, TransparentAddress},
    transaction::{
        components::{Amount, OutPoint, TxOut},
    },
};

/// BitcoinZ Sapling constants
const SAPLING_TX_VERSION: i32 = 4;
const BITCOINZ_VERSION_GROUP_ID: u32 = 0x892f2085;
const SIGHASH_ALL: u32 = 1;

/// Personalization strings for BLAKE2b
const ZCASH_PREVOUTS_HASH_PERSONALIZATION: &[u8; 16] = b"ZcashPrevoutHash";
const ZCASH_SEQUENCE_HASH_PERSONALIZATION: &[u8; 16] = b"ZcashSequencHash";
const ZCASH_OUTPUTS_HASH_PERSONALIZATION: &[u8; 16] = b"ZcashOutputsHash";
const ZCASH_SAPLING_SIGHASH_PERSONALIZATION_PREFIX: &[u8; 12] = b"ZcashSigHash";

/// Build a BitcoinZ v4 transaction without binding signature
pub fn build_bitcoinz_v4_no_sig<P: Parameters>(
    params: &P,
    inputs: Vec<(OutPoint, TxOut, SecretKey)>,
    outputs: Vec<(TransparentAddress, Amount)>,
    height: BlockHeight,
) -> Result<Vec<u8>, String> {
    
    // Build and sign the transaction
    let signed_tx = build_and_sign_v4_tx(params, inputs, outputs, height)?;
    
    
    Ok(signed_tx)
}

/// Build and sign v4 transaction
fn build_and_sign_v4_tx<P: Parameters>(
    params: &P,
    inputs: Vec<(OutPoint, TxOut, SecretKey)>,
    outputs: Vec<(TransparentAddress, Amount)>,
    height: BlockHeight,
) -> Result<Vec<u8>, String> {
    let secp = Secp256k1::new();
    let mut tx_data = Vec::new();
    
    // Header (version 4 with overwinter flag)
    let header = 0x80000000u32 | (SAPLING_TX_VERSION as u32);
    tx_data.write_u32::<LittleEndian>(header)
        .map_err(|e| format!("Failed to write header: {}", e))?;
    
    // Version group ID (BitcoinZ)
    tx_data.write_u32::<LittleEndian>(BITCOINZ_VERSION_GROUP_ID)
        .map_err(|e| format!("Failed to write version group ID: {}", e))?;
    
    // Build unsigned transaction structure first to compute sighashes
    let mut unsigned_tx: Vec<u8> = Vec::new();
    
    // Prevouts hash
    let prevouts_hash = compute_prevouts_hash(&inputs)?;
    
    // Sequence hash
    let sequence_hash = compute_sequence_hash(&inputs)?;
    
    // Outputs hash
    let outputs_hash = compute_outputs_hash(&outputs)?;
    
    // Compute signatures for each input
    let mut signatures = Vec::new();
    for (index, (_outpoint, txout, sk)) in inputs.iter().enumerate() {
        
        let sighash = compute_sapling_sighash(
            params,
            height,
            &prevouts_hash,
            &sequence_hash,
            &outputs_hash,
            &inputs,
            index,
            &txout.script_pubkey,
            txout.value,
        )?;
        
        
        let msg = Message::from_slice(&sighash)
            .map_err(|e| format!("Failed to create message: {}", e))?;
        let sig = secp.sign_ecdsa(&msg, &sk);
        
        let pk = PublicKey::from_secret_key(&secp, &sk);
        
        let mut script_sig = Vec::new();
        
        // Signature with sighash type
        let mut sig_bytes = sig.serialize_der().to_vec();
        sig_bytes.push(SIGHASH_ALL as u8);
        
        script_sig.push(sig_bytes.len() as u8);
        script_sig.extend_from_slice(&sig_bytes);
        
        // Public key
        let pk_bytes = pk.serialize();
        script_sig.push(pk_bytes.len() as u8);
        script_sig.extend_from_slice(&pk_bytes);
        
        
        signatures.push(script_sig);
    }
    
    // Now build the complete transaction
    
    // Input count
    write_compact_size(&mut tx_data, inputs.len() as u64)?;
    
    // Write inputs with signatures
    for (i, (outpoint, _, _)) in inputs.iter().enumerate() {
        tx_data.write_all(outpoint.hash())
            .map_err(|e| format!("Failed to write outpoint hash: {}", e))?;
        tx_data.write_u32::<LittleEndian>(outpoint.n())
            .map_err(|e| format!("Failed to write outpoint index: {}", e))?;
        
        write_compact_size(&mut tx_data, signatures[i].len() as u64)?;
        tx_data.write_all(&signatures[i])
            .map_err(|e| format!("Failed to write script sig: {}", e))?;
        
        tx_data.write_u32::<LittleEndian>(0xfffffffe)
            .map_err(|e| format!("Failed to write sequence: {}", e))?;
    }
    
    // Output count
    write_compact_size(&mut tx_data, outputs.len() as u64)?;
    
    // Write outputs
    for (addr, amount) in &outputs {
        tx_data.write_u64::<LittleEndian>(u64::from(*amount))
            .map_err(|e| format!("Failed to write amount: {}", e))?;
        
        let script = addr.script();
        write_compact_size(&mut tx_data, script.0.len() as u64)?;
        tx_data.write_all(&script.0)
            .map_err(|e| format!("Failed to write script: {}", e))?;
    }
    
    // Lock time
    tx_data.write_u32::<LittleEndian>(0)
        .map_err(|e| format!("Failed to write lock time: {}", e))?;
    
    // Expiry height (BitcoinZ uses 0 for no expiry)
    tx_data.write_u32::<LittleEndian>(0)
        .map_err(|e| format!("Failed to write expiry height: {}", e))?;
    
    // Value balance (0 for transparent only)
    tx_data.write_i64::<LittleEndian>(0)
        .map_err(|e| format!("Failed to write value balance: {}", e))?;
    
    // No shielded spends
    write_compact_size(&mut tx_data, 0)?;
    
    // No shielded outputs
    write_compact_size(&mut tx_data, 0)?;
    
    // No joinsplits
    write_compact_size(&mut tx_data, 0)?;
    
    // For transparent-only transactions, BitcoinZ doesn't expect any binding signature
    // Don't add any binding signature bytes
    
    
    Ok(tx_data)
}

/// Compute Sapling sighash
fn compute_sapling_sighash<P: Parameters>(
    params: &P,
    height: BlockHeight,
    prevouts_hash: &[u8; 32],
    sequence_hash: &[u8; 32],
    outputs_hash: &[u8; 32],
    inputs: &[(OutPoint, TxOut, SecretKey)],
    input_index: usize,
    script_code: &Script,
    value: Amount,
) -> Result<[u8; 32], String> {
    let mut data = Vec::new();
    
    // 1. Header
    let header = 0x80000000u32 | (SAPLING_TX_VERSION as u32);
    data.write_u32::<LittleEndian>(header)
        .map_err(|e| format!("Failed to write header: {}", e))?;
    
    // 2. Version group ID
    data.write_u32::<LittleEndian>(BITCOINZ_VERSION_GROUP_ID)
        .map_err(|e| format!("Failed to write version group ID: {}", e))?;
    
    // 3. Prevouts hash
    data.write_all(prevouts_hash)
        .map_err(|e| format!("Failed to write prevouts hash: {}", e))?;
    
    // 4. Sequence hash
    data.write_all(sequence_hash)
        .map_err(|e| format!("Failed to write sequence hash: {}", e))?;
    
    // 5. Outputs hash
    data.write_all(outputs_hash)
        .map_err(|e| format!("Failed to write outputs hash: {}", e))?;
    
    // 6. JoinSplits hash (empty)
    data.write_all(&[0u8; 32])
        .map_err(|e| format!("Failed to write joinsplits hash: {}", e))?;
    
    // 7. ShieldedSpends hash (empty)
    data.write_all(&[0u8; 32])
        .map_err(|e| format!("Failed to write shielded spends hash: {}", e))?;
    
    // 8. ShieldedOutputs hash (empty)
    data.write_all(&[0u8; 32])
        .map_err(|e| format!("Failed to write shielded outputs hash: {}", e))?;
    
    // 9. Lock time
    data.write_u32::<LittleEndian>(0)
        .map_err(|e| format!("Failed to write lock time: {}", e))?;
    
    // 10. Expiry height (BitcoinZ uses 0 for no expiry)
    data.write_u32::<LittleEndian>(0)
        .map_err(|e| format!("Failed to write expiry height: {}", e))?;
    
    // 11. Value balance
    data.write_i64::<LittleEndian>(0)
        .map_err(|e| format!("Failed to write value balance: {}", e))?;
    
    // 12. Sighash type
    data.write_u32::<LittleEndian>(SIGHASH_ALL)
        .map_err(|e| format!("Failed to write sighash type: {}", e))?;
    
    // 13. Input details (for the input being signed)
    // This part is only included for non-ANYONECANPAY
    
    // Outpoint
    let (outpoint, _, _) = &inputs[input_index];
    data.write_all(outpoint.hash())
        .map_err(|e| format!("Failed to write outpoint hash: {}", e))?;
    data.write_u32::<LittleEndian>(outpoint.n())
        .map_err(|e| format!("Failed to write outpoint index: {}", e))?;
    
    // Script code
    write_compact_size(&mut data, script_code.0.len() as u64)?;
    data.write_all(&script_code.0)
        .map_err(|e| format!("Failed to write script code: {}", e))?;
    
    // Value
    data.write_u64::<LittleEndian>(u64::from(value))
        .map_err(|e| format!("Failed to write value: {}", e))?;
    
    // Sequence
    data.write_u32::<LittleEndian>(0xfffffffe)
        .map_err(|e| format!("Failed to write sequence: {}", e))?;
    
    // Create personalization with branch ID
    let mut personalization = [0u8; 16];
    personalization[..12].copy_from_slice(ZCASH_SAPLING_SIGHASH_PERSONALIZATION_PREFIX);
    // BitcoinZ uses a fixed consensus branch ID regardless of height
    let branch_id: u32 = 1991772603; // 0x76b809bb
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
    
    // Actually, let me try NOT reversing to see if that helps
    // result.reverse();
    
    Ok(result)
}

/// Helper functions (same as in Overwinter builder)

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

fn compute_sequence_hash(inputs: &[(OutPoint, TxOut, SecretKey)]) -> Result<[u8; 32], String> {
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

fn compute_outputs_hash(outputs: &[(TransparentAddress, Amount)]) -> Result<[u8; 32], String> {
    let mut data = Vec::new();
    
    for (addr, amount) in outputs {
        data.write_u64::<LittleEndian>(u64::from(*amount))
            .map_err(|e| format!("Failed to write amount: {}", e))?;
        
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