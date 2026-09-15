# Contributing to web3-suite-defi-contracts

Thank you for your interest in contributing! This guide will help you get started.

## Development Setup

### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- [Soroban CLI](https://soroban.stellar.org/docs/getting-started/setup)
- [Stellar CLI](https://developers.stellar.org/docs/smart-contracts/getting-started)
- Git

### Getting Started

```bash
# Clone the repository
git clone https://github.com/sudo-robi/web3-suite-defi-contracts.git
cd web3-suite-defi-contracts

# Build all contracts
cargo build --all

# Run all tests
cargo test --all

# Build for a specific network
stellar contract build --target wasm contracts/swap
```

## Architecture

We follow a workspace-based monorepo structure. Each contract is an independent crate under `contracts/`.

### Adding a New Contract

1. Create a new directory under `contracts/your-contract/`
2. Add `Cargo.toml` with `crate-type = ["cdylib"]`
3. Implement the `#[contract]` and `#[contractimpl]` traits
4. Add entry points under `contracts/your-contract/src/`
5. Write tests in `contracts/your-contract/tests/`
6. Update the workspace `Cargo.toml` if needed

### Contract Guidelines

- **Single responsibility**: Each contract does one thing well
- **Input validation**: Validate all inputs at the contract boundary
- **Error handling**: Use typed errors, never panic in production
- **Events**: Emit events for all state changes
- **Immutability**: Contracts should be immutable after deployment; use upgradeable patterns only when necessary
- **Gas efficiency**: Minimize storage reads/writes; cache values locally

### Code Style

- Follow `rustfmt` defaults
- Use `clippy` lint warnings as errors
- Document all public functions with `///` doc comments
- Use meaningful variable names; avoid single-letter names except in iterators

```bash
# Format code
cargo fmt --all

# Lint
cargo clippy --all -- -D warnings
```

## Testing

### Unit Tests

Place unit tests in `tests/` directories within each contract crate:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_basic_operation() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MyContract);
        let client = MyContractClient::new(&env, &contract_id);

        // Test your contract
    }
}
```

### Integration Tests

Integration tests that span multiple contracts go in `tests/integration/`.

### Running Tests

```bash
# All tests
cargo test --all

# Specific contract tests
cargo test -p swap-contract

# With output
cargo test --all -- --nocapture
```

## Pull Request Process

1. Fork the repository
2. Create a feature branch from `main`
3. Make your changes
4. Ensure all tests pass: `cargo test --all`
5. Ensure code is formatted: `cargo fmt --all`
6. Ensure no lint warnings: `cargo clippy --all -- -D warnings`
7. Submit a pull request with a clear description

### Commit Messages

Use conventional commits:

- `feat: add flash loan functionality`
- `fix: correct slippage calculation in swap`
- `docs: update API reference`
- `test: add edge case tests for lending`
- `refactor: extract common math utilities`

## Security

- Never commit private keys or secrets
- Report security vulnerabilities privately to the maintainers
- All contract changes undergo security review before merge
- Follow the principle of least privilege

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
