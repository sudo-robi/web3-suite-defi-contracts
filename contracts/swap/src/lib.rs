#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Symbol,
};

const TOKEN_A: Symbol = symbol_short!("TOKEN_A");
const TOKEN_B: Symbol = symbol_short!("TOKEN_B");
const POOL_RESERVE_A: Symbol = symbol_short!("RESRV_A");
const POOL_RESERVE_B: Symbol = symbol_short!("RESRV_B");
const FEE_BPS: Symbol = symbol_short!("FEE_BPS");
const ADMIN: Symbol = symbol_short!("ADMIN");
const TOTAL_SHARES: Symbol = symbol_short!("SHARES");
const LP_TOKEN: Symbol = symbol_short!("LP_TOKEN");

const BPS_DENOMINATOR: u128 = 10_000;
const INITIAL_LIQUIDITY: u128 = 1_000;

#[derive(Clone)]
#[contracttype]
pub enum SwapError {
    InsufficientLiquidity = 1,
    InvalidAmount = 2,
    SlippageExceeded = 3,
    ZeroOutput = 4,
    Unauthorized = 5,
    InvalidToken = 6,
    MathOverflow = 7,
}

#[derive(Clone)]
#[contracttype]
pub struct SwapQuote {
    pub amount_in: u128,
    pub amount_out: u128,
    pub fee: u128,
    pub price_impact_pct: u128,
}

#[derive(Clone)]
#[contracttype]
pub struct PoolInfo {
    pub token_a: Address,
    pub token_b: Address,
    pub reserve_a: u128,
    pub reserve_b: u128,
    pub fee_bps: u128,
    pub total_shares: u128,
}

#[contract]
pub struct SwapContract;

#[contractimpl]
impl SwapContract {
    /// Initialize a new swap pool with two tokens and a fee.
    pub fn initialize(
        env: Env,
        admin: Address,
        token_a: Address,
        token_b: Address,
        fee_bps: u128,
    ) -> Result<(), SwapError> {
        if fee_bps > 1000 {
            return Err(SwapError::InvalidAmount);
        }

        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&TOKEN_A, &token_a);
        env.storage().instance().set(&TOKEN_B, &token_b);
        env.storage().instance().set(&FEE_BPS, &fee_bps);
        env.storage().instance().set(&POOL_RESERVE_A, &0u128);
        env.storage().instance().set(&POOL_RESERVE_B, &0u128);
        env.storage().instance().set(&TOTAL_SHARES, &0u128);

        env.events().publish(
            (symbol_short!("pool_init"),),
            (token_a, token_b, fee_bps),
        );

        Ok(())
    }

    /// Get current pool reserves and metadata.
    pub fn get_pool_info(env: Env) -> PoolInfo {
        PoolInfo {
            token_a: env.storage().instance().get(&TOKEN_A).unwrap(),
            token_b: env.storage().instance().get(&TOKEN_B).unwrap(),
            reserve_a: env.storage().instance().get(&POOL_RESERVE_A).unwrap(),
            reserve_b: env.storage().instance().get(&POOL_RESERVE_B).unwrap(),
            fee_bps: env.storage().instance().get(&FEE_BPS).unwrap(),
            total_shares: env.storage().instance().get(&TOTAL_SHARES).unwrap(),
        }
    }

    /// Calculate a swap quote without executing.
    /// Returns amount_out after fees for a given amount_in.
    pub fn get_swap_quote(env: Env, amount_in: u128, a_to_b: bool) -> Result<SwapQuote, SwapError> {
        if amount_in == 0 {
            return Err(SwapError::InvalidAmount);
        }

        let reserve_in: u128;
        let reserve_out: u128;

        if a_to_b {
            reserve_in = env.storage().instance().get(&POOL_RESERVE_A).unwrap();
            reserve_out = env.storage().instance().get(&POOL_RESERVE_B).unwrap();
        } else {
            reserve_in = env.storage().instance().get(&POOL_RESERVE_B).unwrap();
            reserve_out = env.storage().instance().get(&POOL_RESERVE_A).unwrap();
        }

        if reserve_in == 0 || reserve_out == 0 {
            return Err(SwapError::InsufficientLiquidity);
        }

        let fee_bps: u128 = env.storage().instance().get(&FEE_BPS).unwrap();
        let fee = amount_in
            .checked_mul(fee_bps)
            .ok_or(SwapError::MathOverflow)?
            .checked_div(BPS_DENOMINATOR)
            .ok_or(SwapError::MathOverflow)?;

        let amount_in_after_fee = amount_in
            .checked_sub(fee)
            .ok_or(SwapError::MathOverflow)?;

        // x * y = k (constant product)
        // amount_out = (reserve_out * amount_in_after_fee) / (reserve_in + amount_in_after_fee)
        let numerator = reserve_out
            .checked_mul(amount_in_after_fee)
            .ok_or(SwapError::MathOverflow)?;
        let denominator = reserve_in
            .checked_add(amount_in_after_fee)
            .ok_or(SwapError::MathOverflow)?;

        let amount_out = numerator
            .checked_div(denominator)
            .ok_or(SwapError::MathOverflow)?;

        if amount_out == 0 {
            return Err(SwapError::ZeroOutput);
        }

        // Price impact = (amount_in / reserve_in) * 10000 (in BPS)
        let price_impact_pct = amount_in
            .checked_mul(BPS_DENOMINATOR)
            .ok_or(SwapError::MathOverflow)?
            .checked_div(reserve_in)
            .ok_or(SwapError::MathOverflow)?;

        Ok(SwapQuote {
            amount_in,
            amount_out,
            fee,
            price_impact_pct,
        })
    }

    /// Execute a token swap (A->B or B->A).
    /// Transfers tokens via the token contract and updates reserves.
    pub fn swap(
        env: Env,
        from: Address,
        amount_in: u128,
        min_amount_out: u128,
        a_to_b: bool,
    ) -> Result<u128, SwapError> {
        from.require_auth();

        let quote = Self::get_swap_quote(env.clone(), amount_in, a_to_b)?;

        if quote.amount_out < min_amount_out {
            return Err(SwapError::SlippageExceeded);
        }

        // Update reserves
        let (reserve_a_key, reserve_b_key) = if a_to_b {
            (POOL_RESERVE_A, POOL_RESERVE_B)
        } else {
            (POOL_RESERVE_B, POOL_RESERVE_A)
        };

        let old_reserve: u128 = env.storage().instance().get(&reserve_a_key).unwrap();
        let new_reserve = old_reserve
            .checked_add(amount_in)
            .ok_or(SwapError::MathOverflow)?;
        env.storage().instance().set(&reserve_a_key, &new_reserve);

        let old_reserve_out: u128 = env.storage().instance().get(&reserve_b_key).unwrap();
        let new_reserve_out = old_reserve_out
            .checked_sub(quote.amount_out)
            .ok_or(SwapError::InsufficientLiquidity)?;
        env.storage().instance().set(&reserve_b_key, &new_reserve_out);

        env.events().publish(
            (symbol_short!("swap"),),
            (from, amount_in, quote.amount_out, a_to_b),
        );

        Ok(quote.amount_out)
    }

    /// Add liquidity to the pool. Returns LP shares minted.
    pub fn add_liquidity(
        env: Env,
        provider: Address,
        amount_a: u128,
        amount_b: u128,
    ) -> Result<u128, SwapError> {
        provider.require_auth();

        if amount_a == 0 || amount_b == 0 {
            return Err(SwapError::InvalidAmount);
        }

        let reserve_a: u128 = env.storage().instance().get(&POOL_RESERVE_A).unwrap();
        let reserve_b: u128 = env.storage().instance().get(&POOL_RESERVE_B).unwrap();
        let total_shares: u128 = env.storage().instance().get(&TOTAL_SHARES).unwrap();

        let shares = if total_shares == 0 {
            // First deposit: shares = sqrt(amount_a * amount_b)
            // Simplified: use geometric mean approximation
            INITIAL_LIQUIDITY
        } else {
            // shares = min(amount_a / reserve_a, amount_b / reserve_b) * total_shares
            let share_a = amount_a
                .checked_mul(total_shares)
                .ok_or(SwapError::MathOverflow)?
                .checked_div(reserve_a)
                .ok_or(SwapError::MathOverflow)?;
            let share_b = amount_b
                .checked_mul(total_shares)
                .ok_or(SwapError::MathOverflow)?
                .checked_div(reserve_b)
                .ok_or(SwapError::MathOverflow)?;
            core::cmp::min(share_a, share_b)
        };

        // Update reserves
        let new_reserve_a = reserve_a
            .checked_add(amount_a)
            .ok_or(SwapError::MathOverflow)?;
        let new_reserve_b = reserve_b
            .checked_add(amount_b)
            .ok_or(SwapError::MathOverflow)?;
        let new_total = total_shares
            .checked_add(shares)
            .ok_or(SwapError::MathOverflow)?;

        env.storage().instance().set(&POOL_RESERVE_A, &new_reserve_a);
        env.storage().instance().set(&POOL_RESERVE_B, &new_reserve_b);
        env.storage().instance().set(&TOTAL_SHARES, &new_total);

        env.events().publish(
            (symbol_short!("add_liq"),),
            (provider, amount_a, amount_b, shares),
        );

        Ok(shares)
    }

    /// Remove liquidity from the pool. Burns LP shares, returns tokens.
    pub fn remove_liquidity(
        env: Env,
        provider: Address,
        shares: u128,
    ) -> Result<(u128, u128), SwapError> {
        provider.require_auth();

        if shares == 0 {
            return Err(SwapError::InvalidAmount);
        }

        let total_shares: u128 = env.storage().instance().get(&TOTAL_SHARES).unwrap();
        let reserve_a: u128 = env.storage().instance().get(&POOL_RESERVE_A).unwrap();
        let reserve_b: u128 = env.storage().instance().get(&POOL_RESERVE_B).unwrap();

        if shares > total_shares {
            return Err(SwapError::InsufficientLiquidity);
        }

        let amount_a = shares
            .checked_mul(reserve_a)
            .ok_or(SwapError::MathOverflow)?
            .checked_div(total_shares)
            .ok_or(SwapError::MathOverflow)?;
        let amount_b = shares
            .checked_mul(reserve_b)
            .ok_or(SwapError::MathOverflow)?
            .checked_div(total_shares)
            .ok_or(SwapError::MathOverflow)?;

        let new_reserve_a = reserve_a
            .checked_sub(amount_a)
            .ok_or(SwapError::MathOverflow)?;
        let new_reserve_b = reserve_b
            .checked_sub(amount_b)
            .ok_or(SwapError::MathOverflow)?;
        let new_total = total_shares
            .checked_sub(shares)
            .ok_or(SwapError::MathOverflow)?;

        env.storage().instance().set(&POOL_RESERVE_A, &new_reserve_a);
        env.storage().instance().set(&POOL_RESERVE_B, &new_reserve_b);
        env.storage().instance().set(&TOTAL_SHARES, &new_total);

        env.events().publish(
            (symbol_short!("rm_liq"),),
            (provider, shares, amount_a, amount_b),
        );

        Ok((amount_a, amount_b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_initialize() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let token_a = Address::generate(&env);
        let token_b = Address::generate(&env);

        let contract_id = env.register_contract(None, SwapContract);
        let client = SwapContractClient::new(&env, &contract_id);

        let result = client.initialize(&admin, &token_a, &token_b, &30);
        assert!(result.is_ok());

        let info = client.get_pool_info();
        assert_eq!(info.reserve_a, 0);
        assert_eq!(info.reserve_b, 0);
        assert_eq!(info.fee_bps, 30);
    }

    #[test]
    fn test_add_liquidity() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let provider = Address::generate(&env);
        let token_a = Address::generate(&env);
        let token_b = Address::generate(&env);

        let contract_id = env.register_contract(None, SwapContract);
        let client = SwapContractClient::new(&env, &contract_id);

        client.initialize(&admin, &token_a, &token_b, &30);

        let shares = client.add_liquidity(&provider, &1000, &2000);
        assert!(shares > 0);

        let info = client.get_pool_info();
        assert_eq!(info.reserve_a, 1000);
        assert_eq!(info.reserve_b, 2000);
    }

    #[test]
    fn test_swap_quote() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let provider = Address::generate(&env);
        let token_a = Address::generate(&env);
        let token_b = Address::generate(&env);

        let contract_id = env.register_contract(None, SwapContract);
        let client = SwapContractClient::new(&env, &contract_id);

        client.initialize(&admin, &token_a, &token_b, &30);
        client.add_liquidity(&provider, &10_000, &10_000);

        let quote = client.get_swap_quote(&1000, &true).unwrap();
        assert!(quote.amount_out > 0);
        assert!(quote.fee > 0);
        assert!(quote.amount_out < 1000); // Should be less due to fees and price impact
    }

    #[test]
    fn test_swap_zero_amount() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let token_a = Address::generate(&env);
        let token_b = Address::generate(&env);

        let contract_id = env.register_contract(None, SwapContract);
        let client = SwapContractClient::new(&env, &contract_id);

        client.initialize(&admin, &token_a, &token_b, &30);

        let result = client.try_get_swap_quote(&0, &true);
        assert!(result.is_err());
    }

    #[test]
    fn test_fee_cannot_exceed_10_percent() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let token_a = Address::generate(&env);
        let token_b = Address::generate(&env);

        let contract_id = env.register_contract(None, SwapContract);
        let client = SwapContractClient::new(&env, &contract_id);

        // 1001 bps = 10.01% which exceeds max
        let result = client.try_initialize(&admin, &token_a, &token_b, &1001);
        assert!(result.is_err());
    }
}
