# BitcoinZ Wallet CLI

A command-line BitcoinZ wallet that connects to a lightwalletd server for fast synchronization.

## Features

- **Fast sync** - Uses lightwalletd protocol for efficient blockchain synchronization
- **Full privacy** - Supports shielded (z-addresses) and transparent addresses
- **HD wallet** - Hierarchical deterministic wallet with seed phrase backup
- **Multi-platform** - Works on Windows, macOS, and Linux
- **Secure** - Optional wallet encryption with password protection

## Installation

### Download Pre-built Binary

Download the latest release for your platform from the [releases page](https://github.com/YOUR_USERNAME/bitcoinz-wallet-cli/releases).

### Build from Source

Requirements:
- Rust 1.56 or later
- Cargo

```bash
git clone https://github.com/YOUR_USERNAME/bitcoinz-wallet-cli
cd bitcoinz-wallet-cli
cargo build --release
```

The binary will be in `target/release/bitcoinz-wallet-cli`

## Usage

### Basic Usage

Start the wallet with default settings (connects to localhost:9067):
```bash
./bitcoinz-wallet-cli
```

### Connect to a Custom Server

```bash
./bitcoinz-wallet-cli --server http://your-server:9067
```

### Common Commands

Once the wallet is running, you can use these commands:

- `help` - Show all available commands
- `addresses` - List all wallet addresses
- `balance` - Show current balance
- `send <address> <amount> [memo]` - Send BitcoinZ
- `seed` - Display seed phrase (for backup)
- `export` - Export private keys
- `list` - Show transaction history
- `shield` - Shield transparent funds
- `quit` - Exit the wallet

### Restore from Seed

```bash
./bitcoinz-wallet-cli --seed "your 24 word seed phrase" --birthday 328500
```

The birthday is the block height when your wallet was created. Use 0 to scan from the beginning.

## Configuration

The wallet stores its data in:
- macOS: `~/Library/Application Support/BitcoinZ/`
- Linux: `~/.bitcoinz/`
- Windows: `%APPDATA%\BitcoinZ\`

Files:
- `bitcoinz-wallet.dat` - Encrypted wallet file
- `bitcoinz-wallet.debug.log` - Debug log file

## Running Your Own Server

To run your own lightwalletd server, see the [BitcoinZ lightwalletd](https://github.com/simbav911/lightwalletd-bitcoinz) repository.

## Security

- **Backup your seed phrase** - This is the only way to recover your wallet
- **Use encryption** - Protect your wallet with a strong password
- **Verify server certificates** - Use trusted lightwalletd servers

## Development

This wallet is based on the lightclient protocol and uses:
- Rust for the core implementation
- gRPC for communication with lightwalletd
- Zcash cryptographic libraries (adapted for BitcoinZ)

## License

MIT License - see [LICENSE](LICENSE) file for details

## Credits

Based on the original Zecwallet Light CLI by Aditya Kulkarni, adapted for BitcoinZ.

## Support

- BitcoinZ Discord: [discord.gg/bitcoinz](https://discord.gg/bitcoinz)
- GitHub Issues: [Create an issue](https://github.com/YOUR_USERNAME/bitcoinz-wallet-cli/issues)
