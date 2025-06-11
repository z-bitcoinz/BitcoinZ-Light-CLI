# BitcoinZ Light Client Testing Guide

## Overview

This guide provides step-by-step instructions for testing the BitcoinZ light client implementation. It covers wallet setup, transaction testing, and verification procedures.

## Prerequisites

- Rust toolchain installed
- Access to a BitcoinZ lightwalletd server
- BitcoinZ for testing (testnet or small mainnet amounts)

## Test Environment Setup

### 1. Build the Client

```bash
cd /path/to/btcz-light-cli
cargo build --release
```

### 2. Configure Test Server

For testing, you can use:
- **Mainnet Server**: `http://93.107.37.216:9067`
- **Local Server**: `http://localhost:9067` (if running your own)

### 3. Create Test Directory

```bash
mkdir -p /tmp/btcz-test-wallet
```

## Wallet Creation and Import

### Create New Wallet

```bash
./target/release/bitcoinz-light-cli \
    --server http://93.107.37.216:9067 \
    --data-dir /tmp/btcz-test-wallet \
    new
```

Save the seed phrase securely!

### Import Existing Wallet

```bash
./target/release/bitcoinz-light-cli \
    --server http://93.107.37.216:9067 \
    --data-dir /tmp/btcz-test-wallet \
    import "your twelve word seed phrase goes here"
```

## Basic Operations Testing

### 1. Check Sync Status

```bash
./target/release/bitcoinz-light-cli \
    --server http://93.107.37.216:9067 \
    --data-dir /tmp/btcz-test-wallet \
    sync
```

Expected output:
```json
{
  "result": "success"
}
```

### 2. Check Balance

```bash
./target/release/bitcoinz-light-cli \
    --server http://93.107.37.216:9067 \
    --data-dir /tmp/btcz-test-wallet \
    balance
```

Expected output format:
```json
{
  "uabalance": 0,
  "zbalance": 0,
  "verified_zbalance": 0,
  "spendable_zbalance": 0,
  "unverified_zbalance": 0,
  "tbalance": 100000000,
  "t_addresses": [
    {
      "address": "t1YourAddressHere",
      "balance": 100000000
    }
  ]
}
```

### 3. List Addresses

```bash
./target/release/bitcoinz-light-cli \
    --server http://93.107.37.216:9067 \
    --data-dir /tmp/btcz-test-wallet \
    addresses
```

## Transaction Testing

### Test 1: Transparent-to-Transparent (t→t) Transaction

This is currently the only supported transaction type.

```bash
# Send 0.1 BTCZ
./target/release/bitcoinz-light-cli \
    --server http://93.107.37.216:9067 \
    --data-dir /tmp/btcz-test-wallet \
    send "t1RecipientAddressHere" 10000000
```

Expected successful output:
```json
{
  "txid": "7ca9d30f60cf1e9d3be0cfefbaeccd713cf04b631722d5882e6ce9373cde6065"
}
```

### Test 2: Multiple Recipients

```bash
# Send to multiple addresses
./target/release/bitcoinz-light-cli \
    --server http://93.107.37.216:9067 \
    --data-dir /tmp/btcz-test-wallet \
    send "{'t1Address1': 5000000, 't1Address2': 3000000}"
```

### Test 3: Minimum Amount Transaction

```bash
# Send minimum amount (1 satoshi + fee)
./target/release/bitcoinz-light-cli \
    --server http://93.107.37.216:9067 \
    --data-dir /tmp/btcz-test-wallet \
    send "t1RecipientAddressHere" 1
```

## Verification Procedures

### 1. Verify Transaction Status

After sending, check the transaction:

```bash
./target/release/bitcoinz-light-cli \
    --server http://93.107.37.216:9067 \
    --data-dir /tmp/btcz-test-wallet \
    notes
```

Look for:
- `pending_utxos` - Unconfirmed transactions
- `utxos` - Confirmed transactions

### 2. Verify on Block Explorer

Use a BitcoinZ block explorer to verify:
1. Transaction appears in mempool
2. Transaction gets confirmed
3. Correct amounts and addresses

### 3. Rescan Wallet

If transactions don't appear:

```bash
./target/release/bitcoinz-light-cli \
    --server http://93.107.37.216:9067 \
    --data-dir /tmp/btcz-test-wallet \
    rescan
```

## Debug Mode Testing

### Enable Debug Output

```bash
export RUST_LOG=debug

./target/release/bitcoinz-light-cli \
    --server http://93.107.37.216:9067 \
    --data-dir /tmp/btcz-test-wallet \
    send "t1RecipientAddressHere" 10000000
```

Debug output includes:
- Transaction type detection
- Sighash computation details
- Transaction building steps
- Signature generation

### Analyze Transaction Hex

For failed transactions, save and analyze the hex:

```bash
# Save transaction hex from debug output
echo "YOUR_HEX_HERE" > tx.hex

# Decode (requires bitcoin-cli or similar)
xxd -r -p tx.hex | hexdump -C
```

## Common Test Scenarios

### Scenario 1: Insufficient Funds

Test error handling:
```bash
# Try to send more than balance
./target/release/bitcoinz-light-cli send "t1Address" 999999999999
```

Expected: "Insufficient verified funds" error

### Scenario 2: Invalid Address

```bash
# Test with invalid address
./target/release/bitcoinz-light-cli send "InvalidAddress" 10000000
```

Expected: "Invalid recipient address" error

### Scenario 3: Locked Wallet

```bash
# Lock wallet
./target/release/bitcoinz-light-cli lock

# Try to send
./target/release/bitcoinz-light-cli send "t1Address" 10000000
```

Expected: "Cannot spend while wallet is locked" error

## Performance Testing

### Transaction Building Time

```bash
time ./target/release/bitcoinz-light-cli \
    --server http://93.107.37.216:9067 \
    --data-dir /tmp/btcz-test-wallet \
    send "t1Address" 10000000
```

Expected: < 2 seconds for transaction creation

### Sync Performance

```bash
time ./target/release/bitcoinz-light-cli \
    --server http://93.107.37.216:9067 \
    --data-dir /tmp/btcz-test-wallet \
    sync
```

Monitor:
- Initial sync time
- Incremental sync time
- Memory usage

## Test Checklist

- [ ] Wallet creation works
- [ ] Wallet import works
- [ ] Balance displays correctly
- [ ] Address generation works
- [ ] T→T transactions succeed
- [ ] Change addresses work correctly
- [ ] Transaction fees are correct
- [ ] Sync completes successfully
- [ ] Rescan recovers all transactions
- [ ] Error messages are helpful
- [ ] Debug output is comprehensive

## Troubleshooting Test Failures

### Transaction Rejected

1. Check debug output for specific error
2. Verify balance and confirmations
3. Ensure server is synced
4. Try rescan

### Sync Issues

1. Check server connectivity
2. Verify server is running
3. Check firewall/network issues
4. Try different server

### Balance Incorrect

1. Perform rescan
2. Check transaction confirmations
3. Verify server height
4. Check for reorgs

## Reporting Issues

When reporting test failures, include:
1. Exact command used
2. Full error message
3. Debug output (if available)
4. Transaction hex (for tx errors)
5. Server URL and version
6. Client version (git commit)

## Automated Testing

### Run Unit Tests

```bash
cd lib
cargo test
```

### Run Integration Tests

```bash
cd cli
cargo test -- --test-threads=1
```

### Test Script Example

```bash
#!/bin/bash
# test_bitcoinz_tx.sh

SERVER="http://93.107.37.216:9067"
WALLET_DIR="/tmp/btcz-test-$$"
CLI="./target/release/bitcoinz-light-cli"

echo "Creating test wallet..."
$CLI --server $SERVER --data-dir $WALLET_DIR new

echo "Checking balance..."
$CLI --server $SERVER --data-dir $WALLET_DIR balance

echo "Test complete"
rm -rf $WALLET_DIR
```

## Success Criteria

A successful test run should:
1. Complete without errors
2. Show correct balances
3. Successfully send transactions
4. Receive network confirmations
5. Handle errors gracefully