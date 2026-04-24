# 📚 On-Chain Book Club — Soroban Smart Contract

> A decentralized book-sharing club on the Stellar network. Members pool funds, vote with their wallets, and track every book — from purchase to last page — entirely on-chain.

---

## Project Description

**On-Chain Book Club** is a [Soroban](https://soroban.stellar.org/) smart contract deployed on the Stellar blockchain that lets a group of friends, colleagues, or strangers co-own a shared library. No spreadsheets, no trust-me-bro — every contribution, book purchase, checkout, and return is recorded transparently on Stellar's ledger.

The contract replaces the "someone keeps the Google Sheet" model with immutable, auditable on-chain state. Members chip in XLM, the treasury funds new books proposed by any member, and a lightweight check-out/return system tracks who has what — and how many times each book has been read.

---

## What It Does

| Action | Who | Effect |
|--------|-----|--------|
| `initialize` | Admin (once) | Bootstraps the contract, sets the admin address |
| `join` | Any wallet | Registers the caller as a club member |
| `contribute` | Member | Adds XLM (in stroops) to the shared treasury |
| `add_book` | Member | Proposes a book; cost is deducted from the treasury |
| `checkout` | Member | Marks a book as checked out to their address |
| `return_book` | Member | Returns the book, increments the global read counter & personal stat |
| `mark_lost` | Admin | Flags a book as lost |
| `get_book` | Anyone | Reads full book metadata |
| `get_member` | Anyone | Reads a member's contribution & reading stats |
| `get_treasury` | Anyone | Returns current treasury balance |
| `list_available_books` | Anyone | Returns IDs of all books currently available |

---

## Features

### 💰 Shared Treasury
Members contribute any amount of XLM to a collective treasury. Every contribution is recorded against the member's address — no off-chain bookkeeping required.

### 📖 On-Chain Book Catalog
Each book is stored as a struct with title, author, proposing member, purchase cost, current status (`Available` / `CheckedOut` / `Lost`), and a lifetime read counter. The catalog grows as the treasury grows.

### 🔄 Check-Out / Return System
Only one member can hold a book at a time. The contract enforces this: attempting to check out an already-checked-out book reverts the transaction. Returning a book atomically:
- flips the status back to `Available`
- increments `book.times_read`
- increments `member.books_read`

### 📊 Member Stats
Each member has an on-chain profile: total XLM contributed and total books read — useful for governance, rewards, or just bragging rights.

### 🛡️ Auth-Gated Actions
Every state-changing call uses `address.require_auth()`, so only the rightful address can contribute on their behalf, check out, or return a book. Admin-only operations (e.g. marking a book lost) verify the caller matches the stored admin address.

### 🧪 Test Suite Included
Unit tests cover the full happy path: join → contribute → add book → checkout → return, plus edge-case coverage for treasury balance checks and available-book filtering.

---

## Project Structure

```
book-club/
├── Cargo.toml          # Rust / Soroban dependencies
└── src/
    └── lib.rs          # Contract + data types + tests
```

---

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable)
- Soroban CLI: `cargo install --locked soroban-cli`
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`

### Build

```bash
cargo build --target wasm32-unknown-unknown --release
```

The compiled `.wasm` lands in `target/wasm32-unknown-unknown/release/book_club.wasm`.

### Test

```bash
cargo test
```

### Deploy to Testnet

```bash
# Configure testnet identity
soroban keys generate --global alice --network testnet

# Deploy
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/book_club.wasm \
  --source alice \
  --network testnet

# Initialize (replace CONTRACT_ID with output above)
soroban contract invoke \
  --id CONTRACT_ID \
  --source alice \
  --network testnet \
  -- initialize \
  --admin $(soroban keys address alice)
```

### Example Invocations

```bash
# Join the club
soroban contract invoke --id CONTRACT_ID --source alice --network testnet \
  -- join --member $(soroban keys address alice)

# Contribute 1 XLM (10_000_000 stroops)
soroban contract invoke --id CONTRACT_ID --source alice --network testnet \
  -- contribute --member $(soroban keys address alice) --amount 10000000

# Add a book (cost: 0.5 XLM)
soroban contract invoke --id CONTRACT_ID --source alice --network testnet \
  -- add_book \
  --proposer $(soroban keys address alice) \
  --title "Dune" \
  --author "Frank Herbert" \
  --cost 5000000

# Check out book #1
soroban contract invoke --id CONTRACT_ID --source alice --network testnet \
  -- checkout --member $(soroban keys address alice) --book_id 1

# Return book #1
soroban contract invoke --id CONTRACT_ID --source alice --network testnet \
  -- return_book --member $(soroban keys address alice) --book_id 1
```

---

## Roadmap Ideas

- [ ] On-chain voting to approve book proposals before treasury deduction
- [ ] Late-return penalty (contribute extra to re-activate checkout rights)
- [ ] Token-gated membership (hold an NFT to join)
- [ ] Integration with a simple frontend via Freighter wallet

---

## License

MIT — read freely, fork freely.


Wallet Address : GDDPKB4WWCYKYUW73WNLSKZRO6VG4LDFDQ2JISDN7XQ7YYVMJPA6BEMH

Contract Address : CBN6LP7P3XQIWQ52MAZVS5OTBA7VI6D4PPNGOB64ARJDW7W42WBLZGPD

https://stellar.expert/explorer/testnet/contract/CBN6LP7P3XQIWQ52MAZVS5OTBA7VI6D4PPNGOB64ARJDW7W42WBLZGPD

<img width="1456" height="685" alt="image" src="https://github.com/user-attachments/assets/2aa93392-6bbd-4cae-ba9a-565a67bf3483" />

