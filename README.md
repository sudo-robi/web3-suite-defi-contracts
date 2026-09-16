# Web3 Suite — DeFi Smart Contracts

> Production-grade Soroban smart contracts for token swaps, concentrated liquidity, and decentralized lending on the Stellar network.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Issues](https://img.shields.io/github/issues/sudo-robi/web3-suite-defi-contracts)](https://github.com/sudo-robi/web3-suite-defi-contracts/issues)
[![Stars](https://img.shields.io/github/stars/sudo-robi/web3-suite-defi-contracts)](https://github.com/sudo-robi/web3-suite-defi-contracts/stargazers)

---

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Features](#features)
- [Tech Stack](#tech-stack)
- [Project Structure](#project-structure)
- [Smart Contracts](#smart-contracts)
  - [Swap Contract](#swap-contract)
  - [Liquidity Contract](#liquidity-contract)
  - [Lending Contract](#lending-contract)
- [Getting Started](#getting-started)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Configuration](#configuration)
- [Building](#building)
- [Testing](#testing)
- [Deployment](#deployment)
- [Contributing](#contributing)
- [License](#license)

---

## Overview

**Web3 Suite DeFi Contracts** is a Rust workspace containing three Soroban smart contracts that power a complete decentralized finance protocol on Stellar. The contracts implement a constant-product AMM for token swaps, a concentrated liquidity manager with configurable fee tiers, and a lending pool with dynamic interest rates and collateral management.

### Why This Exists

DeFi on Stellar is rapidly maturing with the Soroban smart contract platform. This project provides a production-ready, auditable foundation for core DeFi primitives — swap, liquidity, and lending — built natively for Stellar's Soroban VM with Rust's memory safety guarantees.

### Target Audience

- **DeFi developers** building on Stellar/Soroban
- **Protocol integrators** looking for composable DeFi building blocks
- **Security researchers** auditing Soroban contract patterns
- **Liquidity providers** seeking concentrated liquidity on Stellar

---

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                     Stellar / Soroban Network                    │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────┐   ┌──────────────────┐   ┌──────────────────┐  │
│  │    Swap      │   │    Liquidity      │   │     Lending      │  │
│  │  Contract    │   │    Contract       │   │    Contract      │  │
│  │              │   │                   │   │                  │  │
│  │  x * y = k   │   │  Concentrated     │   │  Utilization-    │  │
│  │  AMM Pool    │   │  Liquidity (CL)   │   │  based Rates     │  │
│  │              │   │  Manager          │   │  + Collateral    │  │
│  └──────┬───────┘   └────────┬──────────┘   └────────┬─────────┘  │
│         │                    │                       │             │
│         └────────────────────┼───────────────────────┘             │
│                              │                                    │
│                    ┌─────────▼─────────┐                          │
│                    │  Soroban Runtime   │                          │
│                    │  (WASM VM)         │                          │
│                    └───────────────────┘                          │
│                                                                  │
├──────────────────────────────────────────────────────────────────┤
│  Storage: Instance Storage (per-contract) + Persistent Storage   │
│  Events:  Contract events for indexing (pool_init, swap, etc.)   │
│  Auth:    require_auth() for state-mutating operations           │
└──────────────────────────────────────────────────────────────────┘
```

### Data Flow

1. **Swap**: User calls `swap(from, amount_in, min_amount_out, a_to_b)` → contract calculates output via constant-product formula → transfers tokens → updates reserves → emits event.
2. **Liquidity**: Provider calls `add_liquidity(owner, pool_id, tick_lower, tick_upper, amount_a, amount_b)` → contract creates position within tick range → updates pool liquidity → emits event.
3. **Lending**: User calls `supply(supplier, amount)` or `borrow(borrower, amount)` → contract checks collateral/health factor → updates pool utilization → recalculates interest rates → emits event.

---

## Features

1. **Constant-Product AMM** — Classic `x * y = k` swap mechanism with configurable fee tiers
2. **Price Impact Calculation** — Real-time price impact estimation before execution
3. **Slippage Protection** — `min_amount_out` parameter prevents unfavorable trades
4. **Concentrated Liquidity** — LPs provide liquidity within specific tick ranges for capital efficiency
5. **Multiple Fee Tiers** — Configurable fee tiers (1, 5, 30, 100 BPS) for different risk profiles
6. **Position Management** — Create, modify, and remove individual liquidity positions
7. **Fee Collection** — Accumulated trading fees tracked per-position and collectible independently
8. **Dynamic Interest Rates** — Utilization-based interest rate model with kink at optimal utilization
9. **Collateral Management** — Separate collateral deposits with 75% collateral factor
10. **Health Factor Monitoring** — Real-time health factor calculation (10000 = 1.0x safe threshold)
11. **Liquidation Protection** — 80% LTV liquidation threshold prevents undercollateralized borrows
12. **Exchange Rate Tracking** — Q64.64 fixed-point exchange rate for lending pool shares
13. **Overflow-Safe Arithmetic** — All calculations use `checked_add/sub/mul/div` to prevent overflow
14. **Event Emission** — Comprehensive event logging for all state-changing operations
15. **No-Std Optimization** — `#![no_std]` with release profile tuned for minimal WASM binary size

---

## Tech Stack

| Layer | Technology | Version | Purpose |
|-------|-----------|---------|---------|
| Language | Rust | 2021 Edition | Smart contract implementation |
| SDK | Soroban SDK | 21.0.0 | Stellar smart contract framework |
| Test Utils | soroban-sdk-testutils | 21.0.0 | Contract testing utilities |
| Stellar SDK | stellar-sdk | 21.0.0 | Stellar network interaction |
| Build | Cargo | Latest | Rust package manager and build tool |
| Target | WASM | `wasm32-unknown-unknown` | Soroban runtime target |
| LTO | LLVM | Built-in | Link-time optimization for binary size |

---

## Project Structure

```
web3-suite-defi-contracts/
├── Cargo.toml                          # Workspace root — defines members and shared deps
├── LICENSE                             # MIT License
├── CONTRIBUTING.md                     # Contribution guidelines
├── README.md                           # This file
│
├── contracts/
│   ├── swap/                           # Constant-product AMM swap contract
│   │   ├── Cargo.toml                  # Package: swap-contract v0.1.0
│   │   └── src/
│   │       └── lib.rs                  # SwapContract implementation (413 lines)
│   │
│   ├── liquidity/                      # Concentrated liquidity pool manager
│   │   ├── Cargo.toml                  # Package: liquidity-contract v0.1.0
│   │   └── src/
│   │       └── lib.rs                  # LiquidityContract implementation (423 lines)
│   │
│   └── lending/                        # Decentralized lending pool
│       ├── Cargo.toml                  # Package: lending-contract v0.1.0
│       └── src/
│           └── lib.rs                  # LendingContract implementation (596 lines)
│
└── .gitignore                          # Rust/Cargo ignores
```

---

## Smart Contracts

### Swap Contract

**Package**: `swap-contract` | **Source**: `contracts/swap/src/lib.rs`

A constant-product automated market maker (AMM) for swapping two tokens. Implements the `x * y = k` invariant with configurable trading fees, slippage protection, and LP share management.

#### Types

```rust
pub enum SwapError {
    InsufficientLiquidity = 1,
    InvalidAmount = 2,
    SlippageExceeded = 3,
    ZeroOutput = 4,
    Unauthorized = 5,
    InvalidToken = 6,
    MathOverflow = 7,
}

pub struct SwapQuote {
    pub amount_in: u128,
    pub amount_out: u128,
    pub fee: u128,
    pub price_impact_pct: u128,
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

#### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `initialize` | `(admin: Address, token_a: Address, token_b: Address, fee_bps: u128) -> Result<(), SwapError>` | Initialize a new swap pool with two tokens and a fee (max 1000 BPS = 10%) |
| `get_pool_info` | `() -> PoolInfo` | Get current pool reserves and metadata (read-only) |
| `get_swap_quote` | `(amount_in: u128, a_to_b: bool) -> Result<SwapQuote, SwapError>` | Calculate a swap quote without executing |
| `swap` | `(from: Address, amount_in: u128, min_amount_out: u128, a_to_b: bool) -> Result<u128, SwapError>` | Execute a token swap with slippage protection |
| `add_liquidity` | `(provider: Address, amount_a: u128, amount_b: u128) -> Result<u128, SwapError>` | Add liquidity to the pool, returns LP shares minted |
| `remove_liquidity` | `(provider: Address, shares: u128) -> Result<(u128, u128), SwapError>` | Remove liquidity by burning LP shares, returns (amount_a, amount_b) |

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `admin` | `Address` | Pool administrator (set during initialization) |
| `token_a` | `Address` | Address of the first token contract |
| `token_b` | `Address` | Address of the second token contract |
| `fee_bps` | `u128` | Trading fee in basis points (max 1000 = 10%) |
| `amount_in` | `u128` | Amount of input tokens to swap |
| `min_amount_out` | `u128` | Minimum acceptable output (slippage protection) |
| `a_to_b` | `bool` | Swap direction: `true` = A→B, `false` = B→A |
| `provider` | `Address` | Liquidity provider address |
| `amount_a` | `u128` | Amount of token A to deposit |
| `amount_b` | `u128` | Amount of token B to deposit |
| `shares` | `u128` | Number of LP shares to burn |

#### Return Values

| Function | Return Type | Description |
|----------|-------------|-------------|
| `get_swap_quote` | `SwapQuote` | `{ amount_in, amount_out, fee, price_impact_pct }` |
| `swap` | `u128` | Amount of output tokens received |
| `add_liquidity` | `u128` | Number of LP shares minted |
| `remove_liquidity` | `(u128, u128)` | Tuple of (token_a_amount, token_b_amount) returned |

#### Error Codes

| Code | Name | Condition |
|------|------|-----------|
| 1 | `InsufficientLiquidity` | Pool reserves too low for the requested operation |
| 2 | `InvalidAmount` | Zero amount or fee exceeds maximum |
| 3 | `SlippageExceeded` | Output below `min_amount_out` |
| 4 | `ZeroOutput` | Calculated output is zero |
| 5 | `Unauthorized` | Caller not authorized for the operation |
| 6 | `InvalidToken` | Token address mismatch |
| 7 | `MathOverflow` | Arithmetic overflow in calculation |

#### Usage Examples

```rust
// Initialize pool
client.initialize(&admin, &token_a, &token_b, &30)?; // 0.30% fee

// Add liquidity
let shares = client.add_liquidity(&provider, &10_000, &20_000)?;

// Get quote
let quote = client.get_swap_quote(&1000, &true)?;
// quote.amount_out = ~997 (after 0.30% fee + price impact)
// quote.fee = 3
// quote.price_impact_pct = 100 (1.00%)

// Execute swap with slippage protection
let received = client.swap(&user, &1000, &990, &true)?;
// received >= 990 or reverts with SlippageExceeded

// Remove liquidity
let (amt_a, amt_b) = client.remove_liquidity(&provider, &500)?;
```

#### Tests

| Test | Description |
|------|-------------|
| `test_initialize` | Verifies pool initialization with correct fee and zero reserves |
| `test_add_liquidity` | Validates LP share minting and reserve updates |
| `test_swap_quote` | Confirms quote calculation with fee and price impact |
| `test_swap_zero_amount` | Ensures zero-amount swaps are rejected |
| `test_fee_cannot_exceed_10_percent` | Enforces maximum fee cap at 1000 BPS |

---

### Liquidity Contract

**Package**: `liquidity-contract` | **Source**: `contracts/liquidity/src/lib.rs`

A concentrated liquidity pool manager supporting multiple fee tiers and tick-range-based positions. Enables LPs to provide liquidity within specific price ranges for improved capital efficiency.

#### Types

```rust
pub enum LiquidityError {
    InsufficientLiquidity = 1,
    InvalidAmount = 2,
    InvalidRange = 3,
    PositionNotFound = 4,
    Unauthorized = 5,
    PoolFull = 6,
    MathOverflow = 7,
}

pub struct PoolConfig {
    pub token_a: Address,
    pub token_b: Address,
    pub fee_tier_bps: u128,
    pub tick_spacing: u32,
    pub total_liquidity: u128,
    pub sqrt_price: u128,           // Q64.64 fixed-point
}

pub struct LiquidityPosition {
    pub id: u64,
    pub owner: Address,
    pub pool_id: u32,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub liquidity: u128,
    pub fee_growth_inside: u128,
    pub tokens_owed_a: u128,
    pub tokens_owed_b: u128,
}

pub struct AddLiquidityResult {
    pub position_id: u64,
    pub amount_a: u128,
    pub amount_b: u128,
    pub liquidity_minted: u128,
}

pub struct RemoveLiquidityResult {
    pub amount_a: u128,
    pub amount_b: u128,
    pub fees_a: u128,
    pub fees_b: u128,
}
```

#### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `initialize` | `(admin: Address) -> Result<(), LiquidityError>` | Initialize the liquidity pool manager with default fee tiers (1, 5, 30, 100 BPS) |
| `create_pool` | `(token_a: Address, token_b: Address, fee_tier_bps: u128, tick_spacing: u32, initial_sqrt_price: u128) -> Result<u32, LiquidityError>` | Create a new liquidity pool (admin only) |
| `get_pool` | `(pool_id: u32) -> Result<PoolConfig, LiquidityError>` | Get pool configuration by ID |
| `get_pool_count` | `() -> u32` | Get total number of pools |
| `add_liquidity` | `(owner: Address, pool_id: u32, tick_lower: i32, tick_upper: i32, amount_a: u128, amount_b: u128) -> Result<AddLiquidityResult, LiquidityError>` | Add concentrated liquidity within a tick range |
| `remove_liquidity` | `(owner: Address, position_id: u64, liquidity_to_remove: u128) -> Result<RemoveLiquidityResult, LiquidityError>` | Remove liquidity from a position |
| `collect_fees` | `(owner: Address, position_id: u64) -> Result<(u128, u128), LiquidityError>` | Collect accumulated fees from a position |
| `get_fee_tiers` | `() -> Vec<u128>` | Get available fee tiers |

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `admin` | `Address` | Pool manager administrator |
| `pool_id` | `u32` | Unique pool identifier |
| `fee_tier_bps` | `u128` | Fee tier in basis points |
| `tick_spacing` | `u32` | Minimum tick spacing for the pool |
| `initial_sqrt_price` | `u128` | Initial sqrt price in Q64.64 fixed-point |
| `tick_lower` | `i32` | Lower bound of the liquidity position's price range |
| `tick_upper` | `i32` | Upper bound of the liquidity position's price range |
| `amount_a` | `u128` | Amount of token A to deposit |
| `amount_b` | `u128` | Amount of token B to deposit |
| `position_id` | `u64` | Unique position identifier |
| `liquidity_to_remove` | `u128` | Amount of liquidity to withdraw |

#### Return Values

| Function | Return Type | Description |
|----------|-------------|-------------|
| `create_pool` | `u32` | New pool ID |
| `get_pool` | `PoolConfig` | Pool configuration struct |
| `add_liquidity` | `AddLiquidityResult` | `{ position_id, amount_a, amount_b, liquidity_minted }` |
| `remove_liquidity` | `RemoveLiquidityResult` | `{ amount_a, amount_b, fees_a, fees_b }` |
| `collect_fees` | `(u128, u128)` | Tuple of (fees_a, fees_b) collected |

#### Error Codes

| Code | Name | Condition |
|------|------|-----------|
| 1 | `InsufficientLiquidity` | Trying to remove more liquidity than available |
| 2 | `InvalidAmount` | Zero amounts or both amounts are zero |
| 3 | `InvalidRange` | `tick_lower >= tick_upper` |
| 4 | `PositionNotFound` | Pool or position ID does not exist |
| 5 | `Unauthorized` | Caller is not the position owner |
| 6 | `PoolFull` | Pool has reached maximum capacity |
| 7 | `MathOverflow` | Arithmetic overflow in calculation |

#### Usage Examples

```rust
// Initialize manager
client.initialize(&admin)?;

// Create pool with 0.30% fee tier
let pool_id = client.create_pool(&token_a, &token_b, &30, &1, &1_000_000)?;

// Add concentrated liquidity (tick range -100 to 100)
let result = client.add_liquidity(&owner, &pool_id, &-100, &100, &5000, &5000)?;
// result.position_id = 1
// result.liquidity_minted = 10000

// Remove half the liquidity
let removed = client.remove_liquidity(&owner, &result.position_id, &5000)?;

// Collect accumulated fees
let (fees_a, fees_b) = client.collect_fees(&owner, &result.position_id)?;
```

#### Tests

| Test | Description |
|------|-------------|
| `test_initialize` | Verifies manager initialization with zero pool count |
| `test_create_pool` | Confirms pool creation with correct fee tier and ID |
| `test_add_liquidity` | Validates position creation and liquidity accounting |
| `test_invalid_tick_range` | Ensures equal tick bounds are rejected |
| `test_remove_liquidity` | Verifies proportional token withdrawal and position update |

---

### Lending Contract

**Package**: `lending-contract` | **Source**: `contracts/lending/src/lib.rs`

A decentralized lending pool with dynamic interest rates based on utilization, collateral management, and health factor monitoring. Implements a two-slope interest rate model with configurable parameters.

#### Types

```rust
pub enum LendingError {
    InsufficientCollateral = 1,
    InsufficientLiquidity = 2,
    InvalidAmount = 3,
    Undercollateralized = 4,
    PositionNotFound = 5,
    Unauthorized = 6,
    MathOverflow = 7,
    HealthFactorTooLow = 8,
}

pub struct LendingPool {
    pub asset: Address,
    pub total_deposits: u128,
    pub total_borrowed: u128,
    pub reserve_factor_bps: u128,
    pub interest_rate_model: InterestRateModel,
    pub last_update_timestamp: u64,
    pub exchange_rate: u128,           // Q64.64 fixed-point
}

pub struct InterestRateModel {
    pub base_rate_bps: u128,
    pub slope_bps: u128,
    pub optimal_utilization: u128,
}

pub struct UserPosition {
    pub deposited: u128,
    pub borrowed: u128,
    pub collateral_value: u128,
    pub last_accrued: u64,
    pub interest_earned: u128,
    pub interest_owed: u128,
}

pub struct BorrowResult {
    pub amount: u128,
    pub health_factor: u128,
    pub new_utilization: u128,
}

pub struct SupplyResult {
    pub shares: u128,
    pub exchange_rate: u128,
}
```

#### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `initialize` | `(admin: Address, asset: Address, reserve_factor_bps: u128, base_rate_bps: u128, slope_bps: u128, optimal_utilization: u128) -> Result<(), LendingError>` | Initialize the lending pool with interest rate parameters |
| `get_pool_info` | `() -> LendingPool` | Get current lending pool state (read-only) |
| `get_utilization_rate` | `() -> u128` | Calculate current utilization rate (read-only) |
| `get_supply_apy` | `() -> u128` | Calculate current supply APY (read-only) |
| `get_borrow_apy` | `() -> u128` | Calculate current borrow APY (read-only) |
| `supply` | `(supplier: Address, amount: u128) -> Result<SupplyResult, LendingError>` | Supply assets to earn interest |
| `borrow` | `(borrower: Address, amount: u128) -> Result<BorrowResult, LendingError>` | Borrow against deposited collateral |
| `deposit_collateral` | `(depositor: Address, amount: u128) -> Result<u128, LendingError>` | Deposit collateral to enable borrowing |
| `get_user_position` | `(user: Address) -> UserPosition` | Get user position details (read-only) |
| `get_health_factor` | `(user: Address) -> u128` | Get health factor for a user (read-only) |

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `admin` | `Address` | Pool administrator |
| `asset` | `Address` | Address of the lendable asset contract |
| `reserve_factor_bps` | `u128` | Protocol reserve factor (max 5000 = 50%) |
| `base_rate_bps` | `u128` | Base annual interest rate in BPS |
| `slope_bps` | `u128` | Interest rate slope above optimal utilization |
| `optimal_utilization` | `u128` | Optimal utilization target in BPS |
| `supplier` | `Address` | Address supplying assets |
| `borrower` | `Address` | Address borrowing assets |
| `amount` | `u128` | Amount to supply, borrow, or deposit as collateral |
| `user` | `Address` | User address to query |

#### Return Values

| Function | Return Type | Description |
|----------|-------------|-------------|
| `get_pool_info` | `LendingPool` | Full pool state struct |
| `get_utilization_rate` | `u128` | Utilization in BPS (e.g., 5000 = 50%) |
| `get_supply_apy` | `u128` | Supply APY in BPS |
| `get_borrow_apy` | `u128` | Borrow APY in BPS |
| `supply` | `SupplyResult` | `{ shares, exchange_rate }` |
| `borrow` | `BorrowResult` | `{ amount, health_factor, new_utilization }` |
| `deposit_collateral` | `u128` | Total collateral value after deposit |
| `get_user_position` | `UserPosition` | Full user position struct |
| `get_health_factor` | `u128` | Health factor (10000 = 1.0x safe, MAX = no borrows) |

#### Error Codes

| Code | Name | Condition |
|------|------|-----------|
| 1 | `InsufficientCollateral` | Not enough collateral for the borrow |
| 2 | `InsufficientLiquidity` | Borrow exceeds available deposits |
| 3 | `InvalidAmount` | Zero amount or reserve factor > 50% |
| 4 | `Undercollateralized` | Borrow exceeds collateral * factor |
| 5 | `PositionNotFound` | User position does not exist |
| 6 | `Unauthorized` | Caller not authorized |
| 7 | `MathOverflow` | Arithmetic overflow |
| 8 | `HealthFactorTooLow` | Health factor below 1.0x (10000) |

#### Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `BPS_DENOMINATOR` | `10,000` | Basis point denominator (100%) |
| `SECONDS_PER_YEAR` | `31,536,000` | Seconds in a year for APY calculation |
| `LIQUIDATION_THRESHOLD` | `8,000` | 80% LTV liquidation trigger |
| `COLLATERAL_FACTOR` | `7,500` | 75% maximum borrow against collateral |

#### Usage Examples

```rust
// Initialize with 2% base rate, 4% slope, 80% optimal utilization
client.initialize(&admin, &asset, &500, &200, &4000, &8000)?;

// Supply assets
let result = client.supply(&supplier, &10_000)?;
// result.shares = 10_000
// result.exchange_rate = 1_000_000_000

// Deposit collateral
client.deposit_collateral(&borrower, &10_000)?;

// Borrow (requires sufficient collateral)
let borrow = client.borrow(&borrower, &5_000)?;
// borrow.health_factor = 16000 (1.6x safe)

// Check health factor
let hf = client.get_health_factor(&user);
// hf = 16000 (safe) or hf < 10000 (liquidation risk)

// Query rates
let supply_apy = client.get_supply_apy();  // e.g., 150 (1.50%)
let borrow_apy = client.get_borrow_apy();  // e.g., 200 (2.00%)
let utilization = client.get_utilization_rate(); // e.g., 5000 (50%)
```

#### Interest Rate Model

The interest rate follows a two-slope model:

```
If utilization <= optimal_utilization:
    borrow_rate = base_rate + (utilization * slope / optimal_utilization)

If utilization > optimal_utilization:
    borrow_rate = base_rate + slope + ((utilization - optimal) * slope / (100% - optimal))

supply_rate = borrow_rate * utilization * (1 - reserve_factor) / 100%
```

#### Tests

| Test | Description |
|------|-------------|
| `test_initialize` | Verifies pool initialization with zero deposits and borrows |
| `test_supply` | Confirms share minting and deposit accounting |
| `test_borrow_requires_collateral` | Ensures borrowing without collateral fails |
| `test_borrow_with_collateral` | Validates borrowing with sufficient collateral and health factor |
| `test_health_factor` | Confirms infinite health factor with no borrows |
| `test_interest_rate_calculation` | Verifies borrow APY > supply APY due to reserve factor |

---

## Getting Started

### Prerequisites

- **Rust** >= 1.75.0 (`rustc --version`)
- **Cargo** (included with Rust)
- **soroban-cli** (`cargo install --locked soroban-cli`)
- **Stellar CLI** (optional, for deployment)

### Installation

```bash
# Clone the repository
git clone https://github.com/sudo-robi/web3-suite-defi-contracts.git
cd web3-suite-defi-contracts

# Verify toolchain
rustc --version
cargo --version
soroban --version
```

### Building

```bash
# Build all contracts
cargo build --all --target wasm32-unknown-unknown --release

# Build individual contracts
cargo build -p swap-contract --target wasm32-unknown-unknown --release
cargo build -p liquidity-contract --target wasm32-unknown-unknown --release
cargo build -p lending-contract --target wasm32-unknown-unknown --release

# Optimize WASM binary size (already configured in workspace Cargo.toml)
# Release profile: opt-level="z", LTO enabled, strip symbols
```

### Testing

```bash
# Run all tests
cargo test --all

# Run tests for a specific contract
cargo test -p swap-contract
cargo test -p liquidity-contract
cargo test -p lending-contract

# Run tests with output
cargo test --all -- --nocapture

# Run a specific test
cargo test test_initialize -- --nocapture
```

### Deployment

```bash
# Install soroban-cli if not already installed
cargo install --locked soroban-cli

# Deploy to testnet
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/swap_contract.wasm \
  --source admin_secret \
  --network testnet

# Initialize the contract
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source admin_secret \
  --network testnet \
  -- initialize \
  --admin <ADMIN_ADDRESS> \
  --token_a <TOKEN_A_ADDRESS> \
  --token_b <TOKEN_B_ADDRESS> \
  --fee_bps 30
```

---

## Configuration

### Release Profile (Workspace Cargo.toml)

```toml
[profile.release]
opt-level = "z"          # Optimize for binary size
overflow-checks = true   # Safety: revert on overflow
debug = 0                # No debug info in release
strip = "symbols"        # Strip debug symbols
debug-assertions = false # No debug assertions
panic = "abort"          # Abort on panic (saves binary size)
codegen-units = 1        # Single codegen unit (better optimization)
lto = true               # Link-time optimization
```

### Soroban SDK Version

All contracts target `soroban-sdk` version 21.0.0. Ensure your `soroban-cli` version is compatible.

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

### Branch Naming

| Type | Pattern | Example |
|------|---------|---------|
| Feature | `feat/<description>` | `feat/add-flash-loan` |
| Bug Fix | `fix/<description>` | `fix/overflow-in-swap` |
| Refactor | `refactor/<description>` | `refactor/extract-math-utils` |
| Docs | `docs/<description>` | `docs/update-api-reference` |

### Commit Conventions

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add flash loan contract
fix: prevent overflow in reserve calculation
docs: update swap contract API reference
test: add edge case tests for lending collateral
refactor: extract common math utilities to shared crate
```

### Code Style

- `#![no_std]` for all contracts (no standard library)
- Use `checked_add/sub/mul/div` for all arithmetic (never use `+`, `-`, `*`, `/`)
- Use `symbol_short!` for storage keys (max 9 chars)
- Emit events for all state-changing operations
- Use `require_auth()` for user-initiated state changes
- Document all public functions with `///` doc comments
- Keep functions focused and under 50 lines where possible

### Pull Request Process

1. Fork the repository
2. Create a feature branch from `main`
3. Write tests for new functionality
4. Ensure all tests pass: `cargo test --all`
5. Run clippy: `cargo clippy --all --target wasm32-unknown-unknown -- -D warnings`
6. Submit PR with clear description and test evidence

---

## License

MIT License — see [LICENSE](LICENSE) for details.

---

<p align="center">
  <sub>Built with Rust & Soroban on the Stellar network</sub>
</p>
