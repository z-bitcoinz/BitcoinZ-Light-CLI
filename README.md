# BitcoinZ Light CLI

A lightweight command-line BitcoinZ wallet that uses the lightwalletd protocol for fast synchronization without downloading the entire blockchain.

## Overview

BitcoinZ Light CLI is a privacy-preserving wallet that connects to a lightwalletd server to provide:
- Fast blockchain synchronization (minutes instead of days)
- Low storage requirements (< 100MB vs 50GB+ for full node)
- Support for both transparent (t) and shielded (z) addresses
- Complete transaction privacy with z-addresses

## Current Implementation Status

| Transaction Type | Status | Notes |
|-----------------|---------|-------|
| Transparent → Transparent (t→t) | ✅ **Working** | Fully functional on mainnet |
| Transparent → Shielded (t→z) | ✅ **Working** | Shield funds with memo support |
| Shielded → Shielded (z→z) | ✅ **Working** | Private z-to-z transfers with memos |
| Shielded → Transparent (z→t) | ✅ **Working** | Unshield funds to transparent addresses |

## Features

- **Fast sync** - Uses lightwalletd protocol for efficient blockchain synchronization
- **Full privacy** - Supports shielded (z-addresses) and transparent addresses
- **Memo support** - Send encrypted messages with shielded transactions
- **HD wallet** - Hierarchical deterministic wallet with seed phrase backup
- **Multi-platform** - Works on Windows, macOS, and Linux
- **Secure** - Private keys never leave your device

## Installation

### Build from Source

Requirements:
- Rust 1.70 or later (stable channel recommended)
- Cargo
- CMake (for building dependencies)
- Protobuf compiler (protoc)

```bash
git clone https://github.com/z-bitcoinz/BitcoinZ-Light-CLI
cd BitcoinZ-Light-CLI
cargo build --release
```

The binary will be in `target/release/bitcoinz-light-cli`

### Pre-built Binaries

Coming soon - pre-built binaries for major platforms.

## Usage

### Basic Usage

Start the wallet (connects to default server at https://lightd.btcz.rocks:9067):
```bash
./target/release/bitcoinz-light-cli
```

### Connect to a Custom Server

```bash
./target/release/bitcoinz-light-cli --server http://your-server:9067
```

### Common Commands

Once the wallet is running, you can use these commands:

#### Basic Commands
- `help` - Show all available commands
- `addresses` - List all wallet addresses  
- `balance` - Show current balance
- `quit` - Exit the wallet

#### Transaction Commands
- `send <address> <amount_in_zatoshis> "optional_memo"` - Send transaction with optional memo
- `new z` - Generate new z-address
- `new t` - Generate new t-address
- `encryptmessage <z_address> "message"` - Encrypt message for z-address
- `decryptmessage <encrypted_base64>` - Decrypt received message
- `list` - Show transaction history

#### Example Commands
```bash
# Check balance
balance

# Get your addresses
addresses

# Send transaction (amount in zatoshis, 1 BTCZ = 100,000,000 zatoshis)
send t1bzjjWe5gD28AcCW36FVV3t76XnzdAyguw 100000000

# Send with memo to z-address
send zs1k3wanq50ae50lgujv9jkh0p2lq5wn99u8l0j5d4q8tmssv9krrpzcry4xs3jtsceg38qz9ctpnn 50000000 "Hello from BitcoinZ!"

# Encrypt message for z-address
encryptmessage zs1k3wanq50ae50lgujv9jkh0p2lq5wn99u8l0j5d4q8tmssv9krrpzcry4xs3jtsceg38qz9ctpnn "Private message"

# View transaction history
list
```

### Restore from Seed

```bash
./target/release/bitcoinz-light-cli --seed "your 24 word seed phrase" --birthday 328500
```

The birthday is the block height when your wallet was created. Use 0 to scan from the beginning.

## Technical Implementation

This wallet is built specifically for the BitcoinZ network with:

### Key Features
1. **BitcoinZ Network Parameters** - Correct HRP parameters and activation heights
2. **Shielded Transaction Support** - Full z-address functionality with memo encryption
3. **Standard zcash_primitives Builder** - Compatible with BitcoinZ network specifications
4. **Optimized Codebase** - Clean, efficient implementation focused on BitcoinZ

## Configuration

The wallet stores its data in:
- macOS: `~/Library/Application Support/BitcoinZ/`
- Linux: `~/.bitcoinz/`
- Windows: `%APPDATA%\BitcoinZ\`

Files:
- `bitcoinz-wallet.dat` - Encrypted wallet file
- `bitcoinz-wallet.debug.log` - Debug log file

## Security

- **Backup your seed phrase** - This is the only way to recover your wallet
- **Private keys stay local** - Your keys never leave your device
- **Server privacy** - The lightwalletd server cannot see your private keys or shielded transaction details


## License

MIT License - see [LICENSE](LICENSE) file for details


## Support

- GitHub Issues: [Create an issue](https://github.com/z-bitcoinz/BitcoinZ-Light-CLI/issues)
- BitcoinZ Community: [bitcoinz.global](https://bitcoinz.global)