#!/bin/bash

echo "BitcoinZ t→t Transaction Fix Test"
echo "=================================="
echo ""
echo "This test will check if transparent-only transactions"
echo "now use Overwinter (v3) format to bypass the binding signature issue."
echo ""
echo "Test steps:"
echo "1. Connect to BitcoinZ server"
echo "2. Check balance"
echo "3. Send a t→t transaction"
echo "4. Monitor the console output for Overwinter format confirmation"
echo ""
echo "Starting wallet..."
echo ""

# Run the wallet
./target/release/bitcoinz-light-cli --server=http://93.107.37.216:9067