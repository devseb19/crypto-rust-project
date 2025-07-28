
# Crypto-Rust-Project

This repository contains a **Rust-based blockchain simulation**. It implements core cryptocurrency concepts:

- Wallet generation
- Balance tracking
- Transaction processing
- Block mining
- Persistent blockchain storage

---

## Project Snapshot

![Crypto Engine Gear](photo.png)

---

## CLI Usage Guide

### Generate a Wallet

```bash
cargo run -- wallet-gen --name Alexa --fund 100000
```

Creates a new wallet named `Alexa`, stores it, and optionally funds it.

---

### Set or Get Balance

```bash
cargo run -- balance --set <name or address> --amount <u64>
cargo run -- balance --get <name or address>
```

Manually assign or retrieve balance for a given address.

---

### Add a Block (Transaction)

```bash
cargo run -- block-add --from <name or address> --to <name or address> --amount <u64>
```

Transfers value and mines a new block. Fails if sender has insufficient funds.

---

### Show the Blockchain

```bash
cargo run -- blockchain-show
```

Prints a pretty view of all current blocks.

---

### Find a Wallet Address

```bash
cat wallets/<name>.json
```
  