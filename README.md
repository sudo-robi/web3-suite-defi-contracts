<div align="center">

# web3-suite-defi-contracts

**Production-grade Soroban smart contracts for DeFi primitives on Stellar**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![Stellar](https://img.shields.io/badge/Stellar-Soroban-black.svg)](https://soroban.stellar.org)
[![Tests](https://img.shields.io/badge/Tests-passing-brightgreen.svg)](#testing)

</div>

---

## Overview

A collection of auditable, gas-efficient Soroban smart contracts implementing core DeFi primitives on the Stellar network. Built with safety-first design, overflow-checked arithmetic, and comprehensive test coverage.

### What's Included

| Contract | Description | Use Case |
|----------|-------------|----------|
| **Swap** | Constant-product AMM (x·y=k) | Token swaps with configurable fees |
| **Liquidity** | Concentrated liquidity pools | LP positions with tick ranges |
| **Lending** | Over-collateralized lending | Supply, borrow, earn interest |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                     web3-suite-defi-contracts                       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐  │
│  │   swap-contract   │  │liquidity-contract │  │ lending-contract  │  │
│  │                   │  │                   │  │                   │  │
│  │  ┌─────────────┐  │  │  ┌─────────────┐  │  │  ┌─────────────┐  │  │
│  │  │  Pool State  │  │  │  │ Multi-Pool  │  │  │  │  Lending    │  │  │
│  │  │  ─────────── │  │  │  │  Manager    │  │  │  │    Pool     │  │  │
│  │  │  reserve_a   │  │  │  │  ─────────── │  │  │  │  ─────────── │  │  │
│  │  │  reserve_b   │  │  │  │  pool_count  │  │  │  │  deposits   │  │  │
│  │  │  fee_bps     │  │  │  │  fee_tiers   │  │  │  │  borrowed   │  │  │
│  │  │  total_shares│  │  │  │  positions[]  │  │  │  │  collateral │  │  │
│  │  └─────────────┘  │  │  └─────────────┘  │  │  │  ─────────── │  │  │
│  │                   │  │                   │  │  │  int_model    │  │  │
│  │  ┌─────────────┐  │  │  ┌─────────────┐  │  │  │  health_hf   │  │  │
│  │  │   Swap      │  │  │  │  Concentrated│  │  │  └─────────────┘  │  │
│  │  │   Engine    │  │  │  │  Liquidity   │  │  │                   │  │
│  │  │  ─────────── │  │  │  │  ─────────── │  │  │  ┌─────────────┐  │  │
│  │  │  quote()    │  │  │  │  add_liq()   │  │  │  │  Interest   │  │  │
│  │  │  swap()     │  │  │  │  remove_liq()│  │  │  │  Rate Model │  │  │
│  │  │  add_liq()  │  │  │  │  collect()   │  │  │  │  ─────────── │  │  │
│  │  │  remove_liq()│  │  │  └─────────────┘  │  │  │  util_rate()│  │  │
│  │  └─────────────┘  │  │                   │  │  │  supply_apy()│  │  │
│  │                   │  │  ┌─────────────┐  │  │  │  borrow_apy()│  │  │
│  │  ┌─────────────┐  │  │  │   Tick      │  │  │  └─────────────┘  │  │
│  │  │   LP Token  │  │  │  │   Math      │  │  │                   │  │
│  │  │   Shares    │  │  │  │  ─────────── │  │  │  ┌─────────────┐  │  │
│  │  └─────────────┘  │  │  │  ticks[]     │  │  │  │  Liquidation│  │  │
│  │                   │  │  │  positions[]  │  │  │  │  Engine     │  │  │
│  │                   │  │  └─────────────┘  │  │  │  ─────────── │  │  │
│  │                   │  │                   │  │  │  health()    │  │  │
│  │                   │  │                   │  │  │  liquidate() │  │  │
│  │                   │  │                   │  │  └─────────────┘  │  │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘  │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│  Stellar Network  │  soroban-sdk 21.x  │  WASM Compilation Target  │
└─────────────────────────────────────────────────────────────────────┘
```

### Data Flow

```
User → Frontend → Backend API → Stellar CLI → Soroban Contract
                                              ↓
                                    Contract State (Soroban Storage)
                                              ↓
                                    Events → Backend → WebSocket → Frontend
```

---

## Contract Details

### Swap Contract

Constant-product AMM implementing the `x * y = k` invariant.

**Key Features:**
- Configurable fee tiers (0–10% in basis points)
- Price impact calculation
- Slippage protection via `min_amount_out`
- LP share minting/burning on liquidity add/remove
- Overflow-checked arithmetic throughout

**Fee Model:**
- Fees are deducted from `amount_in` before the swap
- Fee revenue accrues to LP providers
- Maximum fee: 1000 BPS (10%)

### Liquidity Contract

Multi-pool concentrated liquidity manager supporting tick-range positions.

**Key Features:**
- Multiple pools with different fee tiers (1, 5, 30, 100 BPS)
- Concentrated liquidity via tick ranges
- Position tracking per user
- Fee accumulation and collection
- Configurable tick spacing

**Fee Tiers:**
| Tier | BPS | Use Case |
|------|-----|----------|
| Stable | 1 | Stablecoin pairs |
| Low | 5 | Correlated assets |
| Medium | 30 | Standard pairs |
| High | 100 | Exotic/volatile pairs |

### Lending Contract

Over-collateralized lending with a kinked interest rate model.

**Key Features:**
- Supply assets and earn interest
- Borrow against collateral
- Dynamic interest rates based on utilization
- Health factor monitoring
- Liquidation threshold enforcement
- Protocol reserve factor

**Interest Rate Model:**
```
if utilization <= optimal:
    borrow_rate = base_rate + (utilization * slope / optimal)
else:
    borrow_rate = base_rate + slope + ((utilization - optimal) * slope / (1 - optimal))

supply_rate = borrow_rate * utilization * (1 - reserve_factor)
```

**Risk Parameters:**
| Parameter | Value | Description |
|-----------|-------|-------------|
| Collateral Factor | 75% | Max borrow against collateral value |
| Liquidation Threshold | 80% | Health factor < 1.0 triggers liquidation |
| Reserve Factor | Configurable | Protocol fee on interest |

---

## API Reference

### Swap Contract

#### `initialize(admin, token_a, token_b, fee_bps)`
Initialize a new swap pool.

| Parameter | Type | Description |
|-----------|------|-------------|
| `admin` | `Address` | Pool admin address |
| `token_a` | `Address` | First token address |
| `token_b` | `Address` | Second token address |
| `fee_bps` | `u128` | Fee in basis points (max 1000) |

#### `get_pool_info() → PoolInfo`
Returns current pool state including reserves, fee, and total shares.

#### `get_swap_quote(amount_in, a_to_b) → SwapQuote`
Calculate swap output without executing. Returns `amount_out`, `fee`, and `price_impact_pct`.

#### `swap(from, amount_in, min_amount_out, a_to_b) → u128`
Execute a swap. Returns actual `amount_out`. Requires auth from `from`.

#### `add_liquidity(provider, amount_a, amount_b) → u128`
Add liquidity to the pool. Returns LP shares minted.

#### `remove_liquidity(provider, shares) → (u128, u128)`
Remove liquidity. Returns `(amount_a, amount_b)` withdrawn.

### Liquidity Contract

#### `initialize(admin)`
Initialize the liquidity pool manager.

#### `create_pool(token_a, token_b, fee_tier_bps, tick_spacing, initial_sqrt_price) → u32`
Create a new pool. Returns pool ID.

#### `get_pool(pool_id) → PoolConfig`
Get pool configuration.

#### `add_liquidity(owner, pool_id, tick_lower, tick_upper, amount_a, amount_b) → AddLiquidityResult`
Add concentrated liquidity within a tick range.

#### `remove_liquidity(owner, position_id, liquidity_to_remove) → RemoveLiquidityResult`
Remove liquidity from a position.

#### `collect_fees(owner, position_id) → (u128, u128)`
Collect accumulated fees.

### Lending Contract

#### `initialize(admin, asset, reserve_factor_bps, base_rate_bps, slope_bps, optimal_utilization)`
Initialize the lending pool with interest rate parameters.

#### `get_pool_info() → LendingPool`
Returns current pool state.

#### `get_utilization_rate() → u128`
Current utilization rate in BPS.

#### `get_supply_apy() → u128`
Current supply APY in BPS.

#### `get_borrow_apy() → u128`
Current borrow APY in BPS.

#### `supply(supplier, amount) → SupplyResult`
Supply assets. Returns shares and exchange rate.

#### `borrow(borrower, amount) → BorrowResult`
Borrow assets. Returns amount, health factor, and utilization.

#### `deposit_collateral(depositor, amount) → u128`
Deposit collateral. Returns total collateral value.

#### `get_user_position(user) → UserPosition`
Get user's deposit, borrow, and collateral state.

#### `get_health_factor(user) → u128`
Health factor (10000 = 1.0x, safe).

---

## Getting Started

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Soroban CLI
cargo install --locked soroban-cli

# Install Stellar CLI
cargo install --locked stellar-cli
```

### Build

```bash
# Clone the repository
git clone https://github.com/sudo-robi/web3-suite-defi-contracts.git
cd web3-suite-defi-contracts

# Build all contracts
cargo build --all

# Build for WASM target (for deployment)
stellar contract build --target wasm contracts/swap
stellar contract build --target wasm contracts/liquidity
stellar contract build --target wasm contracts/lending
```

### Test

```bash
# Run all tests
cargo test --all

# Run tests for a specific contract
cargo test -p swap-contract
cargo test -p liquidity-contract
cargo test -p lending-contract

# Run with output
cargo test --all -- --nocapture
```

### Deploy

```bash
# Deploy to Stellar Testnet
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/swap_contract.wasm \
  --source-account YOUR_SECRET \
  --network testnet

# Initialize the contract
stellar contract invoke \
  --contract-id CONTRACT_ID \
  --source-account YOUR_SECRET \
  --network testnet \
  -- initialize \
  --admin YOUR_ADDRESS \
  --token_a TOKEN_A_ADDRESS \
  --token_b TOKEN_B_ADDRESS \
  --fee_bps 30
```

### Using with Stellar CLI

```bash
# Query pool info
stellar contract invoke \
  --contract-id CONTRACT_ID \
  --network testnet \
  -- get_pool_info

# Get a swap quote
stellar contract invoke \
  --contract-id CONTRACT_ID \
  --network testnet \
  -- get_swap_quote \
  --amount_in 1000 \
  --a_to_b true
```

---

## Project Structure

```
web3-suite-defi-contracts/
├── Cargo.toml                    # Workspace configuration
├── README.md
├── LICENSE                       # MIT License
├── CONTRIBUTING.md
├── .gitignore
└── contracts/
    ├── swap/
    │   ├── Cargo.toml
    │   └── src/
    │       └── lib.rs            # AMM swap contract
    ├── liquidity/
    │   ├── Cargo.toml
    │   └── src/
    │       └── lib.rs            # Liquidity pool contract
    └── lending/
        ├── Cargo.toml
        └── src/
            └── lib.rs            # Lending pool contract
```

---

## Security Considerations

- **Overflow Protection:** All arithmetic uses `checked_*` operations
- **Authorization:** All state-changing functions require `require_auth()`
- **Input Validation:** All inputs validated at contract boundary
- **Fee Caps:** Maximum fee enforced at initialization
- **Health Factors:** Lending positions monitored for solvency
- **Immutable Core Logic:** Contracts cannot be modified post-deployment

### Known Limitations

- Exchange rate calculations are simplified for initial release
- Interest accrual is not real-time (updated on interaction)
- LP token is implicit (shares tracked in pool state, not separate token)
- Liquidation mechanics are placeholder (TODO: full liquidation engine)

---

## Testing

The test suite covers:

- **Initialization:** Pool setup, parameter validation
- **Swap Operations:** Quote calculation, execution, slippage protection
- **Liquidity:** Add/remove, share calculation, fee distribution
- **Lending:** Supply, borrow, collateral, health factors
- **Edge Cases:** Zero amounts, overflow, unauthorized access
- **Interest Rates:** Utilization-based rate model validation

```bash
# Coverage report (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --all
```

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on:

- Development setup
- Code style and conventions
- Testing requirements
- Pull request process
- Security reporting

---

## License

This project is licensed under the MIT License — see [LICENSE](LICENSE) for details.

---

## Acknowledgments

- [Stellar Development Foundation](https://stellar.org) for the Soroban platform
- [soroban-sdk](https://soroban.stellar.org/docs) documentation
- DeFi protocol research from Uniswap V2/V3, Aave, and Compound
