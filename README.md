# Rust Blockchain Implementation

## Overview

This repository contains a functional blockchain and command-line interface (CLI) implemented from scratch in Rust. It serves as an educational project to demonstrate core blockchain concepts and advanced Rust programming patterns, including state management, cryptographic operations, and modular application architecture.

The result is a feature-rich CLI tool that allows users to create wallets, mine blocks, and transfer currency in a simulated, persistent blockchain environment.

## Features

* **Cryptographically Secure Transactions:** All transactions are signed using the Elliptic Curve Digital Signature Algorithm (ECDSA) over the NIST P-256 curve, ensuring the authenticity and integrity of every transfer.
* **Named Wallet Management:** Users can create, manage, and switch between multiple named wallets, each containing a unique cryptographic keypair.
* **Address Book with Aliases:** A persistent contact book allows users to save long, complex public key addresses under easy-to-remember names, greatly improving usability.
* **Proof-of-Work (PoW) Consensus:** New blocks are appended to the chain via a PoW algorithm, requiring computational effort ("mining") to secure the network and validate transactions.
* **Mining Rewards:** A coinbase transaction is included in every new block, rewarding the miner with newly created currency for their work in securing the chain.
* **Dynamic Difficulty Adjustment:** The PoW difficulty is automatically recalibrated every 10 blocks to maintain a consistent average block time, mimicking the behavior of production blockchains.
* **Persistent State Management:** The entire application state—including the blockchain, wallets, contacts, and configuration—is saved to a dedicated directory within the user's standard configuration folder, ensuring data persists between sessions.
* **Professional CLI:** The user interface is a well-structured command-line application featuring subcommands, colorized output, and formatted tables for clear data presentation.

## Getting Started

### Prerequisites

* The [Rust programming language toolchain](https://rustup.rs/) must be installed.

### Installation

To compile and install the application, making it available as a system-wide command:

1.  Clone the repository:
    ```bash
    git clone https://github.com/Rainc4ndyy/mini-blockchain.git
    cd mini-blockchain
    ```

2.  Use Cargo to build and install the binary:
    ```bash
    cargo install --path .
    ```

This will make the `mini-blockchain` command available in your terminal.

### Usage Workflow

The following steps outline a typical user session.

**1. Wallet Generation:**
Create two wallets, named `alice` and `bob`. The first wallet created is automatically set as the active one.
```bash
mini-blockchain wallet new alice
mini-blockchain wallet new bob
```

**2. List Wallets and Create a Contact:**
List the wallets to view their public key addresses. Copy Bob's address to create a contact.
```bash
mini-blockchain wallet list
mini-blockchain contact add bob <BOB'S_PUBLIC_ADDRESS>
```

**3. Acquire Currency via Mining:**
The active wallet (`alice`) must mine a block to receive the initial mining reward.
```bash
mini-blockchain mine
```

**4. Check Balances:**
Verify the balance of the active wallet and the contact.
```bash
mini-blockchain balance
mini-blockchain balance -a bob
```

**5. Create and Confirm a Transaction:**
Create a transaction from the active wallet (`alice`) to `bob`. This adds the transaction to the mempool.
```bash
mini-blockchain add-tx -r bob -a 25
```
To confirm the transaction, a new block must be mined.
```bash
mini-blockchain mine
```
The transaction is now permanently recorded on the blockchain. Final balances can be verified again.

## Command Reference

| Command | Subcommand | Arguments | Description |
|---|---|---|---|
| `wallet` | `new` | `<name>` | Creates a new wallet. |
| | `list` | | Lists all saved wallets. |
| | `use` | `<name>` | Sets the active wallet. |
| `contact`| `add` | `<name> <address>` | Saves a new contact. |
| | `list` | | Lists all saved contacts. |
| `add-tx` | | `-r <dest> -a <amount>` | Adds a transaction to the mempool. |
| `mine` | | | Mines a new block with pending transactions. |
| `balance`| | `[-a <dest>]` | Displays the balance of the active or specified wallet. |
| `pending`| | | Shows pending transactions in the mempool. |
| `list` | | | Displays all blocks in the blockchain history. |
| `validate`| | | Verifies the cryptographic integrity of the blockchain. |
| `clear` | | | Deletes all application data. |

## Project Architecture

This project is structured as a Rust workspace with a library and a binary crate:
* **`src/lib.rs`**: The library crate root, which defines the public API for the core blockchain logic.
* **`src/main.rs`**: The binary crate root, which consumes the library and is solely responsible for the command-line interface logic.
* **Modules (`src/config.rs`, `src/wallet.rs`, etc.)**: Each module has a single, well-defined responsibility (e.g., state management, wallet logic), promoting clean, maintainable code.
