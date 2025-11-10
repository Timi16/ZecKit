# Faucet Service - Placeholder for M2

## Status: Not Implemented (M1)

This directory is a placeholder for the faucet service that will be implemented in **Milestone 2**.

---

## Planned Implementation

### Technology Stack
- **Language:** Python 3.10+
- **Framework:** Flask
- **RPC Client:** python-zcashd or custom JSON-RPC wrapper

### Functionality

The faucet will:
1. Accept funding requests via HTTP API
2. Validate addresses and amounts
3. Send test ZEC using Zebra RPC
4. Track dispensed funds
5. Implement rate limiting

### API Endpoints (Planned)

```
GET  /health
  → Service health status

POST /fund
  Body: { "address": "ztestsapling...", "amount": 10.0 }
  → Request test funds
  Response: { "txid": "...", "amount": 10.0 }

GET  /status
  → Faucet balance and statistics

GET  /fixtures
  → Pre-funded addresses for testing
```

---

## M1 Placeholder Structure

```
faucet/
├── README.md              (this file)
├── requirements.txt       (to be added in M2)
├── app.py                 (to be added in M2)
├── config.py              (to be added in M2)
└── tests/                 (to be added in M2)
```

---

## Docker Integration (M2)

The faucet will be added to the Docker Compose stack:

```yaml
# docker/compose/faucet.yml (M2)
services:
  faucet:
    build: ./faucet
    container_name: zecdev-faucet
    networks:
      - zecdev
    ports:
      - "127.0.0.1:8080:8080"
    environment:
      - ZEBRA_RPC_URL=http://zebra:8232
    depends_on:
      zebra:
        condition: service_healthy
```

---

## Contributing

If you'd like to contribute to the faucet implementation in M2:

1. Review the [Technical Spec](../specs/technical-spec.md)
2. Check [Acceptance Tests](../specs/acceptance-tests.md) for M2 criteria
3. Open a discussion or issue to coordinate

---

## Timeline

- **M1:** Placeholder structure (current)
- **M2:** Full implementation
  - Python Flask app
  - Zebra RPC integration
  - Rate limiting
  - Pre-funded fixture generation

---

**Questions?** Open a [GitHub Discussion](https://github.com/Supercoolkayy/ZecKit/discussions)