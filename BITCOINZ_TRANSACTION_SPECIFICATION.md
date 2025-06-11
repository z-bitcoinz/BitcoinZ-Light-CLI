# BitcoinZ Transaction Specification

## Overview

This document provides the technical specification for BitcoinZ v4 (Sapling) transactions as implemented in the BitcoinZ light client. It details the exact format, constants, and algorithms required to create valid BitcoinZ transactions.

## Transaction Version

BitcoinZ uses Sapling v4 transactions for all post-Overwinter transactions:
- **Version**: 4
- **Overwinter Flag**: Set (0x80000000)
- **Header**: `0x80000004` (version 4 with Overwinter flag)

## Key Constants

```rust
// Transaction version with Overwinter flag
const HEADER: u32 = 0x80000004;

// BitcoinZ version group ID
const VERSION_GROUP_ID: u32 = 0x892f2085;

// Consensus branch ID (fixed for BitcoinZ)
const CONSENSUS_BRANCH_ID: u32 = 1991772603; // 0x76b809bb

// Default values
const DEFAULT_EXPIRY_HEIGHT: u32 = 0;  // No expiry
const DEFAULT_LOCK_TIME: u32 = 0;
const SEQUENCE_NUMBER: u32 = 0xfffffffe;
```

## Transaction Structure

### 1. Header Section
```
[4 bytes] header           = 0x80000004 (LE)
[4 bytes] nVersionGroupId  = 0x892f2085 (LE)
```

### 2. Transparent Inputs
```
[varint]  tx_in_count
for each input:
    [32 bytes] prevout_hash (LE)
    [4 bytes]  prevout_index (LE)
    [varint]   script_length
    [variable] script_sig
    [4 bytes]  sequence = 0xfffffffe (LE)
```

### 3. Transparent Outputs
```
[varint]  tx_out_count
for each output:
    [8 bytes]  value_satoshis (LE)
    [varint]   script_length
    [variable] script_pubkey
```

### 4. Lock Time and Expiry
```
[4 bytes] nLockTime     = 0x00000000 (LE)
[4 bytes] nExpiryHeight = 0x00000000 (LE)
```

### 5. Sapling Data (Transparent-Only)
```
[8 bytes] valueBalance = 0x0000000000000000 (LE)
[varint]  nShieldedSpend = 0
[varint]  nShieldedOutput = 0
[varint]  nJoinSplit = 0
```

### 6. Binding Signature
For transparent-only transactions: **NO binding signature bytes**

## Sighash Computation

### Algorithm: Sapling (ZIP-243)

The sighash is computed using BLAKE2b-256 with the following data structure:

```rust
struct SighashData {
    header: u32,                    // 0x80000004
    nVersionGroupId: u32,           // 0x892f2085
    hashPrevouts: [u8; 32],
    hashSequence: [u8; 32],
    hashOutputs: [u8; 32],
    hashJoinSplits: [u8; 32],       // zeros for transparent-only
    hashShieldedSpends: [u8; 32],   // zeros for transparent-only
    hashShieldedOutputs: [u8; 32],  // zeros for transparent-only
    nLockTime: u32,                 // 0
    nExpiryHeight: u32,             // 0
    valueBalance: i64,              // 0
    nHashType: u32,                 // 1 (SIGHASH_ALL)
    
    // Input being signed
    prevout: [u8; 36],              // hash + index
    scriptCode: Vec<u8>,            // serialized with length
    amount: u64,
    nSequence: u32,                 // 0xfffffffe
}
```

### BLAKE2b Personalization

```rust
let mut personalization = [0u8; 16];
personalization[..12] = b"ZcashSigHash";
personalization[12..16] = CONSENSUS_BRANCH_ID.to_le_bytes();
```

### Critical Implementation Detail

**DO NOT REVERSE THE SIGHASH BYTES AFTER BLAKE2b COMPUTATION**

Unlike some Bitcoin implementations, BitcoinZ does not reverse the final hash bytes.

## Hash Computations

### hashPrevouts
```rust
// BLAKE2b-256 with personalization "ZcashPrevoutHash"
for each input:
    write prevout_hash (32 bytes)
    write prevout_index (4 bytes, LE)
```

### hashSequence
```rust
// BLAKE2b-256 with personalization "ZcashSequencHash"
for each input:
    write sequence (4 bytes, LE)
```

### hashOutputs
```rust
// BLAKE2b-256 with personalization "ZcashOutputsHash"
for each output:
    write value (8 bytes, LE)
    write script_length (varint)
    write script_bytes
```

## Script Signature Format (P2PKH)

For Pay-to-Public-Key-Hash inputs:
```
[1 byte]   signature_length (including sighash type)
[variable] signature_DER
[1 byte]   sighash_type = 0x01 (SIGHASH_ALL)
[1 byte]   pubkey_length
[variable] public_key (compressed)
```

## Example Transaction

### Transparent-to-Transparent Transaction
```
Transaction ID: 7ca9d30f60cf1e9d3be0cfefbaeccd713cf04b631722d5882e6ce9373cde6065

Raw Transaction (hex):
0400008085202f890157be20808ee8c83521fc06de85661d1573cede4959e7de87a8711642f01e4c8f010000006a47304402202082aba759846d65bfc933f2a4c52751e61c20ae378733d3eaae021760cd265902204273671f638af939ba9b625d200012661017c9f6cc1e62127860c4cb6812f9a9012102083978427b8d47618a11c51174db388d59e2361411528d4ff6cd366582d1a6b0feffffff0280969800000000001976a914053148c04eaba3f7a6836a968ff7c895a3c000b388ac98465d05000000001976a914d7562d15b63a45782a66a8f72fd2de5497b2e49288ac00000000000000000000000000000000000000

Decoded:
- Version: 4 (with Overwinter flag)
- Version Group ID: 0x892f2085
- Inputs: 1
  - Previous Output: 8f4c1ef0421671a887dee75949dece73151d6685de06fc2135c8e88e8020be57:1
  - Script Sig: Standard P2PKH signature
  - Sequence: 0xfffffffe
- Outputs: 2
  - Output 0: 0.1 BTCZ to t1JM4RcuaFKmYxiFj1Zptc3a96EQ5ktHiWD
  - Output 1: 0.8999 BTCZ change to t1dWCXCaMn2tJqUuzxTPRNXfmaLQQVnYPcN
- Lock Time: 0
- Expiry Height: 0
- Value Balance: 0
- No Sapling data
- No binding signature
```

## Implementation Example

```rust
use blake2b_simd::{Params, State};
use byteorder::{LittleEndian, WriteBytesExt};

fn compute_sighash_bitcoinz(
    tx_data: &TransactionData,
    input_index: usize,
) -> [u8; 32] {
    let mut data = Vec::new();
    
    // Write all fields as specified above
    data.write_u32::<LittleEndian>(0x80000004).unwrap();
    data.write_u32::<LittleEndian>(0x892f2085).unwrap();
    // ... continue with all fields
    
    // Create personalization
    let mut personalization = [0u8; 16];
    personalization[..12].copy_from_slice(b"ZcashSigHash");
    personalization[12..16].copy_from_slice(&1991772603u32.to_le_bytes());
    
    // Compute BLAKE2b
    let hash = Params::new()
        .hash_length(32)
        .personal(&personalization)
        .hash(&data);
    
    let mut result = [0u8; 32];
    result.copy_from_slice(hash.as_bytes());
    
    // DO NOT REVERSE!
    result
}
```

## Validation Rules

1. **Version**: Must be 4 with Overwinter flag set
2. **Version Group ID**: Must be exactly `0x892f2085`
3. **Signatures**: Must use correct sighash algorithm
4. **Binding Signature**: Must be absent for transparent-only transactions
5. **Branch ID**: Must use `1991772603` for sighash computation

## Common Errors

1. **bad-txns-sapling-binding-signature-invalid**
   - Cause: Including binding signature on transparent-only transaction
   - Solution: Omit binding signature entirely

2. **mandatory-script-verify-flag-failed**
   - Cause: Incorrect sighash computation
   - Solution: Ensure sighash bytes are not reversed

3. **bad-txns-vin-empty**
   - Cause: No inputs provided
   - Solution: Ensure at least one input is included

## References

- ZIP-243: Transaction Signature Verification for Sapling
- BitcoinZ Protocol Specification
- Zcash Protocol Specification (for base Sapling format)