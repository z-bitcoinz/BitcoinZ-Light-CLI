#!/bin/bash

echo "Starting BitcoinZ wallet..."
echo ""

# Create a command file with the operations we want to perform
cat > wallet_commands.txt << EOF
# First check the balance
balance

# List all addresses
addresses

# Wait a moment
sleep 2

# Get the first transparent address
list

# Send 0.1 BTCZ to our own transparent address (adjust the address and amount as needed)
# Format: send <address> <amount> <memo>
# We'll need to update this with the actual address after seeing the list

quit
EOF

# Run the wallet with the commands
./target/release/bitcoinz-light-cli --server=http://93.107.37.216:9067 < wallet_commands.txt