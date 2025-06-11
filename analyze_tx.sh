#!/bin/bash

echo "Analyzing BitcoinZ transaction structure..."
echo ""

# Create a script to analyze the transaction hex
cat > analyze_commands.txt << 'EOF'
# Send a transaction and capture the hex
send t1JM4RcuaFKmYxiFj1Zptc3a96EQ5ktHiWD 10000000 "test" true
quit
EOF

# Run the wallet
./target/release/bitcoinz-light-cli --server=http://93.107.37.216:9067 < analyze_commands.txt 2>&1 | tee transaction_output.log

echo ""
echo "Transaction analysis complete. Check transaction_output.log for details."