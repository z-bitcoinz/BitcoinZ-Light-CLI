/// BitcoinZ Legacy Transaction Builder
/// 
/// This module builds legacy (pre-Overwinter) Bitcoin-style transactions
/// for transparent-only transfers, completely avoiding Sapling components.

use zcash_primitives::{
    legacy::{Script, TransparentAddress},
    transaction::{
        components::{Amount, OutPoint, TxOut},
    },
};
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Write;

/// Build a legacy Bitcoin-style transaction for BitcoinZ
/// This completely avoids any Zcash-specific features
pub fn build_legacy_tx(
    inputs: Vec<(OutPoint, TxOut, secp256k1::SecretKey)>,
    outputs: Vec<(TransparentAddress, Amount)>,
) -> Result<Vec<u8>, String> {
    
    let mut tx_bytes = Vec::new();
    
    // Version (4 bytes) - Use version 1 for legacy
    tx_bytes.write_u32::<LittleEndian>(1)?;
    
    // Input count (varint)
    write_varint(&mut tx_bytes, inputs.len() as u64)?;
    
    // Inputs
    for (outpoint, _txout, _sk) in &inputs {
        // Previous output (36 bytes: 32 for txid + 4 for index)
        tx_bytes.write_all(&outpoint.hash())?;
        tx_bytes.write_u32::<LittleEndian>(outpoint.n())?;
        
        // Script sig (empty for now, will sign later)
        write_varint(&mut tx_bytes, 0)?; // script length
        
        // Sequence
        tx_bytes.write_u32::<LittleEndian>(0xffffffff)?;
    }
    
    // Output count (varint)
    write_varint(&mut tx_bytes, outputs.len() as u64)?;
    
    // Outputs
    for (addr, amount) in &outputs {
        // Amount (8 bytes)
        tx_bytes.write_u64::<LittleEndian>(u64::from(*amount))?;
        
        // Script pubkey
        let script = addr.script();
        write_varint(&mut tx_bytes, script.0.len() as u64)?;
        tx_bytes.write_all(&script.0)?;
    }
    
    // Locktime (4 bytes)
    tx_bytes.write_u32::<LittleEndian>(0)?;
    
    // TODO: Sign the inputs properly
    // This would require implementing Bitcoin-style signing
    
    Err("Legacy transaction building not fully implemented - needs signing logic".to_string())
}

/// Write a variable-length integer
fn write_varint(writer: &mut Vec<u8>, n: u64) -> std::io::Result<()> {
    if n < 0xfd {
        writer.write_u8(n as u8)?;
    } else if n <= 0xffff {
        writer.write_u8(0xfd)?;
        writer.write_u16::<LittleEndian>(n as u16)?;
    } else if n <= 0xffffffff {
        writer.write_u8(0xfe)?;
        writer.write_u32::<LittleEndian>(n as u32)?;
    } else {
        writer.write_u8(0xff)?;
        writer.write_u64::<LittleEndian>(n)?;
    }
    Ok(())
}