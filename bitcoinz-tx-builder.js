#!/usr/bin/env node

/**
 * BitcoinZ Transaction Builder
 * 
 * This script uses the native BitcoinZ JavaScript library to create
 * properly formatted transactions that BitcoinZ nodes will accept.
 */

// Adjust the path based on where the script is run from
const path = require('path');
const bitcorePath = path.resolve(__dirname, '../../../../bitcore-lib-btcz');
console.log('Loading BitcoinZ library from:', bitcorePath);
const bitcore = require(bitcorePath);
const Transaction = bitcore.Transaction;
const PrivateKey = bitcore.PrivateKey;

// Parse command line arguments
const args = process.argv.slice(2);
if (args.length < 3) {
    console.error('Usage: node bitcoinz-tx-builder.js <private_key_wif> <to_address> <amount_satoshis> [<utxo_json>]');
    process.exit(1);
}

const privateKeyWIF = args[0];
const toAddress = args[1];
const amountSatoshis = parseInt(args[2]);
const utxoJson = args[3] || null;

try {
    // Create private key from WIF
    const privateKey = new PrivateKey(privateKeyWIF);
    console.log('From address:', privateKey.toAddress().toString());

    // Create new transaction
    const tx = new Transaction();
    
    // Set BitcoinZ-specific parameters
    tx.fOverwintered = true;
    tx.version = 4; // Sapling
    tx.nVersionGroupId = 0x892f2085; // BitcoinZ version group ID
    tx.nExpiryHeight = 0; // No expiry

    // Add UTXO input
    if (utxoJson) {
        const utxo = JSON.parse(utxoJson);
        tx.from({
            txId: utxo.txid,
            outputIndex: utxo.vout,
            address: privateKey.toAddress(),
            script: utxo.scriptPubKey,
            satoshis: utxo.satoshis
        });
    } else {
        // For testing, use the known UTXO
        tx.from({
            txId: '8f4c1ef0421671a887dee75949dece73151d6685de06fc2135c8e88e8020be57',
            outputIndex: 1,
            address: privateKey.toAddress(),
            script: bitcore.Script.buildPublicKeyHashOut(privateKey.toAddress()),
            satoshis: 100000000 // 1 BTCZ
        });
    }

    // Add output
    tx.to(toAddress, amountSatoshis);

    // Calculate fee (10000 satoshis)
    const fee = 10000;
    
    // Add change output if needed
    const inputAmount = 100000000; // 1 BTCZ
    const changeAmount = inputAmount - amountSatoshis - fee;
    if (changeAmount > 0) {
        tx.change(privateKey.toAddress());
    }

    // Sign the transaction
    tx.sign(privateKey);

    // Get the raw transaction hex
    const rawTx = tx.serialize();
    console.log('Transaction created successfully!');
    console.log('Transaction ID:', tx.id);
    console.log('Raw transaction hex:', rawTx);
    console.log('Transaction size:', Buffer.from(rawTx, 'hex').length, 'bytes');
    
    // Output JSON for easy parsing
    console.log('---JSON OUTPUT---');
    console.log(JSON.stringify({
        success: true,
        txid: tx.id,
        hex: rawTx,
        size: Buffer.from(rawTx, 'hex').length
    }));

} catch (error) {
    console.error('Error creating transaction:', error.message);
    console.log('---JSON OUTPUT---');
    console.log(JSON.stringify({
        success: false,
        error: error.message
    }));
    process.exit(1);
}