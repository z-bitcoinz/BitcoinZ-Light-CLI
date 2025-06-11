/// Ultra-simple Bitcoin v1 transaction builder for testing
/// This creates the most basic Bitcoin transaction format

use byteorder::{LittleEndian, WriteBytesExt};
use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
use sha2::{Digest, Sha256};
use std::io::Write;
use zcash_primitives::{
    consensus::{BlockHeight, Parameters},
    legacy::TransparentAddress,
    transaction::components::{Amount, OutPoint, TxOut},
};

/// Build a simple Bitcoin v1 transaction
pub fn build_simple_v1_tx<P: Parameters>(
    _params: &P,
    inputs: Vec<(OutPoint, TxOut, SecretKey)>,
    outputs: Vec<(TransparentAddress, Amount)>,
    _height: BlockHeight,
) -> Result<Vec<u8>, String> {
    
    let secp = Secp256k1::new();
    let mut tx_data = Vec::new();
    
    // Version (1)
    tx_data.write_u32::<LittleEndian>(1)
        .map_err(|e| format!("Failed to write version: {}", e))?;
    
    // Input count
    write_compact_size(&mut tx_data, inputs.len() as u64)?;
    
    // First pass: write unsigned inputs
    let mut unsigned_tx = tx_data.clone();
    for (outpoint, _, _) in &inputs {
        unsigned_tx.write_all(outpoint.hash())
            .map_err(|e| format!("Failed to write outpoint hash: {}", e))?;
        unsigned_tx.write_u32::<LittleEndian>(outpoint.n())
            .map_err(|e| format!("Failed to write outpoint index: {}", e))?;
        write_compact_size(&mut unsigned_tx, 0)?; // Empty scriptSig
        unsigned_tx.write_u32::<LittleEndian>(0xffffffff)
            .map_err(|e| format!("Failed to write sequence: {}", e))?;
    }
    
    // Write outputs to unsigned tx
    write_compact_size(&mut unsigned_tx, outputs.len() as u64)?;
    for (addr, amount) in &outputs {
        unsigned_tx.write_u64::<LittleEndian>(u64::from(*amount))
            .map_err(|e| format!("Failed to write amount: {}", e))?;
        let script = addr.script();
        write_compact_size(&mut unsigned_tx, script.0.len() as u64)?;
        unsigned_tx.write_all(&script.0)
            .map_err(|e| format!("Failed to write script: {}", e))?;
    }
    
    // Lock time
    unsigned_tx.write_u32::<LittleEndian>(0)
        .map_err(|e| format!("Failed to write lock time: {}", e))?;
    
    // Now sign each input
    let mut signatures = Vec::new();
    for (index, (_, txout, sk)) in inputs.iter().enumerate() {
        // Create sighash for this input
        let mut sighash_data = unsigned_tx.clone();
        
        // For the input being signed, replace empty scriptSig with scriptPubKey
        let mut input_start = 4 + 1; // version + input count
        for i in 0..index {
            // Skip to next input: 32 (txid) + 4 (vout) + 1 (script len) + 0 (script) + 4 (sequence)
            input_start += 32 + 4 + 1 + 4;
        }
        
        // Insert the scriptPubKey for this input
        let script_offset = input_start + 32 + 4;
        sighash_data[script_offset] = txout.script_pubkey.0.len() as u8;
        sighash_data.splice(script_offset + 1..script_offset + 1, txout.script_pubkey.0.iter().cloned());
        
        // Append sighash type
        sighash_data.write_u32::<LittleEndian>(1)?; // SIGHASH_ALL
        
        // Double SHA256
        let hash1 = Sha256::digest(&sighash_data);
        let sighash = Sha256::digest(&hash1);
        
        
        // Sign
        let msg = Message::from_slice(&sighash)
            .map_err(|e| format!("Failed to create message: {}", e))?;
        let sig = secp.sign_ecdsa(&msg, &sk);
        
        // Create scriptSig
        let pk = PublicKey::from_secret_key(&secp, &sk);
        let mut script_sig = Vec::new();
        
        // Signature with sighash type
        let mut sig_bytes = sig.serialize_der().to_vec();
        sig_bytes.push(1); // SIGHASH_ALL
        script_sig.push(sig_bytes.len() as u8);
        script_sig.extend_from_slice(&sig_bytes);
        
        // Public key
        let pk_bytes = pk.serialize();
        script_sig.push(pk_bytes.len() as u8);
        script_sig.extend_from_slice(&pk_bytes);
        
        signatures.push(script_sig);
    }
    
    // Build final transaction with signatures
    tx_data.clear();
    
    // Version
    tx_data.write_u32::<LittleEndian>(1)?;
    
    // Input count
    write_compact_size(&mut tx_data, inputs.len() as u64)?;
    
    // Inputs with signatures
    for (i, (outpoint, _, _)) in inputs.iter().enumerate() {
        tx_data.write_all(outpoint.hash())?;
        tx_data.write_u32::<LittleEndian>(outpoint.n())?;
        write_compact_size(&mut tx_data, signatures[i].len() as u64)?;
        tx_data.write_all(&signatures[i])?;
        tx_data.write_u32::<LittleEndian>(0xffffffff)?;
    }
    
    // Output count
    write_compact_size(&mut tx_data, outputs.len() as u64)?;
    
    // Outputs
    for (addr, amount) in &outputs {
        tx_data.write_u64::<LittleEndian>(u64::from(*amount))?;
        let script = addr.script();
        write_compact_size(&mut tx_data, script.0.len() as u64)?;
        tx_data.write_all(&script.0)?;
    }
    
    // Lock time
    tx_data.write_u32::<LittleEndian>(0)?;
    
    
    Ok(tx_data)
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