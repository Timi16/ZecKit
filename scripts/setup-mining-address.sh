#!/bin/bash
set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🔧 ZecKit Mining Address Setup${NC}"
echo "================================"

# Get profile (zaino or lwd)
PROFILE=${1:-zaino}
echo -e "${BLUE}📋 Using profile: $PROFILE${NC}"

# Set backend-specific variables
if [ "$PROFILE" = "zaino" ]; then
    BACKEND_SERVICE="zaino"
    WALLET_SERVICE="zingo-wallet-zaino"
    BACKEND_URI="http://zaino:9067"
elif [ "$PROFILE" = "lwd" ]; then
    BACKEND_SERVICE="lightwalletd"
    WALLET_SERVICE="zingo-wallet-lwd"
    BACKEND_URI="http://lightwalletd:9067"
else
    echo -e "${RED}❌ Error: Invalid profile '$PROFILE'. Use 'zaino' or 'lwd'${NC}"
    exit 1
fi

# Start required services
echo -e "${BLUE}📦 Starting required services...${NC}"
docker-compose --profile "$PROFILE" up -d zebra "$BACKEND_SERVICE" "$WALLET_SERVICE"

# Wait for services to initialize
echo -e "${YELLOW}⏳ Waiting for services to initialize...${NC}"
sleep 45

# Try to extract wallet's transparent address
echo "🔍 Extracting wallet transparent address..."
for i in {1..3}; do
  echo "  Attempt $i/3..."
  
  # Try to get existing transparent addresses first
  WALLET_OUTPUT=$(docker exec zeckit-zingo-wallet bash -c \
    "echo -e 't_addresses\nquit' | timeout 15 zingo-cli --data-dir /var/zingo --server $BACKEND_URI --chain regtest --nosync 2>/dev/null" || true)
  
  WALLET_ADDRESS=$(echo "$WALLET_OUTPUT" | grep '"encoded_address"' | grep -o 'tm[a-zA-Z0-9]\{34\}' | head -1)
  
  # If no transparent address exists, create one (force creation even without gap)
  if [ -z "$WALLET_ADDRESS" ]; then
    echo "  📝 No transparent address found, creating one..."
    docker exec zeckit-zingo-wallet bash -c \
      "echo -e 'new_taddress_allow_gap\nquit' | timeout 15 zingo-cli --data-dir /var/zingo --server $BACKEND_URI --chain regtest --nosync 2>/dev/null" >/dev/null || true
    
    sleep 5
    
    # Try again to get the newly created address
    WALLET_OUTPUT=$(docker exec zeckit-zingo-wallet bash -c \
      "echo -e 't_addresses\nquit' | timeout 15 zingo-cli --data-dir /var/zingo --server $BACKEND_URI --chain regtest --nosync 2>/dev/null" || true)
    
    WALLET_ADDRESS=$(echo "$WALLET_OUTPUT" | grep '"encoded_address"' | grep -o 'tm[a-zA-Z0-9]\{34\}' | head -1)
  fi
  
  if [ -n "$WALLET_ADDRESS" ]; then
    echo "  ✅ Found address: $WALLET_ADDRESS"
    break
  fi
  
  echo "  ⏳ Wallet not ready, waiting 20s..."
  sleep 20
done

# Fallback to deterministic address if extraction fails
if [ -z "$WALLET_ADDRESS" ]; then
  echo -e "${YELLOW}⚠️  Could not extract address from wallet${NC}"
  echo -e "${YELLOW}📝 Using deterministic address from zingolib default seed...${NC}"
  WALLET_ADDRESS="tmV8gvQCgovPQ9JwzLVsesLZjuyEEF5STAD"
  echo "  Address: $WALLET_ADDRESS"
fi

echo -e "${GREEN}✅ Using wallet address: $WALLET_ADDRESS${NC}"

# Stop services
echo -e "${BLUE}🛑 Stopping services...${NC}"
docker-compose --profile "$PROFILE" down

# Update zebra.toml with the wallet address
echo -e "${BLUE}📝 Updating zebra.toml...${NC}"
sed -i.bak "s|miner_address = \"tm[a-zA-Z0-9]\{34\}\"|miner_address = \"$WALLET_ADDRESS\"|" docker/configs/zebra.toml
echo -e "${GREEN}✅ Mining address updated in zebra.toml${NC}"

# Show updated config section
echo ""
echo -e "${BLUE}📋 Updated Zebra mining config:${NC}"
grep -A 2 "\[mining\]" docker/configs/zebra.toml
echo ""

# Success message
echo -e "${GREEN}🎉 Setup complete!${NC}"
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Next steps:${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
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
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}Mining rewards will go to: $WALLET_ADDRESS${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
