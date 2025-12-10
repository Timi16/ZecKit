# ZecKit

> A Zcash developer toolkit built on Zebra with real blockchain transactions

[![Smoke Test](https://github.com/Supercoolkayy/ZecKit/actions/workflows/smoke-test.yml/badge.svg)](https://github.com/Zecdev/ZecKit/actions/workflows/smoke-test.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

---

## 🚀 Project Status: Milestone 2 Complete

**Current Milestone:** M2 - Real Blockchain Transactions  
**Completion:** ✅ 95% Complete (known limitations documented)

### What Works Now

- ✅ **One-command devnet:** `zecdev up` starts everything
- ✅ **Real blockchain transactions:** Actual ZEC transfers via ZingoLib + pexpect
- ✅ **Auto-mining:** 101+ blocks mined automatically (coinbase maturity)
- ✅ **Faucet API:** REST API for test funds with real on-chain transactions
- ✅ **Backend toggle:** Switch between lightwalletd and Zaino
- ✅ **UA fixtures:** ZIP-316 unified addresses generated
- ✅ **Smoke tests:** 4-5/5 tests passing
- ✅ **Mining address setup:** Automated script to configure correct mining address

### Known Issues

- ⚠️ **Initial setup required:** Must run `setup-mining-address.sh` before first use
- ⚠️ **10-15 minute mining wait:** Required for coinbase maturity (101 blocks)
- ⚠️ **Wallet sync errors:** Upstream zingolib limitation with wallet state management
- ⚠️ **Transparent mining only:** Zebra internal miner limitation (upstream issue)

---

## Quick Start

### Prerequisites

- **OS:** Linux (Ubuntu 22.04+), WSL2, or macOS with Docker Desktop 4.34+
- **Docker:** Engine ≥ 24.x + Compose v2
- **Resources:** 2 CPU cores, 4GB RAM, 5GB disk

### Installation
```bash
# Clone repository
git clone https://github.com/Supercoolkayy/ZecKit.git
cd ZecKit

# Build CLI
cd cli
cargo build --release
cd ..

# IMPORTANT: Setup mining address (first time only)
./scripts/setup-mining-address.sh lwd

# Clear any existing data
docker volume rm zeckit_zebra-data zeckit_lightwalletd-data 2>/dev/null || true

# Start devnet (takes 10-15 minutes for mining)
docker-compose --profile lwd up -d

# Monitor mining progress (wait for 101 blocks)
# Check every minute:
curl -s http://localhost:8232 -X POST -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"1.0","id":"1","method":"getblockcount","params":[]}' | jq .result

# Once 101+ blocks, verify faucet has funds
curl http://localhost:8080/stats

# Run tests
./cli/target/release/zecdev test
```

### Verify It's Working
```bash
# Check service status
curl http://localhost:8080/health

# Get faucet stats (should show balance after 101+ blocks)
curl http://localhost:8080/stats

# Request test funds (real transaction!)
curl -X POST http://localhost:8080/request \
  -H "Content-Type: application/json" \
  -d '{"address": "tmXXXXX...", "amount": 10.0}'
```

---

## CLI Usage

### Start Devnet
```bash
# FIRST TIME: Setup mining address
./scripts/setup-mining-address.sh lwd

# Start with lightwalletd
docker-compose --profile lwd up -d

# OR start with Zaino
./scripts/setup-mining-address.sh zaino
docker-compose --profile zaino up -d

# Stop services
docker-compose --profile lwd down
# or
docker-compose --profile zaino down

# Stop and remove volumes (fresh start)
docker-compose --profile lwd down
docker volume rm zeckit_zebra-data zeckit_lightwalletd-data
```

### Run Tests
```bash
./cli/target/release/zecdev test

# Expected: 4-5 tests passing
# [1/5] Zebra RPC connectivity... ✓ PASS
# [2/5] Faucet health check... ✓ PASS
# [3/5] Faucet stats endpoint... ✓ PASS
# [4/5] Faucet address retrieval... ✓ PASS
# [5/5] Faucet funding request... ✓ PASS or SKIP (timing dependent)
```

---

## Faucet API

### Base URL
```
http://localhost:8080
```

### Endpoints

**Get Statistics**
```bash
curl http://localhost:8080/stats
```

Response:
```json
{
  "current_balance": 1234.56,
  "transparent_balance": 1234.56,
  "orchard_balance": 0.0,
  "faucet_address": "tmXXXXX...",
  "total_requests": 0,
  "uptime": "5m 23s"
}
```

**Get Address**
```bash
curl http://localhost:8080/address
```

**Request Funds**
```bash
curl -X POST http://localhost:8080/request \
  -H "Content-Type: application/json" \
  -d '{"address": "tmXXXXX...", "amount": 10.0}'
```

Response includes real TXID from blockchain:
```json
{
  "success": true,
  "txid": "a1b2c3d4e5f6...",
  "timestamp": "2025-12-10T12:00:00Z",
  "amount": 10.0
}
```

---

## Architecture
```
┌─────────────────────────────────────┐
│         Docker Compose              │
│                                     │
│  ┌──────────┐      ┌──────────┐   │
│  │  Zebra   │◄─────┤  Faucet  │   │
│  │ regtest  │      │  Flask   │   │
│  │  :8232   │      │  :8080   │   │
│  └────┬─────┘      └────┬─────┘   │
│       │                 │          │
│       ▼                 ▼          │
│  ┌──────────┐      ┌──────────┐   │
│  │Lightwald │◄─────┤  Zingo   │   │
│  │  :9067   │      │  Wallet  │   │
│  └──────────┘      └──────────┘   │
└─────────────────────────────────────┘
           ▲
           │
      ┌────┴────┐
      │ zecdev  │  (Rust CLI)
      └─────────┘
```

**Components:**
- **Zebra:** Full node with internal miner (regtest mode)
- **Lightwalletd:** Light client protocol server (gRPC)
- **Zingo Wallet:** Official Zcash wallet (ZingoLib)
- **Faucet:** Python Flask API for test funds (uses pexpect for reliable CLI interaction)
- **CLI:** Rust tool for orchestration

---

## Known Limitations (M2)

### 1. ⚠️ Mining Address Setup Required

**Issue:** Mining rewards must be sent to the faucet wallet's address.

**Solution:** Run the setup script before first use:
```bash
./scripts/setup-mining-address.sh lwd  # For lightwalletd
# or
./scripts/setup-mining-address.sh zaino  # For Zaino
```

**What it does:**
- Extracts the faucet wallet's transparent address
- Updates `docker/configs/zebra.toml` with the correct `miner_address`
- Ensures mining rewards go to the faucet wallet

**When to run:**
- Before first `docker-compose up`
- After switching backends (lwd ↔ zaino)
- After deleting volumes (fresh start)

---

### 2. ⚠️ 10-15 Minute Initial Startup

**Issue:** First run requires mining 101 blocks for coinbase maturity.

**Root Cause:** Zcash consensus rule - coinbase outputs must mature 100 blocks before spending.

**Cannot be optimized:** This is an inherent blockchain requirement.

**Progress Monitoring:**
```bash
# Check block count every minute
curl -s http://localhost:8232 -X POST -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"1.0","id":"1","method":"getblockcount","params":[]}' | jq .result

# Once result >= 101, funds are spendable
```

**Alternative for M3:** Pre-mined blockchain snapshots for faster startup.

---

### 3. ⚠️ Wallet Sync Errors

**Problem:** Wallet may show sync errors or empty balance even with 101+ blocks.

**Error Message:**
```
Sync error: Error: wallet height is more than 100 blocks ahead of best chain height
```

**Root Cause:** Upstream zingolib limitation with wallet state management across container restarts.

**Workaround 1 - Fresh Restart:**
```bash
# Stop everything
docker-compose --profile lwd down

# Remove volumes
docker volume rm zeckit_zebra-data zeckit_lightwalletd-data

# Restart (will mine fresh 101 blocks)
docker-compose --profile lwd up -d
```

**Workaround 2 - Manual Wallet Sync:**
```bash
# Enter zingo-cli
docker exec -it zeckit-zingo-wallet zingo-cli \
  --data-dir /var/zingo \
  --server http://lightwalletd:9067

# At prompt:
sync run
# Wait for completion
balance
# Should show transparent_balance
```

**Planned Fix:** M3 will improve wallet state management and add better sync detection.

---

### 4. ⚠️ Transparent Mining Only

**Issue:** Zebra's internal miner currently requires transparent addresses for coinbase rewards.

**Technical Details:**
- Zcash protocol supports shielded coinbase since Heartwood (2020) via [ZIP-213](https://zips.z.cash/zip-0213)
- Zebra's internal miner implementation for Orchard unified addresses is still in development
- See [Zebra #5929](https://github.com/ZcashFoundation/zebra/issues/5929) for tracking

**Current Configuration:**
```toml
# docker/configs/zebra.toml
[mining]
internal_miner = true
miner_address = "tmXXXXX..."  # Transparent address
```

**Impact:** Mining rewards go to transparent pool. For testing shielded transactions, funds must be manually shielded.

**Planned Fix:** M3 will add automatic shielding workflow or wait for Zebra upstream support.

---

### 5. ⚠️ Shielding Large UTXO Sets Fails

**Problem:** Attempting to shield many small coinbase UTXOs fails with change output error.

**Error Message:**
```
The transaction requires an additional change output of ZatBalance(15000) zatoshis
```

**Root Cause:** Upstream zingolib/zcash_client_backend limitation when building transactions with many inputs.

**Status:** Reported to Zingo Labs team, tracked in [Zebra #10186](https://github.com/ZcashFoundation/zebra/issues/10186)

**Workaround:** Shield smaller amounts at a time (fewer UTXOs per transaction).

**Impact:** Does not block M2 - faucet uses transparent sends which work reliably.

---

## Troubleshooting

### Balance Shows 0.0 After Mining

**Problem:** Faucet stats show `0.0` balance even after 101+ blocks mined.

**Cause:** Mining address doesn't match faucet wallet address.

**Solution:**
```bash
# 1. Stop services
docker-compose --profile lwd down

# 2. Run setup script
./scripts/setup-mining-address.sh lwd

# 3. Remove old blockchain
docker volume rm zeckit_zebra-data zeckit_lightwalletd-data

# 4. Start fresh (will mine to correct address)
docker-compose --profile lwd up -d

# 5. Wait for 101 blocks
```

---

### Wallet Sync Error

**Problem:**
```
Sync error: Error: wallet height is more than 100 blocks ahead of best chain height
```

**Solution:**
```bash
docker-compose --profile lwd down
docker volume rm zeckit_zebra-data zeckit_lightwalletd-data
docker-compose --profile lwd up -d
```

---

### Port Conflicts
```bash
# Check what's using ports
lsof -i :8232
lsof -i :8080
lsof -i :9067

# Or change ports in docker-compose.yml
```

---

### Lightwalletd Takes Long to Start

**Issue:** Lightwalletd container shows "Waiting" status for 5+ minutes.

**Cause:** Initial sync with Zebra takes time. Healthcheck is lenient (300s start period).

**Solution:** Wait for healthcheck to pass. Check logs:
```bash
docker logs zeckit-lightwalletd
# Look for "Starting gRPC server on 0.0.0.0:9067"
```

---

### Zebra Won't Start

**Check logs:**
```bash
docker logs zeckit-zebra
```

**Common issues:**
- Port 8232 already in use
- Insufficient disk space
- Corrupted state database

**Solution:**
```bash
docker volume rm zeckit_zebra-data
docker-compose --profile lwd up -d
```

---

## Documentation

- **[Architecture](specs/architecture.md)** - System design and data flow
- **[Technical Spec](specs/technical-spec.md)** - Implementation details
- **[Acceptance Tests](specs/acceptance-tests.md)** - Test criteria

---

## Roadmap

### ✅ Milestone 1: Foundation (Complete)
- Docker-based Zebra regtest
- CI/CD pipeline
- Health checks

### ✅ Milestone 2: Real Transactions (95% Complete)
- Rust CLI tool (`zecdev`)
- Real blockchain transactions via ZingoLib + pexpect
- Faucet API with balance tracking
- Backend toggle (lightwalletd ↔ Zaino)
- Mining address setup automation
- UA fixture generation
- Smoke tests (4-5/5 passing)

### ⏳ Milestone 3: GitHub Action (Next)
- Pre-mined blockchain snapshots
- Improved wallet state management
- Reusable GitHub Action
- Full E2E golden flows (5/5 tests passing reliably)
- Auto-shielding workflow
- Backend parity testing

---

## Technical Implementation Notes

### Pexpect for Wallet Interaction

The faucet uses `pexpect` (Python pseudo-terminal library) instead of `subprocess` for reliable zingo-cli interaction:

**Why pexpect:**
- Creates real PTY (pseudo-terminal) - zingo-cli detects interactive mode
- Can wait for specific prompts/patterns
- Handles async output properly
- Natural command flow like human typing

**Key implementation:**
```python
# Spawn interactive session
child = pexpect.spawn('docker exec -i zeckit-zingo-wallet zingo-cli ...')

# Wait for prompt
child.expect(r'\(test\) Block:\d+', timeout=90)

# Run commands
child.sendline('sync')
child.expect(r'Sync completed succesfully', timeout=60)

# Check balance
child.sendline('spendable_balance')
child.expect(r'"spendable_balance":\s*(\d+)')
balance = int(child.match.group(1))
```

### Ephemeral Wallet Volumes

Wallet data uses tmpfs (temporary RAM filesystem) for clean state:
```yaml
zingo-wallet:
  tmpfs:
    - /var/zingo:mode=1777,size=512m
```

**Benefits:**
- Fresh wallet on every restart
- No stale state conflicts
- Fast I/O operations

---

## Contributing

Contributions welcome! Please:

1. Fork and create feature branch
2. Test locally: `docker-compose --profile lwd up && ./cli/target/release/zecdev test`
3. Follow code style (Rust: `cargo fmt`, Python: `black`)
4. Open PR with clear description

---

## FAQ

**Q: Are these real blockchain transactions?**  
A: Yes! M2 uses real on-chain transactions via ZingoLib and Zingo wallet, not mocks.

**Q: Can I use this in production?**  
A: No. ZecKit is for development/testing only (regtest mode).

**Q: Why does startup take so long?**  
A: Mining 101 blocks for coinbase maturity takes 10-15 minutes. This is unavoidable (consensus requirement).

**Q: Why do I need to run setup-mining-address.sh?**  
A: To ensure mining rewards go to the faucet wallet's address, not a hardcoded address.

**Q: How do I reset everything?**  
A: `docker-compose --profile lwd down && docker volume rm zeckit_zebra-data zeckit_lightwalletd-data`

**Q: Why use transparent mining address?**  
A: Zebra's internal miner doesn't yet support Orchard unified addresses. This is an upstream limitation being tracked in [Zebra #5929](https://github.com/ZcashFoundation/zebra/issues/5929).

**Q: Can I switch between lightwalletd and Zaino?**  
A: Yes! Stop services, run `./scripts/setup-mining-address.sh [lwd|zaino]`, then start with the new profile.

---

## Support

- **Issues:** [GitHub Issues](https://github.com/Supercoolkayy/ZecKit/issues)
- **Discussions:** [GitHub Discussions](https://github.com/Supercoolkayy/ZecKit/discussions)
- **Community:** [Zcash Forum](https://forum.zcashcommunity.com/)

---

## License

Dual-licensed under MIT OR Apache-2.0

---

## Acknowledgments

**Built by:** Dapps over Apps team

**Thanks to:**
- Zcash Foundation (Zebra)
- Electric Coin Company (lightwalletd)
- Zingo Labs (ZingoLib)
- Zcash community

---

**Last Updated:** December 10, 2025  
**Next:** M3 - GitHub Action & Pre-mined Snapshots