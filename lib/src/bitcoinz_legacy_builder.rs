/// BitcoinZ Legacy (v1) Transaction Builder
/// 
/// This module builds legacy Bitcoin v1 transactions for transparent-only transfers
/// to avoid issues with Overwinter/Sapling transaction formats.

use byteorder::{LittleEndian, WriteBytesExt};
use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
use sha2::{Digest, Sha256};
use std::io::Write;
use zcash_primitives::{
    consensus::{BlockHeight, Parameters},
    legacy::{Script, TransparentAddress},
    transaction::{
        components::{Amount, OutPoint, TxOut},
    },
};

/// Build a raw legacy v1 transaction for BitcoinZ transparent-only transfers
pub fn build_legacy_tx<P: Parameters>(
    _params: &P,
    inputs: Vec<(OutPoint, TxOut, SecretKey)>,
    outputs: Vec<(TransparentAddress, Amount)>,
    _height: BlockHeight,
) -> Result<Vec<u8>, String> {
    
    // First, build the unsigned transaction
    let unsigned_tx = build_unsigned_legacy_tx(&inputs, &outputs)?;
    
    // Then sign all inputs
    let signed_tx = sign_legacy_transaction(unsigned_tx, inputs, &outputs)?;
    
    
    Ok(signed_tx)
}

/// Build unsigned legacy transaction
fn build_unsigned_legacy_tx(
    inputs: &[(OutPoint, TxOut, SecretKey)],
    outputs: &[(TransparentAddress, Amount)],
) -> Result<Vec<u8>, String> {
    let mut tx_data = Vec::new();
    
    // Version (4 bytes) - v1 = 0x00000001
    tx_data.write_u32::<LittleEndian>(1)
        .map_err(|e| format!("Failed to write version: {}", e))?;
    
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
        
        // Sequence (0xffffffff for RBF disabled)
        tx_data.write_u32::<LittleEndian>(0xffffffff)
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
    
    // Lock time (0 = no lock time)
    tx_data.write_u32::<LittleEndian>(0)
        .map_err(|e| format!("Failed to write lock time: {}", e))?;
    
    Ok(tx_data)
}

/// Sign the legacy transaction
fn sign_legacy_transaction(
    unsigned_tx: Vec<u8>,
    inputs: Vec<(OutPoint, TxOut, SecretKey)>,
    outputs: &[(TransparentAddress, Amount)],
) -> Result<Vec<u8>, String> {
    let secp = Secp256k1::new();
    let mut signatures = Vec::new();
    
    // Compute signatures for each input
    for (index, (_, txout, sk)) in inputs.iter().enumerate() {
        // Compute the sighash for this input
        let sighash = compute_legacy_sighash(
            &unsigned_tx,
            index,
            &txout.script_pubkey,
            1, // SIGHASH_ALL
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
        sig_bytes.push(1); // SIGHASH_ALL
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
    
    // Version
    signed_tx.write_u32::<LittleEndian>(1)
        .map_err(|e| format!("Failed to write version: {}", e))?;
    
    // Input count
    write_compact_size(&mut signed_tx, inputs.len() as u64)?;
    
    // Write inputs with signatures
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
        signed_tx.write_u32::<LittleEndian>(0xffffffff)
            .map_err(|e| format!("Failed to write sequence: {}", e))?;
    }
    
    // Output count
    write_compact_size(&mut signed_tx, outputs.len() as u64)?;
    
    // Write outputs
    for (addr, amount) in outputs {
        // Amount
        signed_tx.write_u64::<LittleEndian>(u64::from(*amount))
            .map_err(|e| format!("Failed to write amount: {}", e))?;
        
        // Script pubkey
        let script = addr.script();
        write_compact_size(&mut signed_tx, script.0.len() as u64)?;
        signed_tx.write_all(&script.0)
            .map_err(|e| format!("Failed to write script: {}", e))?;
    }
    
    // Lock time
    signed_tx.write_u32::<LittleEndian>(0)
        .map_err(|e| format!("Failed to write lock time: {}", e))?;
    
    Ok(signed_tx)
}

/// Compute legacy sighash using double SHA256
fn compute_legacy_sighash(
    tx_data: &[u8],
    input_index: usize,
    script_code: &Script,
    sighash_type: u32,
) -> Result<[u8; 32], String> {
    let mut data = Vec::new();
    
    // Copy transaction up to inputs
    let mut cursor = 0;
    
    // Version (4 bytes)
    data.extend_from_slice(&tx_data[0..4]);
    cursor = 4;
    
    // Input count
    let (input_count, varint_size) = read_compact_size(&tx_data[cursor..])?
        .ok_or("Failed to read input count")?;
    write_compact_size(&mut data, input_count)?;
    cursor += varint_size;
    
    // Process inputs
    for i in 0..input_count as usize {
        // Previous output (36 bytes)
        data.extend_from_slice(&tx_data[cursor..cursor + 36]);
        cursor += 36;
        
        // Skip original script sig length
        let (script_len, varint_size) = read_compact_size(&tx_data[cursor..])?
            .ok_or("Failed to read script length")?;
        cursor += varint_size + script_len as usize;
        
        // Write script sig (empty for non-signing inputs, script_code for signing input)
        if i == input_index {
            write_compact_size(&mut data, script_code.0.len() as u64)?;
            data.extend_from_slice(&script_code.0);
        } else {
            write_compact_size(&mut data, 0)?;
        }
        
        // Sequence (4 bytes)
        data.extend_from_slice(&tx_data[cursor..cursor + 4]);
        cursor += 4;
    }
    
    // Copy the rest of the transaction (outputs and locktime)
    data.extend_from_slice(&tx_data[cursor..]);
    
    // Append sighash type
    data.write_u32::<LittleEndian>(sighash_type)
        .map_err(|e| format!("Failed to write sighash type: {}", e))?;
    
    // Double SHA256
    let hash1 = Sha256::digest(&data);
    let hash2 = Sha256::digest(&hash1);
    
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash2);
    
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