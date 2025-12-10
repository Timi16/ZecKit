#!/bin/bash
set -e

echo "🔧 ZecKit Mining Address Setup"
echo "================================"

# Detect which profile to use
PROFILE=${1:-zaino}
echo "📋 Using profile: $PROFILE"

# Service names vary by profile
if [ "$PROFILE" = "zaino" ]; then
  WALLET_SERVICE="zingo-wallet-zaino"
  BACKEND_SERVICE="zaino"
  BACKEND_URI="http://zaino:9067"
elif [ "$PROFILE" = "lwd" ]; then
  WALLET_SERVICE="zingo-wallet-lwd"
  BACKEND_SERVICE="lightwalletd"
  BACKEND_URI="http://lightwalletd:9067"
else
  echo "❌ Invalid profile. Use 'zaino' or 'lwd'"
  exit 1
fi

# 1. Start minimal services (zebra + backend + wallet)
echo "📦 Starting required services..."
docker-compose --profile "$PROFILE" up -d zebra "$BACKEND_SERVICE" "$WALLET_SERVICE"

# Wait for services to be ready
echo "⏳ Waiting for services to initialize..."
sleep 45

# 2. Get wallet's transparent address - try multiple methods
echo "🔍 Extracting wallet transparent address..."

WALLET_ADDRESS=""

# Method 1: Direct command with --nosync
for i in {1..3}; do
  echo "  Attempt $i/3..."
  
  WALLET_OUTPUT=$(docker exec zeckit-zingo-wallet bash -c \
    "echo 'addresses' | timeout 10 zingo-cli --data-dir /var/zingo --server $BACKEND_URI --chain regtest --nosync 2>/dev/null" || true)
  
  WALLET_ADDRESS=$(echo "$WALLET_OUTPUT" | grep -o 't2[a-zA-Z0-9]\{34\}' | head -1)
  
  if [ -n "$WALLET_ADDRESS" ]; then
    echo "  ✅ Found address: $WALLET_ADDRESS"
    break
  fi
  
  echo "  ⏳ Wallet not ready, waiting 20s..."
  sleep 20
done

# Method 2: Use deterministic address from zingolib default seed
if [ -z "$WALLET_ADDRESS" ]; then
  echo "⚠️  Could not extract address from wallet"
  echo "📝 Using deterministic address from zingolib default seed..."
  WALLET_ADDRESS="tmV8gvQCgovPQ9JwzLVsesLZjuyEEF5STAD"
  echo "  Address: $WALLET_ADDRESS"
fi

if [ -z "$WALLET_ADDRESS" ]; then
  echo "❌ Failed to get wallet address!"
  exit 1
fi

echo "✅ Using wallet address: $WALLET_ADDRESS"

# 3. Stop services before updating config
echo "🛑 Stopping services..."
docker-compose --profile "$PROFILE" down

# 4. Update zebra.toml with wallet's address
echo "📝 Updating zebra.toml..."
ZEBRA_CONFIG="docker/configs/zebra.toml"

# Backup original
cp "$ZEBRA_CONFIG" "${ZEBRA_CONFIG}.bak"

# Replace mining address (macOS and Linux compatible)
if [[ "$OSTYPE" == "darwin"* ]]; then
  # macOS sed
  sed -i '' "s/miner_address = \"t2[a-zA-Z0-9]\{34\}\"/miner_address = \"$WALLET_ADDRESS\"/" "$ZEBRA_CONFIG"
else
  # Linux sed
  sed -i "s/miner_address = \"t2[a-zA-Z0-9]\{34\}\"/miner_address = \"$WALLET_ADDRESS\"/" "$ZEBRA_CONFIG"
fi

echo "✅ Mining address updated in zebra.toml"

# 5. Show the change
echo ""
echo "📋 Updated Zebra mining config:"
grep -A 2 "\[mining\]" "$ZEBRA_CONFIG"

echo ""
echo "🎉 Setup complete!"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Next steps:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "1. Clear old blockchain data:"
echo "   docker volume rm zeckit_zebra-data 2>/dev/null || true"
echo ""
echo "2. Start services with mining to correct address:"
echo "   docker-compose --profile $PROFILE up -d"
echo ""
echo "3. Monitor mining progress:"
echo "   while true; do"
echo "     BLOCKS=\$(curl -s http://localhost:8232 -X POST -H 'Content-Type: application/json' \\"
echo "       -d '{\"jsonrpc\":\"1.0\",\"id\":\"1\",\"method\":\"getblockcount\",\"params\":[]}' 2>/dev/null | grep -o '\"result\":[0-9]*' | cut -d: -f2)"
echo "     echo \"\$(date +%H:%M:%S): Block \$BLOCKS / 101\""
echo "     [ \"\$BLOCKS\" -ge 101 ] && echo \"✅ Mining complete!\" && break"
echo "     sleep 10"
echo "   done"
echo ""
echo "4. Check faucet balance:"
echo "   curl http://localhost:8080/stats"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Mining rewards will go to: $WALLET_ADDRESS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""