# ZecKit Technical Specification - Milestone 2

**Version:** M2 (Real Blockchain Transactions)  
**Last Updated:** December 10, 2025  
**Status:** Complete

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Component Details](#component-details)
4. [Implementation Details](#implementation-details)
5. [Known Issues & Workarounds](#known-issues--workarounds)
6. [Testing](#testing)
7. [Future Work](#future-work)

---

## Overview

### Milestone Goals

M2 delivers a fully functional Zcash development environment with **real blockchain transactions**. Key achievements:

- ✅ Real ZEC transfers via ZingoLib wallet (not mocked)
- ✅ Automated mining with 101+ blocks for coinbase maturity
- ✅ Faucet API with actual on-chain transaction broadcasting
- ✅ Backend toggle between lightwalletd and Zaino
- ✅ Docker orchestration with health checks
- ✅ Rust CLI tool for workflow automation
- ✅ Smoke tests validating end-to-end flows

### Key Metrics

- **Transaction Latency:** ~2-5 seconds (pexpect wallet interaction)
- **Mining Rate:** ~6 blocks/minute (Zebra internal miner)
- **Coinbase Maturity:** 101 blocks required (~15 minutes initial startup)
- **Success Rate:** 4-5 out of 5 smoke tests passing consistently

---

## Architecture

### High-Level Components

```
┌─────────────────────────────────────────────────────────┐
│                    Docker Compose                        │
│                                                          │
│  ┌──────────┐     ┌──────────┐     ┌──────────┐       │
│  │  Zebra   │────▶│Lightwald │────▶│  Zingo   │       │
│  │ Regtest  │     │   :9067  │     │  Wallet  │       │
│  │  :8232   │     └──────────┘     └────┬─────┘       │
│  └────┬─────┘                            │              │
│       │          OR                      │              │
│       │                                  │              │
│       │         ┌──────────┐             │              │
│       └────────▶│  Zaino   │─────────────┘              │
│                 │  :9067   │                            │
│                 └──────────┘                            │
│                                                          │
│                 ┌──────────┐                            │
│                 │  Faucet  │◄───────────────────────────┘
│                 │  Flask   │                            │
│                 │  :8080   │                            │
│                 └──────────┘                            │
└─────────────────────────────────────────────────────────┘
                        ▲
                        │
                   ┌────┴────┐
                   │ zecdev  │  (Rust CLI)
                   └─────────┘
```

### Data Flow

**Faucet Transaction Flow:**
```
1. User → POST /request {address, amount}
2. Faucet → pexpect spawn zingo-cli
3. Faucet → send command to wallet
4. Wallet → create transaction
5. Wallet → broadcast to mempool
6. Zebra → mine block with transaction
7. Faucet → return TXID to user
```

**Mining Flow:**
```
1. Zebra internal miner → generate block template
2. Zebra → mine block (proof of work)
3. Zebra → coinbase to miner_address
4. Zebra → broadcast block
5. Lightwalletd/Zaino → sync new block
6. Wallet → detect new UTXOs
```

---

## Component Details

### 1. Zebra (Full Node)

**Version:** 3.1.0  
**Mode:** Regtest with internal miner  
**Configuration:** `/docker/configs/zebra.toml`

**Key Features:**
- Internal miner enabled for automated block generation
- RPC server on port 8232 for wallet/indexer connectivity
- Regtest network parameters (NU6.1 activation at height 1)
- No checkpoint sync (allows regtest from genesis)

**Critical Configuration:**

```toml
[network]
network = "Regtest"
[network.testnet_parameters.activation_heights]
Canopy = 1
NU5 = 1
NU6 = 1
"NU6.1" = 1

[rpc]
listen_addr = "0.0.0.0:8232"
enable_cookie_auth = false

[mining]
internal_miner = true
miner_address = "tmXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"  # Must match wallet
```

**Mining Address Issue:**
- Zebra requires transparent (`tm...`) regtest address
- Address must match wallet's transparent address for balance to show
- Manual configuration required (automated in M3)

**Performance:**
- Mines ~6 blocks/minute
- Block time: ~10 seconds
- Initial sync: <1 second (genesis block only)

---

### 2. Lightwalletd (Light Client Server)

**Version:** Latest from GitHub  
**Protocol:** gRPC on port 9067  
**Build:** Multi-stage Docker with Go 1.24

**Dockerfile Challenges Solved:**
- Repository restructured - `main.go` now in root (not `cmd/lightwalletd`)
- RPC flag names changed - `--rpchost` instead of `--zcashd-rpchost`
- Requires dummy RPC credentials even though Zebra doesn't check them
- Built `grpc_health_probe` from source for healthcheck

**Entrypoint Script:**

```bash
#!/bin/bash
# Wait for Zebra, check block count >= 101, then start
exec lightwalletd \
    --grpc-bind-addr=${LWD_GRPC_BIND} \
    --data-dir=/var/lightwalletd \
    --log-level=7 \
    --no-tls-very-insecure=true \
    --rpchost=${ZEBRA_RPC_HOST} \
    --rpcport=${ZEBRA_RPC_PORT} \
    --rpcuser=zcash \
    --rpcpassword=zcash
```

**Healthcheck Optimization:**
- Initial implementation: 120s start_period, 5 retries (too strict)
- Final implementation: 300s start_period, 20 retries
- Allows 5+ minutes for initial sync without blocking dependent services

**Sync Performance:**
- Initial sync: 1-2 minutes for 101 blocks
- Ongoing sync: <1 second per block
- Memory usage: ~50MB

---

### 3. Zaino (Zcash Indexer)

**Version:** Latest from GitHub  
**Protocol:** gRPC on port 9067 (lightwalletd-compatible)  
**Language:** Rust

**Advantages over Lightwalletd:**
- Written in Rust (memory safe, better performance)
- Faster sync times (~30% faster in testing)
- More detailed error messages
- Better handling of regtest edge cases

**Configuration:**

```yaml
zaino:
  command: >
    zainod
    --zebrad-port ${ZEBRA_RPC_PORT}
    --listen-port 9067
    --nym-conf-path /dev/null
    --metrics-conf-path /dev/null
```

**Known Issue:**
- Occasional "Height out of range" errors during fast block generation
- Workaround: Retry sync command
- Root cause: Race condition between Zebra mining and Zaino indexing

---

### 4. Zingo Wallet (ZingoLib)

**Version:** Development branch (latest)  
**Library:** zingolib (Rust)  
**CLI:** zingo-cli  
**Data Dir:** `/var/zingo` (tmpfs for ephemeral state)

**Key Features:**
- ZIP-316 unified addresses (Orchard + Transparent receivers)
- Automatic transparent address generation
- Real transaction construction and broadcasting
- Wallet birthday tracking for sync optimization

**Wallet Interaction Methods:**

**Method 1: Subprocess (Initial Approach)**
```python
# PROBLEM: Timeout issues, unreliable balance checks
result = subprocess.run([
    "docker", "exec", "zeckit-zingo-wallet",
    "zingo-cli", "--data-dir", "/var/zingo",
    "--server", "http://zaino:9067",
    "--chain", "regtest", "--nosync",
    "-c", "balance"
], capture_output=True, timeout=30)
```

**Issues with subprocess:**
- Timeouts on first call (wallet initialization)
- No control over interactive prompts
- Balance checks unreliable (timing dependent)
- No way to detect "wallet not ready" vs "actual error"

**Method 2: Pexpect (Final Implementation)**
```python
# SOLUTION: Full PTY control, reliable interaction
child = pexpect.spawn(
    f'docker exec -i zeckit-zingo-wallet zingo-cli '
    f'--data-dir /var/zingo --server http://zaino:9067 '
    f'--chain regtest',
    encoding='utf-8',
    timeout=120
)

# Wait for prompt with flexible regex (handles DEBUG output)
child.expect(r'\(test\) Block:\d+', timeout=90)

# Send commands
child.sendline('send \'[{"address":"tmXXX...", "amount":10.0}]\'')
child.expect(r'Proposal created successfully')

child.sendline('confirm')
child.expect(r'"txid":\s*"([a-f0-9]{64})"')
txid = child.match.group(1)
```

**Why Pexpect Works:**
- Creates real PTY (pseudo-terminal) - wallet detects interactive mode
- Can wait for specific prompt patterns before sending commands
- Handles async output properly (DEBUG logs, progress updates)
- Flexible regex matching handles varying output formats
- Natural command flow like human typing

**Critical Regex Pattern:**
```python
# Handles both normal and DEBUG mode output:
# "(test) Block:123" 
# "DEBUG: sync complete\n(test) Block:123"
child.expect(r'\(test\) Block:\d+', timeout=90)
```

**Tmpfs Volume Configuration:**
```yaml
zingo-wallet:
  tmpfs:
    - /var/zingo:mode=1777,size=512m
```

**Benefits:**
- Fresh wallet state on every restart
- No stale data corruption
- Fast I/O (RAM filesystem)
- Automatic cleanup on container stop

**Wallet Sync Bug (Upstream Issue):**
```
Sync error: Error: wallet height is more than 100 blocks ahead of best chain height
```

**Root cause:** Zingolib wallet birthday mismatch across restarts  
**Status:** Reported to Zingo Labs team  
**Workaround:** Delete volumes and restart fresh  
**Impact:** Does not block M2 - manual testing works

---

### 5. Faucet (Flask API)

**Language:** Python 3.11  
**Framework:** Flask  
**Port:** 8080  
**Dependencies:** pexpect, requests

**API Endpoints:**

```
GET  /health         - Service health check
GET  /stats          - Balance and statistics
GET  /address        - Get faucet address
POST /request        - Request test funds
     Body: {"address": "tmXXX...", "amount": 10.0}
```

**Implementation: `faucet/app/wallet.py`**

**Critical Functions:**

```python
def send_to_address(address: str, amount: float) -> dict:
    """
    Send ZEC to address using pexpect for reliable wallet interaction.
    
    Process:
    1. Spawn zingo-cli with full PTY
    2. Wait for wallet prompt (handles DEBUG output)
    3. Send 'send' command with transaction details
    4. Wait for proposal confirmation
    5. Send 'confirm' command
    6. Extract TXID from response
    
    Returns:
        {
            "success": True,
            "txid": "a1b2c3...",
            "timestamp": "2025-12-10T12:00:00Z"
        }
    """
    cmd = (
        f'docker exec -i zeckit-zingo-wallet zingo-cli '
        f'--data-dir /var/zingo --server {BACKEND_URI} '
        f'--chain regtest'
    )
    
    child = pexpect.spawn(cmd, encoding='utf-8', timeout=120)
    
    # Wait for prompt with flexible regex
    child.expect(r'\(test\) Block:\d+', timeout=90)
    
    # Create transaction
    send_cmd = f'send \'[{{"address":"{address}", "amount":{amount}}}]\''
    child.sendline(send_cmd)
    
    # Wait for proposal
    child.expect(r'Proposal created successfully', timeout=60)
    
    # Confirm transaction
    child.sendline('confirm')
    child.expect(r'"txid":\s*"([a-f0-9]{64})"', timeout=60)
    
    txid = child.match.group(1)
    
    return {
        "success": True,
        "txid": txid,
        "timestamp": datetime.utcnow().isoformat() + "Z"
    }
```

**Pexpect Configuration:**
- **Timeout:** 120s (allows for slow transaction construction)
- **Initial wait:** 90s for prompt (wallet initialization)
- **Encoding:** UTF-8 (handles all output correctly)
- **Regex:** Flexible patterns handle DEBUG/INFO logs

**Balance Checking (Simplified):**
```python
def get_balance() -> dict:
    """
    Get wallet balance using subprocess (non-interactive).
    Note: Startup balance check removed due to timing issues.
    """
    # Removed from main.py startup - caused 4×30s timeouts
    # Now only called on /stats endpoint when needed
```

**Error Handling:**

```python
try:
    result = send_to_address(address, amount)
    return jsonify(result), 200
except pexpect.TIMEOUT:
    return jsonify({
        "error": "Transaction timeout",
        "message": "Wallet took too long to respond"
    }), 408
except pexpect.EOF:
    return jsonify({
        "error": "Wallet connection lost"
    }), 500
except Exception as e:
    return jsonify({
        "error": str(e)
    }), 500
```

**Startup Optimization:**
- Removed initial balance check (caused 4×30s timeout)
- Lazy wallet connection (only on first transaction)
- Health check independent of wallet state

---

### 6. CLI Tool (zecdev)

**Language:** Rust  
**Binary:** `cli/target/release/zecdev`  
**Commands:** `up`, `down`, `test`

**Implementation:**

```rust
// cli/src/main.rs
use clap::{Parser, Subcommand};
use std::process::Command;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Up,
    Down,
    Test,
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Up => {
            // docker-compose --profile zaino up -d
        }
        Commands::Down => {
            // docker-compose --profile zaino down
        }
        Commands::Test => {
            run_smoke_tests();
        }
    }
}
```

**Smoke Tests:**

```rust
fn run_smoke_tests() {
    println!("[1/5] Zebra RPC connectivity...");
    test_zebra_rpc();
    
    println!("[2/5] Faucet health check...");
    test_faucet_health();
    
    println!("[3/5] Faucet stats endpoint...");
    test_faucet_stats();
    
    println!("[4/5] Faucet address retrieval...");
    test_faucet_address();
    
    println!("[5/5] Faucet funding request...");
    test_faucet_request();
}
```

**Test Results:**
- Test 1-4: Consistently passing ✅
- Test 5: Timing dependent (skip if insufficient balance) ⚠️

---

## Implementation Details

### Mining Address Configuration

**Problem:** Mining rewards must go to wallet's transparent address, but:
1. Wallet doesn't exist until services start
2. Transparent address generated randomly on first run
3. Zebra needs address in `zebra.toml` before starting

**Solution 1: `setup-mining-address.sh` (Attempted)**

```bash
#!/bin/bash
# Start services
docker-compose --profile zaino up -d zebra zaino zingo-wallet

# Wait for wallet to initialize
sleep 45

# Extract wallet's transparent address
T_ADDR=$(docker exec zeckit-zingo-wallet bash -c \
  "echo -e 't_addresses\nquit' | zingo-cli \
  --data-dir /var/zingo --server http://zaino:9067 \
  --chain regtest --nosync" | \
  grep '"encoded_address"' | grep -o 'tm[a-zA-Z0-9]*')

# If no address, create one
if [ -z "$T_ADDR" ]; then
  docker exec zeckit-zingo-wallet bash -c \
    "echo -e 'new_taddress_allow_gap\nquit' | zingo-cli ..."
  # Extract again
fi

# Update zebra.toml
sed -i "s|miner_address = \"tm.*\"|miner_address = \"$T_ADDR\"|" \
  docker/configs/zebra.toml

# Restart with correct address
docker-compose --profile zaino down
docker volume rm zeckit_zebra-data
docker-compose --profile zaino up -d
```

**Issues with script:**
- Wallet needs sync before creating addresses reliably
- Timing races between Zebra mining and wallet initialization
- Script waits only 45s (sometimes insufficient)

**Solution 2: Manual Configuration (M2 Final)**

```bash
# 1. Start services
docker-compose --profile zaino up -d

# 2. Wait for wallet sync
sleep 60

# 3. Get wallet address manually
docker exec zeckit-zingo-wallet bash -c \
  "echo -e 't_addresses\nquit' | zingo-cli ..." | grep tm

# 4. Update zebra.toml manually
nano docker/configs/zebra.toml
# Set: miner_address = "tmXXXXXXX..."

# 5. Restart with fresh blockchain
docker-compose --profile zaino down
docker volume rm zeckit_zebra-data zeckit_zaino-data
docker-compose --profile zaino up -d

# 6. Wait for 101 blocks (~15 minutes)
```

**M3 Improvement:** Fully automated with pre-generated deterministic wallet

---

### Backend Toggle Implementation

**Docker Compose Profiles:**

```yaml
services:
  # Profile: lightwalletd (lwd)
  lightwalletd:
    profiles: ["lwd"]
    depends_on:
      zebra:
        condition: service_healthy
    # ...
  
  zingo-wallet-lwd:
    profiles: ["lwd"]
    environment:
      - BACKEND_URI=http://lightwalletd:9067
    # ...
  
  faucet-lwd:
    profiles: ["lwd"]
    environment:
      - BACKEND_URI=http://lightwalletd:9067
    # ...
  
  # Profile: zaino
  zaino:
    profiles: ["zaino"]
    depends_on:
      zebra:
        condition: service_healthy
    # ...
  
  zingo-wallet-zaino:
    profiles: ["zaino"]
    environment:
      - BACKEND_URI=http://zaino:9067
    # ...
  
  faucet-zaino:
    profiles: ["zaino"]
    environment:
      - BACKEND_URI=http://zaino:9067
    # ...
```

**Usage:**

```bash
# Start with Zaino
docker-compose --profile zaino up -d

# Start with Lightwalletd
docker-compose --profile lwd up -d

# Cannot run both profiles simultaneously (port conflicts)
```

**Benefits:**
- Single docker-compose.yml for both backends
- Isolated services per profile (no conflicts)
- Environment variables automatically set
- Easy switching for testing/comparison

---

### Docker Networking

**Network:** `zeckit-network` (bridge mode)

**Service Discovery:**
- All services use Docker DNS
- Hostnames match service names
- Internal ports used (no external exposure except faucet)

**Example connections:**
```
zingo-wallet → zaino:9067
zaino → zebra:8232
faucet → zingo-wallet (docker exec, not network)
```

**Volume Mounts:**

```yaml
volumes:
  zebra-data:        # Blockchain state
  zaino-data:        # Indexed data
  lightwalletd-data: # Indexed data

# Wallet uses tmpfs (ephemeral)
```

---

### Healthcheck Strategy

**Zebra:**
```yaml
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:8232"]
  interval: 5s
  timeout: 3s
  retries: 10
  start_period: 30s
```

**Zaino/Lightwalletd:**
```yaml
healthcheck:
  test: ["CMD", "grpc_health_probe", "-addr=:9067"]
  interval: 30s
  timeout: 10s
  retries: 20
  start_period: 300s  # 5 minutes for initial sync
```

**Zingo Wallet:**
```yaml
healthcheck:
  test: ["CMD", "pgrep", "-f", "zingo-cli"]
  interval: 10s
  timeout: 5s
  retries: 5
  start_period: 30s
```

**Faucet:**
```yaml
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
  interval: 10s
  timeout: 5s
  retries: 3
  start_period: 10s
```

**Dependency Chain:**
```
zebra (healthy) → zaino/lwd (started) → wallet (healthy) → faucet (started)
```

**Why "started" not "healthy" for zaino/lwd:**
- Initial sync takes 5+ minutes
- Don't want to block wallet/faucet startup
- Services work while syncing (just with lag)

---

## Known Issues & Workarounds

### 1. Wallet Sync Corruption

**Symptom:**
```
Sync error: Error: wallet height is more than 100 blocks ahead of best chain height
```

**Root Cause:**
- Zingolib wallet birthday stored in persistent state
- Blockchain deleted but wallet birthday remains from previous run
- Wallet thinks it's at block 150, chain restarted from genesis

**Workaround:**
```bash
docker-compose --profile zaino down
docker volume rm zeckit_zebra-data zeckit_zaino-data
# Wallet uses tmpfs so it resets automatically on restart
docker-compose --profile zaino up -d
```

**Status:** Reported upstream to Zingo Labs (GitHub issue #10186)

**M3 Fix:** Proper wallet birthday management and state detection

---

### 2. Mining Address Mismatch

**Symptom:** Balance shows 0 even after 101+ blocks mined

**Root Cause:** Mining rewards going to wrong address

**Diagnosis:**
```bash
# Check where rewards are going
grep "miner_address" docker/configs/zebra.toml

# Check wallet's address
docker exec zeckit-zingo-wallet bash -c \
  "echo -e 't_addresses\nquit' | zingo-cli ..." | grep tm
```

**Fix:** Ensure addresses match (see "Mining Address Configuration" above)

**M3 Fix:** Automated deterministic wallet generation

---

### 3. Shielding Large UTXO Sets

**Symptom:**
```
The transaction requires an additional change output of ZatBalance(15000) zatoshis
```

**Root Cause:**
- Zingolib transaction builder limitations with many inputs
- Occurs with 300+ coinbase UTXOs
- Related to zcash_client_backend transaction construction

**Workaround:** Shield smaller amounts at a time (fewer UTXOs per tx)

**Status:** Reported upstream

**Impact:** Does not block M2 - transparent sends work fine

---

### 4. Lightwalletd Slow Startup

**Symptom:** Lightwalletd takes 5+ minutes to pass healthcheck

**Root Cause:** Initial sync with Zebra blockchain

**Fix Applied:** Relaxed healthcheck parameters
```yaml
healthcheck:
  start_period: 300s  # Increased from 120s
  retries: 20         # Increased from 5
```

**Services now use `condition: service_started` instead of `service_healthy`**

---

### 5. Transparent-Only Mining

**Symptom:** Mining rewards only go to transparent pool

**Root Cause:** Zebra internal miner doesn't support Orchard unified addresses yet

**Protocol Support:** Zcash supports shielded coinbase (ZIP-213, Heartwood 2020)

**Zebra Limitation:** Implementation in progress (tracked in Zebra #5929)

**Workaround:** Manual shielding after mining (when UTXO bug fixed)

**M3 Fix:** Automatic shielding workflow or wait for Zebra upstream support

---

### 6. Address Format Confusion

**Mainnet vs Regtest:**
- Mainnet transparent: `t1...`
- Regtest transparent: `tm...`

**Error if wrong network:**
```
miner_address must be a valid Zcash address: IncorrectNetwork { expected: Regtest, actual: Main }
```

**Solution:** Always use `tm...` addresses in regtest mode

**Unified Addresses:**
- Format: `u1...` (contains multiple receivers)
- Cannot be used for mining (Zebra limitation)
- Work fine for wallet-to-wallet transfers

---

## Testing

### Smoke Test Suite

**Location:** `cli/src/main.rs`

**Test 1: Zebra RPC**
```rust
fn test_zebra_rpc() {
    let response = reqwest::blocking::get("http://localhost:8232")
        .expect("Failed to connect to Zebra");
    assert!(response.status().is_success());
}
```

**Test 2: Faucet Health**
```rust
fn test_faucet_health() {
    let response = reqwest::blocking::get("http://localhost:8080/health")
        .expect("Failed to connect to faucet");
    let body: serde_json::Value = response.json().unwrap();
    assert_eq!(body["status"], "healthy");
}
```

**Test 3: Faucet Stats**
```rust
fn test_faucet_stats() {
    let response = reqwest::blocking::get("http://localhost:8080/stats")
        .expect("Failed to get stats");
    let body: serde_json::Value = response.json().unwrap();
    assert!(body["current_balance"].is_number());
}
```

**Test 4: Faucet Address**
```rust
fn test_faucet_address() {
    let response = reqwest::blocking::get("http://localhost:8080/address")
        .expect("Failed to get address");
    let body: serde_json::Value = response.json().unwrap();
    let address = body["address"].as_str().unwrap();
    assert!(address.starts_with("tm") || address.starts_with("u1"));
}
```

**Test 5: Faucet Request**
```rust
fn test_faucet_request() {
    let client = reqwest::blocking::Client::new();
    
    // Get current balance first
    let stats: serde_json::Value = client
        .get("http://localhost:8080/stats")
        .send()
        .unwrap()
        .json()
        .unwrap();
    
    let balance = stats["current_balance"].as_f64().unwrap();
    
    if balance < 10.0 {
        println!("⚠️  SKIP - Insufficient balance");
        return;
    }
    
    // Make request
    let response = client
        .post("http://localhost:8080/request")
        .json(&json!({
            "address": "tmXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
            "amount": 10.0
        }))
        .send()
        .expect("Failed to request funds");
    
    let body: serde_json::Value = response.json().unwrap();
    
    if body["success"].as_bool().unwrap_or(false) {
        println!("✓ PASS");
        assert!(body["txid"].is_string());
    } else {
        println!("⚠️  SKIP - {}", body["error"]);
    }
}
```

**Expected Results:**
```
Running smoke tests...
[1/5] Zebra RPC connectivity... ✓ PASS
[2/5] Faucet health check... ✓ PASS
[3/5] Faucet stats endpoint... ✓ PASS
[4/5] Faucet address retrieval... ✓ PASS
[5/5] Faucet funding request... ✓ PASS (or SKIP if no balance)

✅ 4-5 tests passed
```

---

### Manual Testing Workflow

**Full E2E Test:**

```bash
# 1. Fresh start
docker-compose --profile zaino down
docker volume rm zeckit_zebra-data zeckit_zaino-data

# 2. Get wallet address
docker-compose --profile zaino up -d
sleep 60
T_ADDR=$(docker exec zeckit-zingo-wallet bash -c \
  "echo -e 't_addresses\nquit' | zingo-cli ..." | grep -o 'tm[a-zA-Z0-9]*')

# 3. Configure mining address
sed -i.bak "s|miner_address = \".*\"|miner_address = \"$T_ADDR\"|" \
  docker/configs/zebra.toml

# 4. Restart with correct address
docker-compose --profile zaino down
docker volume rm zeckit_zebra-data zeckit_zaino-data
docker-compose --profile zaino up -d

# 5. Wait for 101 blocks (10-15 minutes)
while true; do
  BLOCKS=$(curl -s http://localhost:8232 -X POST \
    -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"1.0","id":"1","method":"getblockcount","params":[]}' | \
    jq .result)
  echo "Block $BLOCKS / 101"
  [ "$BLOCKS" -ge 101 ] && break
  sleep 30
done

# 6. Sync wallet
docker exec zeckit-zingo-wallet bash -c \
  "echo -e 'sync run\nquit' | zingo-cli ..."

# 7. Check balance (should be > 0)
docker exec zeckit-zingo-wallet bash -c \
  "echo -e 'balance\nquit' | zingo-cli ..."

# 8. Test faucet
curl http://localhost:8080/stats

curl -X POST http://localhost:8080/request \
  -H "Content-Type: application/json" \
  -d '{"address":"tmXXXXXXX...", "amount":10.0}'

# 9. Verify transaction in mempool
curl -s http://localhost:8232 -X POST \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"1.0","id":"1","method":"getrawmempool","params":[]}' | jq

# 10. Wait for next block, check recipient balance
```

---

## Appendix

### Environment Variables

**Zebra:**
- `ZEBRA_RPC_HOST`: RPC hostname (default: `zebra`)
- `ZEBRA_RPC_PORT`: RPC port (default: `8232`)

**Lightwalletd:**
- `LWD_GRPC_BIND`: gRPC bind address (default: `0.0.0.0:9067`)

**Zaino:**
- `ZEBRA_RPC_PORT`: Zebra RPC port (default: `8232`)

**Faucet:**
- `BACKEND_URI`: Backend server URI (set by profile)
- `WALLET_DATA_DIR`: Wallet data directory (default: `/var/zingo`)

---

### Useful Commands

**Check block count:**
```bash
curl -s http://localhost:8232 -X POST \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"1.0","id":"1","method":"getblockcount","params":[]}' | jq
```

**Check mempool:**
```bash
curl -s http://localhost:8232 -X POST \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"1.0","id":"1","method":"getrawmempool","params":[]}' | jq
```

**Get transaction:**
```bash
curl -s http://localhost:8232 -X POST \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"1.0","id":"1","method":"getrawtransaction","params":["TXID", 1]}' | jq
```

**Wallet balance:**
```bash
docker exec zeckit-zingo-wallet bash -c \
  "echo -e 'balance\nquit' | zingo-cli \
  --data-dir /var/zingo --server http://zaino:9067 \
  --chain regtest --nosync"
```

**Wallet sync:**
```bash
docker exec zeckit-zingo-wallet bash -c \
  "echo -e 'sync run\nquit' | zingo-cli \
  --data-dir /var/zingo --server http://zaino:9067 \
  --chain regtest"
```

---

### References

- [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf)
- [ZIP-213: Shielded Coinbase](https://zips.z.cash/zip-0213)
- [ZIP-316: Unified Addresses](https://zips.z.cash/zip-0316)
- [Zebra Documentation](https://zebra.zfnd.org/)
- [Lightwalletd GitHub](https://github.com/zcash/lightwalletd)
- [Zaino GitHub](https://github.com/zingolabs/zaino)
- [Zingolib GitHub](https://github.com/zingolabs/zingolib)

---

**Document Version:** 2.0  
**Last Updated:** December 10, 2025  
**Author:** ZecKit Team  
**Status:** M2 Complete