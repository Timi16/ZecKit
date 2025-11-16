# ZecDev CLI

Command-line tool for managing ZecKit development environment.

## Installation

### From Source

```bash
cd cli
cargo build --release
```

The binary will be at `target/release/zecdev` (or `zecdev.exe` on Windows).

### Add to PATH

**Linux/macOS:**
```bash
sudo cp target/release/zecdev /usr/local/bin/
```

**Windows (PowerShell as Admin):**
```powershell
copy target\release\zecdev.exe C:\Windows\System32\
```

## Usage

### Start Devnet

```bash
# Start Zebra + Faucet only
zecdev up

# Start with lightwalletd
zecdev up --backend lwd

# Start with Zaino (experimental)
zecdev up --backend zaino

# Fresh start (remove old data)
zecdev up --fresh
```

### Stop Devnet

```bash
# Stop services (keep data)
zecdev down

# Stop and remove volumes
zecdev down --purge
```

### Check Status

```bash
zecdev status
```

### Run Tests

```bash
zecdev test
```

## Commands

| Command | Description |
|---------|-------------|
| `up` | Start the devnet |
| `down` | Stop the devnet |
| `status` | Show service status |
| `test` | Run smoke tests |

## Options

### `zecdev up`

- `--backend <BACKEND>` - Backend to use: `lwd` (lightwalletd) or `zaino`
- `--fresh` - Remove old data and start fresh

### `zecdev down`

- `--purge` - Remove volumes (clean slate)

## Examples

```bash
# Start everything
zecdev up --backend lwd

# Check if running
zecdev status

# Run tests
zecdev test

# Stop and clean up
zecdev down --purge
```

## Development

### Build

```bash
cargo build
```

### Run

```bash
cargo run -- up
cargo run -- status
cargo run -- test
cargo run -- down
```

### Test

```bash
cargo test
```

## Troubleshooting

### Docker not found

```bash
# Install Docker: https://docs.docker.com/get-docker/
```

### Services not starting

```bash
# Check Docker is running
docker ps

# View logs
docker compose logs zebra
docker compose logs faucet
```

### Port conflicts

```bash
# Stop other services using:
# - 8232 (Zebra RPC)
# - 8080 (Faucet API)
# - 9067 (Backend)
```

## License

MIT OR Apache-2.0