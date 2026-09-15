# Web3 Suite — Soroban Smart Contracts

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Issues](https://img.shields.io/github/issues/web3-suite/defi-contracts)](https://github.com/web3-suite/defi-contracts/issues)
[![Stars](https://img.shields.io/github/stars/web3-suite/defi-contracts)](https://github.com/web3-suite/defi-contracts/stargazers)
[![Soroban](https://img.shields.io/badge/Soroban-21.0-purple)](https://soroban.stellar.org)
[![Rust](https://img.shields.io/badge/Rust-stable-orange)](https://www.rust-lang.org)

> Production-grade DeFi smart contracts for the Stellar/Soroban ecosystem — AMM swap, concentrated liquidity, and over-collateralized lending.

---

## Overview

This workspace contains three Soroban smart contracts that form the core DeFi primitive layer:

| Contract | Purpose | Key Features |
|----------|---------|--------------|
| **swap** | Constant-product AMM | Configurable fees (1-10%), slippage protection, LP share minting |
| **liquidity** | Concentrated liquidity pools | 4 fee tiers, tick-range positions, fee collection |
| **lending** | Over-collateralized lending | Interest rate model, health factors, liquidation thresholds |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Soroban Runtime                          │
│                                                             │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │  Swap        │  │  Liquidity   │  │  Lending         │   │
│  │  Contract    │  │  Contract    │  │  Contract        │   │
│  │              │  │              │  │                   │   │
│  │  ┌────────┐  │  │  ┌────────┐  │  │  ┌────────────┐  │   │
│  │  │ AMM    │  │  │  │ Pool   │  │  │  │ Interest   │  │   │
│  │  │ Engine │  │  │  │ Mgmt   │  │  │  │ Rate Model │  │   │
│  │  └────────┘  │  │  └────────┘  │  │  └────────────┘  │   │
│  │  ┌────────┐  │  │  ┌────────┐  │  │  ┌────────────┐  │   │
│  │  │ Fee    │  │  │  │ Tick   │  │  │  │ Collateral │  │   │
│  │  │ Calc   │  │  │  │ Range  │  │  │  │ Manager    │  │   │
│  │  └────────┘  │  │  └────────┘  │  │  └────────────┘  │   │
│  └─────────────┘  └──────────────┘  └──────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Soroban Token Interface                 │   │
│  │         (transfer, transfer_from, balance)           │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
         │                    │                    │
         ▼                    ▼                    ▼
    ┌─────────┐         ┌─────────┐         ┌─────────┐
    │ Token A │         │ Token B │         │ LP Token│
    └─────────┘         └─────────┘         └─────────┘
```

---

## Workspace Structure

```
contracts/
├── Cargo.toml                  # Workspace root
├── contracts/
│   ├── swap/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs          # AMM swap contract
│   ├── liquidity/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs          # Concentrated liquidity contract
│   └── lending/
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs          # Over-collateralized lending contract
├── README.md
├── CONTRIBUTING.md
├── LICENSE
└── .gitignore
```

---

## Contract API Reference

### Swap Contract (`contracts/swap/src/lib.rs`)

Constant-product AMM with configurable fees and slippage protection.

#### Functions

| Function | Visibility | Parameters | Returns | Description |
|----------|-----------|------------|---------|-------------|
| `initialize` | Public | `admin, token_a, token_b, fee_bps` | `Result<(), SwapError>` | Create a new swap pool |
| `get_pool_info` | Public | — | `PoolInfo` | Get reserves, fees, and shares |
| `get_swap_quote` | Public | `amount_in, a_to_b` | `Result<SwapQuote, SwapError>` | Get quote without executing |
| `swap` | Public | `from, amount_in, min_amount_out, a_to_b` | `Result<u128, SwapError>` | Execute a token swap |
| `add_liquidity` | Public | `provider, amount_a, amount_b` | `Result<u128, SwapError>` | Deposit tokens, receive LP shares |
| `remove_liquidity` | Public | `provider, shares` | `Result<(u128, u128), SwapError>` | Burn shares, receive tokens |

#### Types

```rust
pub struct SwapQuote {
    pub amount_in: u128,
    pub amount_out: u128,
    pub fee: u128,
    pub price_impact_pct: u128,  // Basis points
}

pub struct PoolInfo {
    pub token_a: Address,
    pub token_b: Address,
    pub reserve_a: u128,
    pub reserve_b: u128,
    pub fee_bps: u128,
    pub total_shares: u128,
}
```

#### Error Codes

| Code | Name | Description |
|------|------|-------------|
| 1 | `InsufficientLiquidity` | Pool has no reserves |
| 2 | `InvalidAmount` | Zero amount or fee > 10% |
| 3 | `SlippageExceeded` | Output < minimum requested |
| 4 | `ZeroOutput` | Calculated output is zero |
| 5 | `Unauthorized` | Missing auth |
| 6 | `InvalidToken` | Wrong token address |
| 7 | `MathOverflow` | Arithmetic overflow |

---

### Liquidity Contract (`contracts/liquidity/src/lib.rs`)

Concentrated liquidity with tick-range positions and 4 fee tiers.

#### Functions

| Function | Visibility | Parameters | Returns | Description |
|----------|-----------|------------|---------|-------------|
| `initialize` | Public | `admin` | `Result<(), LiquidityError>` | Initialize pool manager |
| `create_pool` | Public | `token_a, token_b, fee_tier_bps, tick_spacing, initial_sqrt_price` | `Result<u32, LiquidityError>` | Create new pool |
| `get_pool` | Public | `pool_id` | `Result<PoolConfig, LiquidityError>` | Get pool config |
| `get_pool_count` | Public | — | `u32` | Total pool count |
| `add_liquidity` | Public | `owner, pool_id, tick_lower, tick_upper, amount_a, amount_b` | `Result<AddLiquidityResult, LiquidityError>` | Add concentrated liquidity |
| `remove_liquidity` | Public | `owner, position_id, liquidity_to_remove` | `Result<RemoveLiquidityResult, LiquidityError>` | Remove liquidity |
| `collect_fees` | Public | `owner, position_id` | `Result<(u128, u128), LiquidityError>` | Collect accumulated fees |
| `get_fee_tiers` | Public | — | `Vec<u128>` | Get available fee tiers |

#### Fee Tiers

| Tier | Fee (BPS) | Tick Spacing | Use Case |
|------|-----------|-------------|----------|
| 1 | 0.01% | 1 | Stable pairs (USDC/USDT) |
| 5 | 0.05% | 10 | Low-volatility pairs |
| 30 | 0.30% | 60 | Standard pairs (XLM/USDC) |
| 100 | 1.00% | 200 | Exotic/high-volatility pairs |

---

### Lending Contract (`contracts/lending/src/lib.rs`)

Over-collateralized lending with dynamic interest rates.

#### Functions

| Function | Visibility | Parameters | Returns | Description |
|----------|-----------|------------|---------|-------------|
| `initialize` | Public | `admin, asset, reserve_factor_bps, base_rate_bps, slope_bps, optimal_utilization` | `Result<(), LendingError>` | Initialize lending pool |
| `get_pool_info` | Public | — | `LendingPool` | Get pool state |
| `get_utilization_rate` | Public | — | `u128` | Current utilization (BPS) |
| `get_supply_apy` | Public | — | `u128` | Current supply APY (BPS) |
| `get_borrow_apy` | Public | — | `u128` | Current borrow APY (BPS) |
| `supply` | Public | `supplier, amount` | `Result<SupplyResult, LendingError>` | Deposit assets |
| `borrow` | Public | `borrower, amount` | `Result<BorrowResult, LendingError>` | Borrow against collateral |
| `deposit_collateral` | Public | `depositor, amount` | `Result<u128, LendingError>` | Deposit collateral |
| `get_user_position` | Public | `user` | `UserPosition` | Get user's position |
| `get_health_factor` | Public | `user` | `u128` | Health factor (10000 = 1.0x) |

#### Interest Rate Model

```
Borrow Rate = Base Rate + Slope × (Utilization / Optimal Utilization)
            (below optimal)

Borrow Rate = Base Rate + Slope + Slope × (Excess / Max Excess)
            (above optimal)

Supply APY = Borrow APY × Utilization × (1 - Reserve Factor)
```

#### Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `reserve_factor_bps` | 500 (5%) | Protocol fee on interest |
| `base_rate_bps` | 200 (2%) | Minimum borrow rate |
| `slope_bps` | 400 (4%) | Rate increase per utilization unit |
| `optimal_utilization` | 8000 (80%) | Target utilization rate |
| `LIQUIDATION_THRESHOLD` | 8000 (80%) | LTV for liquidation |
| `COLLATERAL_FACTOR` | 7500 (75%) | Max borrow against collateral |

---

## Build Instructions

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Soroban CLI
cargo install --locked --git https://github.com/stellar/soroban-tools soroban-cli

# Verify installation
soroban --version
```

### Build All Contracts

```bash
cd contracts/

# Build all (release mode for deployment)
cargo build --release

# Build specific contract
cargo build -p swap-contract --release
cargo build -p liquidity-contract --release
cargo build -p lending-contract --release
```

### Run Tests

```bash
# Run all tests
cargo test

# Run tests for specific contract
cargo test -p swap-contract
cargo test -p liquidity-contract
cargo test -p lending-contract

# Run with output
cargo test -- --nocapture
```

---

## Deploy Guide

### 1. Deploy to Testnet

```bash
# Configure network
export SOROBAN_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
export SOROBAN_RPC_URL="https://soroban-testnet.stellar.org"

# Generate an admin keypair
soroban keys generate --network testnet admin

# Fund the admin account
curl "https://friendbot.stellar.org?addr=$(soroban keys address admin)"

# Deploy swap contract
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/swap_contract.wasm \
  --source admin \
  --network testnet

# Initialize swap contract
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source admin \
  --network testnet \
  -- initialize \
  --admin $(soroban keys address admin) \
  --token_a <TOKEN_A_ADDRESS> \
  --token_b <TOKEN_B_ADDRESS> \
  --fee_bps 30
```

### 2. Deploy to Mainnet

```bash
# Switch to mainnet
export SOROBAN_NETWORK_PASSPHRASE="Public Global Stellar Network ; September 2015"
export SOROBAN_RPC_URL="https://soroban-mainnet.stellar.org"

# Use the same deploy commands with mainnet credentials
```

### 3. Verify Deployment

```bash
# Read contract state
soroban contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  -- get_pool_info
```

---

## Configuration

### Fee Constraints

| Contract | Min Fee | Max Fee | Denominator |
|----------|---------|---------|-------------|
| Swap | 0 BPS | 1000 BPS (10%) | 10,000 |
| Liquidity | 1 BPS | 100 BPS | 10,000 |
| Lending (reserve) | 0 BPS | 5000 BPS (50%) | 10,000 |

### Storage Layout

All contracts use Soroban instance storage with `symbol_short!` keys:

```
Swap:     TOKEN_A, TOKEN_B, RESRV_A, RESRV_B, FEE_BPS, ADMIN, SHARES
Liquidity: ADMIN, PL_COUNT, FEE_TIRS, (POOL_COUNT, pool_id), (FEE_TIERS, position_id)
Lending:  ADMIN, RESERVES, BORROWS, COLLTRL, INT_MODEL, TOT_BRRW, TOT_DEP, PROT_FEE
```

---

## Testing Strategy

Each contract includes comprehensive unit tests:

- **Swap**: Initialization, liquidity provision, quote calculation, swap execution, slippage protection, fee validation
- **Liquidity**: Pool creation, position management, tick range validation, fee collection
- **Lending**: Supply/borrow flows, collateral requirements, health factor calculation, interest accrual

```bash
# Run all tests with coverage
cargo test -- --show-output

# Run specific test
cargo test test_initialize -- --nocapture
cargo test test_swap_quote -- --nocapture
cargo test test_borrow_with_collateral -- --nocapture
```

---

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Write tests for new functionality
4. Ensure all tests pass: `cargo test`
5. Format code: `cargo fmt`
6. Lint: `cargo clippy -- -D warnings`
7. Submit a pull request

### Code Standards

- All public functions must have doc comments
- Use checked arithmetic (`.checked_add()`, `.checked_mul()`) — never raw `+`/`*`
- Error types must use `#[derive(Clone)]` and implement `Into<StorageError>`
- Tests must cover happy path AND error paths
- No `unwrap()` in production code — use `.ok_or(Error)?`

---

## License

MIT License — see [LICENSE](LICENSE) for details.
