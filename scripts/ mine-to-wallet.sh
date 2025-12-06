#!/bin/bash
WALLET_ADDR=$1
BLOCKS=${2:-110}

echo "⛏️ Mining $BLOCKS blocks to $WALLET_ADDR..."

for i in $(seq 1 $BLOCKS); do
  curl -s -u zcashrpc:notsecure \
    -d "{\"method\":\"generatetoaddress\",\"params\":[1,\"$WALLET_ADDR\"]}" \
    -H 'content-type: text/plain;' \
    http://127.0.0.1:8232/ > /dev/null
  
  if [ $((i % 10)) -eq 0 ]; then
    echo "  Mined $i/$BLOCKS blocks..."
  fi
  sleep 0.1
done

echo "✅ Mining complete!"