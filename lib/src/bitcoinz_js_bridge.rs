/// BitcoinZ JavaScript Bridge
/// 
/// This module bridges to the BitcoinZ JavaScript library to create
/// properly formatted transactions that BitcoinZ nodes will accept.

use std::process::Command;
use serde::{Deserialize, Serialize};
use zcash_primitives::{
    consensus::{BlockHeight, Parameters},
    legacy::TransparentAddress,
    transaction::components::{Amount, OutPoint, TxOut},
};
use zcash_client_backend::encoding::AddressCodec;
use secp256k1::SecretKey;

#[derive(Debug, Serialize, Deserialize)]
struct JsTransactionResult {
    success: bool,
    txid: Option<String>,
    hex: Option<String>,
    size: Option<usize>,
    error: Option<String>,
}

/// Build a BitcoinZ transaction using the JavaScript library
pub fn build_bitcoinz_js_tx<P: Parameters>(
    params: &P,
    inputs: Vec<(OutPoint, TxOut, SecretKey)>,
    outputs: Vec<(TransparentAddress, Amount)>,
    _height: BlockHeight,
) -> Result<Vec<u8>, String> {
    
    if inputs.len() != 1 || outputs.len() > 2 {
        return Err("JavaScript bridge currently only supports single input transactions".to_string());
    }
    
    let (outpoint, _txout, sk) = &inputs[0];
    let (to_addr, amount) = &outputs[0];
    
    // Convert secret key to WIF format
    let sk_bytes = sk.as_ref();
    let wif = secret_key_to_wif(sk_bytes)?;
    
    // Get the to address
    let to_address = to_addr.encode(params);
    
    // Amount in satoshis
    let amount_sats = u64::from(*amount);
    
    // Build UTXO JSON
    let utxo_json = serde_json::json!({
        "txid": hex::encode(outpoint.hash()),
        "vout": outpoint.n(),
        "scriptPubKey": hex::encode(&_txout.script_pubkey.0),
        "satoshis": u64::from(_txout.value)
    });
    
    // Find the Node.js script path
    let script_path = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?
        .join("bitcoinz-tx-builder.js");
    
    if !script_path.exists() {
        return Err(format!("BitcoinZ transaction builder script not found at {:?}", script_path));
    }
    
    
    // Execute the Node.js script
    let output = Command::new("node")
        .arg(script_path)
        .arg(&wif)
        .arg(&to_address)
        .arg(amount_sats.to_string())
        .arg(utxo_json.to_string())
        .output()
        .map_err(|e| format!("Failed to execute Node.js script: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Node.js script failed: {}", stderr));
    }
    
    // Parse the output
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Find the JSON output
    if let Some(json_start) = stdout.find("---JSON OUTPUT---") {
        let json_str = &stdout[json_start + 17..].trim();
        
        let result: JsTransactionResult = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse JSON output: {}", e))?;
        
        if result.success {
            if let Some(hex) = result.hex {
                let tx_bytes = hex::decode(&hex)
                    .map_err(|e| format!("Failed to decode transaction hex: {}", e))?;
                
                
                return Ok(tx_bytes);
            } else {
                return Err("No transaction hex in successful result".to_string());
            }
        } else {
            return Err(format!("JavaScript error: {}", result.error.unwrap_or_default()));
        }
    }
    
    Err("Failed to parse Node.js output".to_string())
}

/// Convert secret key bytes to WIF format
fn secret_key_to_wif(sk_bytes: &[u8]) -> Result<String, String> {
    use base58::ToBase58;
    use sha2::{Sha256, Digest};
    
    // BitcoinZ mainnet private key prefix is 0x80
    let mut data = vec![0x80];
    data.extend_from_slice(sk_bytes);
    data.push(0x01); // Compressed pubkey flag
    
    // Double SHA256 for checksum
    let hash1 = Sha256::digest(&data);
    let hash2 = Sha256::digest(&hash1);
    
    // Append first 4 bytes of checksum
    data.extend_from_slice(&hash2[..4]);
    
    Ok(data.to_base58())
}