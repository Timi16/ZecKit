# ZecKit

> A Linux-first toolkit for Zcash development on Zebra

[![Smoke Test](https://github.com/Supercoolkayy/ZecKit/actions/workflows/smoke-test.yml/badge.svg)](https://github.com/Supercoolkayy/ZecKit/actions/workflows/smoke-test.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

---

## 🚀 Project Status: Milestone 2 - Developer Toolkit

**Current Milestone:** M2 - CLI Tool + Faucet + UA Fixtures  
**Completion:** ✅ Complete

### What Works Now (M1 + M2)
- ✅ **Zebra regtest node** in Docker with health checks
- ✅ **CLI tool** (`zecdev up/test/down/status`)
- ✅ **Faucet service** with 1000 ZEC auto-funded
- ✅ **UA fixtures** for ZIP-316 testing (pre-funded)
- ✅ **Backend toggle** (lightwalletd/Zaino)
- ✅ **Smoke tests** (5/5 passing)
- ✅ **CI pipeline** with self-hosted runner
- ✅ **Balance tracking** with dual-history accounting

### Coming in Future Milestones
- ⏳ M3: Real blockchain transactions + GitHub Action + E2E golden flows
- ⏳ M4: Comprehensive documentation + Video tutorials
- ⏳ M5: 90-day maintenance window + Community handover

---

## 📋 Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [CLI Usage](#cli-usage)
- [Faucet API](#faucet-api)
- [UA Fixtures](#ua-fixtures)
- [Project Goals](#project-goals)
- [Architecture](#architecture)
- [Development](#development)
- [CI/CD](#cicd)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

---

## Overview

**ZecKit** is a developer-first toolkit that provides a fast, reliable, and unified environment for building on Zebra, the new Zcash node implementation replacing zcashd.

ZecKit standardizes the post-zcashd testing workflow by delivering a one-command devnet with pre-mined funds, UA fixtures, and CI-ready smoke tests.

### Key Features (M1 + M2)

- ✅ **One-Command Devnet:** `zecdev up` starts Zebra + faucet + backend
- ✅ **Auto-Funded Faucet:** 1000 ZEC available immediately
- ✅ **UA Fixtures:** ZIP-316 test addresses pre-funded and ready
- ✅ **Backend Toggle:** Switch between lightwalletd and Zaino
- ✅ **Built-in Tests:** 5 smoke tests covering all critical paths
- ✅ **Health Monitoring:** Automated service readiness checks
- ✅ **CI Integration:** GitHub Actions on self-hosted runner
- ✅ **Linux-First:** Optimized for Linux/WSL environments

---

## Quick Start

### Prerequisites

- **OS:** Linux (Ubuntu 22.04+), WSL, or macOS/Windows with Docker Desktop 4.34+
- **Docker:** Engine ≥ 24.x + Compose v2
- **Resources:** 2 CPU cores, 4GB RAM, 5GB disk
- **Rust:** 1.70+ (for building CLI from source)

### Installation

#### Option 1: Quick Start (Prebuilt CLI)

```bash
# Clone the repository
git clone https://github.com/Supercoolkayy/ZecKit.git
cd ZecKit

# Download prebuilt CLI (Linux x86_64)
# TODO: Add release download link once published

# Or build from source
cd cli
cargo build --release
cd ..

# Start devnet with lightwalletd backend
./cli/target/release/zecdev up --backend=lwd

# Wait for services to be ready (~30 seconds)
# ✓ All services ready!

# Run smoke tests
./cli/target/release/zecdev test

# ✓ Tests passed: 5/5
```

#### Option 2: Manual Docker Setup

```bash
# Clone and setup
git clone https://github.com/Supercoolkayy/ZecKit.git
cd ZecKit

# Start with lightwalletd
docker compose --profile lwd up -d

# Check services are ready
docker compose ps

# Test manually
curl http://127.0.0.1:8080/stats
```

### Verify It's Working

```bash
# Check service status
./cli/target/release/zecdev status

# Check faucet stats
curl http://127.0.0.1:8080/stats

# Get UA fixtures
curl http://127.0.0.1:8080/fixtures

# Request test funds
curl -X POST http://127.0.0.1:8080/request \
  -H "Content-Type: application/json" \
  -d '{"address": "tmBsTi2xWTjUdEXnuTceL7fecEQKeWu4u6d", "amount": 10}'
```

### Shutdown

```bash
# Stop services
./cli/target/release/zecdev down

# Remove volumes (fresh start next time)
./cli/target/release/zecdev down --purge
```

---

## CLI Usage

The `zecdev` CLI provides a simple interface for managing your devnet.

### Commands

#### `zecdev up`

Start the devnet with all services.

```bash
# Start with lightwalletd backend
zecdev up --backend=lwd

# Start with Zaino backend (experimental)
zecdev up --backend=zaino

# Start with no backend (Zebra + faucet only)
zecdev up --backend=none

# Force fresh start (removes volumes)
zecdev up --fresh
```

**What it does:**
- Starts Zebra (regtest mode)
- Starts faucet with 1000 ZEC auto-funded
- Starts selected backend (lightwalletd or Zaino)
- Runs health checks on all services
- Generates and pre-funds UA fixtures

#### `zecdev test`

Run comprehensive smoke tests.

```bash
zecdev test
```

**Tests performed:**
1. Zebra RPC connectivity
2. Faucet health check
3. Faucet stats endpoint
4. Faucet address retrieval
5. Faucet funding request (basic shielded send)

#### `zecdev status`

Check service status and health.

```bash
zecdev status
```

**Shows:**
- Container status (running/stopped)
- Zebra block height
- Faucet balance
- Service health indicators

#### `zecdev down`

Stop the devnet.

```bash
# Stop services (keeps volumes)
zecdev down

# Stop and remove volumes (clean slate)
zecdev down --purge
```

### Windows Usage

On Windows, use PowerShell and include the full path:

```powershell
# Navigate to CLI directory
cd C:\Users\USER\Documents\ZecKit\cli

# Run commands
.\target\release\zecdev.exe up --backend=lwd
.\target\release\zecdev.exe test
.\target\release\zecdev.exe status
.\target\release\zecdev.exe down
```

---

## Faucet API

The faucet provides test funds for development. It auto-funds with 1000 ZEC on startup.

### Base URL

```
http://127.0.0.1:8080
```

### Endpoints

#### `GET /stats`

Get faucet statistics.

**Response:**
```json
{
  "faucet_address": "tmBsTi2xWTjUdEXnuTceL7fecEQKeWu4u6d",
  "current_balance": 1197.0,
  "total_funded": 1300.0,
  "total_sent": 103.0,
  "total_requests": 4,
  "uptime": "1m 14s",
  "version": "0.1.0"
}
```

#### `GET /fixtures`

Get UA test fixtures (ZIP-316 addresses).

**Response:**
```json
{
  "generated_at": "2025-11-18T07:27:02.584236Z",
  "all_fixtures": [
    {
      "address": "tmBsTi2xWTjUdEXnuTceL7fecEQKeWu4u6d",
      "address_type": "transparent",
      "name": "transparent_fallback",
      "pre_funded": true,
      "pre_fund_amount": 100.0,
      "receivers": ["transparent"]
    }
  ],
  "transparent_addresses": [...],
  "sapling_addresses": [],
  "unified_addresses": []
}
```

#### `GET /health`

Service health check.

**Response:**
```json
{
  "status": "healthy",
  "zebra_connected": true,
  "zebra_height": 1,
  "wallet_loaded": true,
  "balance": 1197.0
}
```

#### `GET /address`

Get faucet address.

**Response:**
```json
{
  "address": "tmBsTi2xWTjUdEXnuTceL7fecEQKeWu4u6d",
  "balance": 1197.0
}
```

#### `POST /request`

Request test funds.

**Request:**
```json
{
  "address": "tmBsTi2xWTjUdEXnuTceL7fecEQKeWu4u6d",
  "amount": 10.0
}
```

**Response:**
```json
{
  "txid": "48cebb65dc301a4f36f73c72682fef3e6d4d5202e6e484eaccf59e9741430cbd",
  "address": "tmBsTi2xWTjUdEXnuTceL7fecEQKeWu4u6d",
  "amount": 10.0,
  "status": "sent",
  "timestamp": "2025-11-18T03:35:58.224962Z",
  "new_balance": 1188.0
}
```

**Limits:**
- Min: 1.0 ZEC
- Max: 100.0 ZEC
- Default: 10.0 ZEC

#### `GET /history`

Get transaction history.

**Query Parameters:**
- `limit` - Max transactions to return (1-1000, default: 100)

**Response:**
```json
{
  "count": 10,
  "limit": 100,
  "transactions": [
    {
      "txid": "48cebb...",
      "type": "spending",
      "to_address": "tmBs...",
      "amount": 10.0,
      "timestamp": "2025-11-18T03:35:58Z"
    }
  ]
}
```

#### `POST /admin/add-funds` (Dev Only)

Manually add funds to faucet.

**Request:**
```json
{
  "amount": 1000.0,
  "secret": "dev-secret-change-in-production"
}
```

**Response:**
```json
{
  "success": true,
  "amount_added": 1000.0,
  "new_balance": 2197.0
}
```

---

## UA Fixtures

ZecKit automatically generates Unified Address (ZIP-316) test fixtures for E2E testing.

### What Are UA Fixtures?

Test addresses that are:
- ✅ Generated on first startup
- ✅ Pre-funded with 100 ZEC each
- ✅ Available via `/fixtures` API
- ✅ Ready for immediate testing

### Fixture Types

**M2 Implementation:**
- Transparent addresses (always available)

**M3 Will Add:**
- Full Unified Addresses (transparent + sapling + orchard)
- Standalone Sapling addresses
- Backend-specific test vectors

### Why Only Transparent in M2?

Zebra doesn't have wallet RPCs to generate Sapling or Unified addresses directly. M2 uses transparent addresses as test fixtures, which is sufficient for infrastructure validation.

M3 will integrate zcashd to generate full UAs with all receiver types.

### Using Fixtures in Tests

```bash
# Get all fixtures
curl http://127.0.0.1:8080/fixtures

# Use in your tests
FIXTURE_ADDR=$(curl -s http://127.0.0.1:8080/fixtures | jq -r '.all_fixtures[0].address')

# Fund the fixture
curl -X POST http://127.0.0.1:8080/request \
  -H "Content-Type: application/json" \
  -d "{\"address\": \"$FIXTURE_ADDR\", \"amount\": 50}"
```

---

## Project Goals

ZecKit aims to solve the critical gap left by zcashd deprecation:

1. **Standardize Zebra Development:** One consistent way to run Zebra + light-client backends locally and in CI
2. **Enable UA-Centric Testing:** Built-in support for Unified Address (ZIP-316) workflows
3. **Support Backend Parity:** Toggle between lightwalletd and Zaino without changing tests
4. **Catch Breakage Early:** Automated E2E tests in CI before code reaches users

### Why This Matters

- **Zcash is migrating** from zcashd to Zebra (official deprecation in 2025)
- **Teams lack tooling** - No standard, maintained devnet + CI setup
- **Fragmented workflows** - Teams rebuild the same plumbing independently
- **ZecKit standardizes** - The exact workflow builders need, productized

---

## Architecture

### High-Level (M1 + M2)

```
┌─────────────────────────────────────────────────────────┐
│                  Docker Compose                          │
│                                                          │
│  ┌─────────────┐      ┌─────────────┐                  │
│  │   Zebra     │◄─────┤   Faucet    │                  │
│  │  (regtest)  │      │  (Flask)    │                  │
│  │   :8232     │      │   :8080     │                  │
│  └─────────────┘      └─────────────┘                  │
│         │                     │                         │
│         │                     │                         │
│         ▼                     ▼                         │
│  ┌──────────────────────────────────┐                  │
│  │     Backend (Profile-based)      │                  │
│  │                                   │                  │
│  │  • lightwalletd :9067 (lwd)      │                  │
│  │  • Zaino :9067 (zaino)           │                  │
│  └──────────────────────────────────┘                  │
│                                                          │
│  Volumes:                                               │
│  - zebra-data (blockchain state)                        │
│  - faucet-data (wallet + fixtures)                      │
│  - lightwalletd-data (cache)                            │
└─────────────────────────────────────────────────────────┘
                         ▲
                         │
                    ┌────┴────┐
                    │  zecdev │  (CLI)
                    └─────────┘
```

### Components

#### Zebra Node
- Core Zcash regtest node
- RPC enabled on port 8232
- Health checks for readiness
- Version pinned (1.9.0)

#### Faucet Service
- Flask-based REST API
- Auto-funds with 1000 ZEC on startup
- Generates and pre-funds UA fixtures
- Tracks balance with dual-history
- Mock transactions (M2), real transactions (M3)

#### Backend Services
- **lightwalletd** - Stable light client backend
- **Zaino** - Experimental Rust indexer
- Profile-based activation (`--profile lwd`)

#### CLI Tool
- Rust-based command-line interface
- Orchestrates Docker Compose
- Runs health checks and smoke tests
- Linux-first (Windows/macOS best-effort)

---

## Development

### Repository Structure

```
ZecKit/
├── cli/                  # Rust CLI tool
│   ├── src/
│   │   ├── commands/     # up/down/test/status
│   │   ├── docker/       # Docker Compose wrapper
│   │   └── main.rs
│   └── Cargo.toml
├── faucet/               # Python faucet service
│   ├── app/
│   │   ├── routes/       # API endpoints
│   │   ├── main.py       # Flask app factory
│   │   ├── wallet.py     # Balance tracking
│   │   └── ua_fixtures.py # UA fixture manager
│   ├── Dockerfile
│   └── requirements.txt
├── docker/
│   ├── configs/          # Zebra/lightwalletd configs
│   └── healthchecks/     # Health check scripts
├── specs/                # Technical specs
├── tests/
│   └── smoke/            # Smoke test suite
├── scripts/              # Helper scripts
├── .github/workflows/    # CI configuration
└── docker-compose.yml    # Main compose file
```

### Common Tasks

#### Build CLI

```bash
cd cli
cargo build --release
```

#### Run Faucet Locally

```bash
cd faucet
pip install -r requirements.txt
ZEBRA_RPC_URL=http://127.0.0.1:8232 python -m app.main
```

#### Manual Docker Commands

```bash
# Start with lightwalletd
docker compose --profile lwd up -d

# Start with Zaino (experimental)
docker compose --profile zaino up -d

# View logs
docker compose logs -f faucet
docker compose logs -f zebra

# Exec into container
docker exec -it zecdev-faucet bash

# Restart service
docker compose restart faucet

# Rebuild after changes
docker compose build faucet
docker compose up -d --force-recreate faucet
```

#### Manual API Testing

```bash
# Check faucet health
curl http://127.0.0.1:8080/health

# Get stats
curl http://127.0.0.1:8080/stats | jq

# Get fixtures
curl http://127.0.0.1:8080/fixtures | jq

# Request funds
curl -X POST http://127.0.0.1:8080/request \
  -H "Content-Type: application/json" \
  -d '{"address": "tmBsTi2xWTjUdEXnuTceL7fecEQKeWu4u6d", "amount": 10}' | jq
```

#### Test Zebra RPC Directly

```bash
# Using helper script
./scripts/test-zebra-rpc.sh

# Manual RPC call
curl -X POST http://127.0.0.1:8232 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":"test","method":"getblockcount","params":[]}'
```

---

## CI/CD

### GitHub Actions Setup

ZecKit uses a **self-hosted runner** for CI (recommended on WSL/Linux).

#### Setup Self-Hosted Runner

```bash
# Automated setup (WSL/Linux)
./scripts/setup-wsl-runner.sh

# Manual setup:
# 1. Go to: Settings → Actions → Runners
# 2. Click: New self-hosted runner
# 3. Select: Linux
# 4. Follow instructions
```

### Workflows

#### Smoke Test Workflow

Runs on:
- Push to `main` branch
- Pull requests to `main`
- Manual dispatch

**What it does:**
1. Checks out code
2. Starts devnet with `zecdev up`
3. Runs smoke tests with `zecdev test`
4. Uploads logs as artifacts
5. Stops devnet with `zecdev down`

See [.github/workflows/smoke-test.yml](.github/workflows/smoke-test.yml)

---

## Troubleshooting

### Services Won't Start

```bash
# Check Docker is running
docker --version
docker compose version

# Check logs
docker compose logs zebra
docker compose logs faucet

# Force clean restart
docker compose down -v
docker compose build faucet
docker compose up -d
```

### Tests Failing

```bash
# Check service health manually
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8232 -X POST \
  -d '{"jsonrpc":"2.0","id":"test","method":"getblockcount","params":[]}'

# View test logs
./cli/target/release/zecdev test 2>&1 | tee test.log

# Increase timeout (in health.rs)
# backend_max_retries: 120 (4 minutes)
```

### Faucet Balance Issues

```bash
# Check current balance
curl http://127.0.0.1:8080/stats | jq '.current_balance'

# Add funds manually (dev only)
curl -X POST http://127.0.0.1:8080/admin/add-funds \
  -H "Content-Type: application/json" \
  -d '{"amount": 1000, "secret": "dev-secret-change-in-production"}'

# Reset wallet (removes volumes)
docker compose down -v
docker compose up -d
```

### Windows-Specific Issues

```bash
# Path issues - always use full paths
cd C:\Users\USER\Documents\ZecKit\cli
.\target\release\zecdev.exe up

# Docker Desktop networking
# Make sure Docker Desktop 4.34+ is installed
# Enable "Use kernel networking for UDP" in settings

# PowerShell execution policy
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### Port Conflicts

```bash
# Check what's using ports
# Linux/macOS
sudo lsof -i :8232
sudo lsof -i :8080
sudo lsof -i :9067

# Windows
netstat -ano | findstr :8232
netstat -ano | findstr :8080
netstat -ano | findstr :9067

# Change ports in docker-compose.yml if needed
```

---

## Roadmap

### ✅ Milestone 1: Foundation (Complete)
- Repository structure and CI
- Zebra regtest in Docker
- Health checks & smoke tests
- Self-hosted GitHub Actions runner

### ✅ Milestone 2: CLI Tool (Complete)
- `zecdev` command-line tool
- Python Flask faucet with auto-funding
- UA fixtures generation and pre-funding
- Backend toggle (lightwalletd/Zaino)
- Comprehensive smoke tests (5/5 passing)
- Balance tracking with dual-history

### ⏳ Milestone 3: GitHub Action (Next)
- Reusable Action for other repos
- Real blockchain transactions (zcashd integration)
- Full E2E golden flows:
  - Generate UA → fund → autoshield
  - Shielded send with memos
  - Rescan/sync edge cases
- Backend parity testing (lightwalletd ↔ Zaino)
- Sample repository with CI integration

### ⏳ Milestone 4: Documentation
- Quickstart guides (2-minute local, 5-line CI)
- Video tutorials and demos
- Troubleshooting documentation
- Compatibility matrix (Zebra/NU + backends)
- Security best practices

### ⏳ Milestone 5: Maintenance
- 90-day support window
- Version pin updates (Zebra/backends)
- Bug fixes & compatibility patches
- Monthly status reports
- Community handover plan

---

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Quick Guidelines

- **Branch:** Create feature branches from `main`
- **Commits:** Use conventional commits (`feat:`, `fix:`, `docs:`)
- **Tests:** Ensure smoke tests pass (`zecdev test`)
- **Style:** Follow existing code style (Rust: `cargo fmt`, Python: `black`)
- **Documentation:** Update docs for new features

### Development Workflow

1. Fork and clone the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make changes and test locally:
   ```bash
   zecdev up --backend=lwd
   zecdev test
   ```
4. Commit and push: `git push origin feature/my-feature`
5. Open a Pull Request with clear description

---

## Documentation

- [Architecture](specs/architecture.md) - System design and components
- [Technical Spec](specs/technical-spec.md) - Implementation details
- [Acceptance Tests](specs/acceptance-tests.md) - Test criteria
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribution guidelines
- [SECURITY.md](SECURITY.md) - Security policy
- [Demo Script](docs/demo-script.md) - Presentation guide

---

## FAQ

### Why only transparent addresses in UA fixtures?

Zebra doesn't have wallet RPCs to generate Sapling or Unified addresses. M2 uses transparent addresses as test fixtures, which is sufficient for infrastructure validation. M3 will integrate zcashd for full UA generation.

### Are these real blockchain transactions?

Not in M2. The faucet uses simulated transactions for smoke testing. M3 will add real on-chain transactions using zcashd for full E2E testing.

### Can I use this in production?

ZecKit is designed for **development and testing only**. It runs in regtest mode with mock/test transactions. Never use regtest configurations or test keys in production.

### What's the difference between lightwalletd and Zaino?

- **lightwalletd** - Stable, widely used light client backend written in Go
- **Zaino** - Experimental Rust indexer from ZingoLabs with emerging API

ZecKit supports both so teams can test backend migrations without losing coverage.

### Does this work on Windows/macOS?

Yes, but **Linux is the reference platform**. Windows/macOS work with Docker Desktop 4.34+ but may have networking quirks. CI predominantly runs on Linux.

### How do I get help?

- **Issues:** [GitHub Issues](https://github.com/Supercoolkayy/ZecKit/issues)
- **Discussions:** [GitHub Discussions](https://github.com/Supercoolkayy/ZecKit/discussions)
- **Community:** [Zcash Community Forum](https://forum.zcashcommunity.com/)

---

## License

Dual-licensed under your choice of:

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

---

## Acknowledgments

**Built by:** Dapps over Apps team

**Special thanks to:**
- Zcash Foundation (Zebra development)
- Electric Coin Company (Zcash protocol & lightwalletd)
- Zingo Labs (Zaino indexer)
- Zcash community (Testing & feedback)

---

## Support

- **Issues:** [GitHub Issues](https://github.com/Supercoolkayy/ZecKit/issues)
- **Discussions:** [GitHub Discussions](https://github.com/Supercoolkayy/ZecKit/discussions)
- **Community:** [Zcash Community Forum](https://forum.zcashcommunity.com/)
- **Grant:** [Zcash Community Grants](https://zcashcommunitygrants.org/)

---

**Status:** ✅ Milestone 2 Complete - Developer Toolkit Ready  
**Last Updated:** November 18, 2025  
**Next Milestone:** M3 - Real Transactions & GitHub Action