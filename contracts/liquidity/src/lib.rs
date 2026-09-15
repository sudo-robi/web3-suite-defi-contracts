#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol,
};

const ADMIN: Symbol = symbol_short!("ADMIN");
const POOL_COUNT: Symbol = symbol_short!("PL_COUNT");
const FEE_TIERS: Symbol = symbol_short!("FEE_TIRS");

#[derive(Clone)]
#[contracttype]
pub enum LiquidityError {
    InsufficientLiquidity = 1,
    InvalidAmount = 2,
    InvalidRange = 3,
    PositionNotFound = 4,
    Unauthorized = 5,
    PoolFull = 6,
    MathOverflow = 7,
}

#[derive(Clone)]
#[contracttype]
pub struct PoolConfig {
    pub token_a: Address,
    pub token_b: Address,
    pub fee_tier_bps: u128,
    pub tick_spacing: u32,
    pub total_liquidity: u128,
    pub sqrt_price: u128, // Q64.64 fixed-point
}

#[derive(Clone)]
#[contracttype]
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

#[derive(Clone)]
#[contracttype]
pub struct AddLiquidityResult {
    pub position_id: u64,
    pub amount_a: u128,
    pub amount_b: u128,
    pub liquidity_minted: u128,
}

#[derive(Clone)]
#[contracttype]
pub struct RemoveLiquidityResult {
    pub amount_a: u128,
    pub amount_b: u128,
    pub fees_a: u128,
    pub fees_b: u128,
}

#[contract]
pub struct LiquidityContract;

#[contractimpl]
impl LiquidityContract {
    /// Initialize the liquidity pool manager.
    pub fn initialize(env: Env, admin: Address) -> Result<(), LiquidityError> {
        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&POOL_COUNT, &0u32);

        // Default fee tiers: 1 bps, 5 bps, 30 bps, 100 bps
        let tiers: soroban_sdk::Vec<u128> = soroban_sdk::vec![&env, 1, 5, 30, 100];
        env.storage().instance().set(&FEE_TIERS, &tiers);

        Ok(())
    }

    /// Create a new liquidity pool for a token pair at a given fee tier.
    pub fn create_pool(
        env: Env,
        token_a: Address,
        token_b: Address,
        fee_tier_bps: u128,
        tick_spacing: u32,
        initial_sqrt_price: u128,
    ) -> Result<u32, LiquidityError> {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();

        let pool_count: u32 = env.storage().instance().get(&POOL_COUNT).unwrap();
        let pool_id = pool_count
            .checked_add(1)
            .ok_or(LiquidityError::MathOverflow)?;

        let pool = PoolConfig {
            token_a,
            token_b,
            fee_tier_bps,
            tick_spacing,
            total_liquidity: 0,
            sqrt_price: initial_sqrt_price,
        };

        env.storage()
            .instance()
            .set(&(POOL_COUNT, pool_id), &pool);
        env.storage().instance().set(&POOL_COUNT, &pool_id);

        env.events().publish(
            (symbol_short!("pool_cre"),),
            (pool_id, fee_tier_bps, tick_spacing),
        );

        Ok(pool_id)
    }

    /// Get pool configuration by ID.
    pub fn get_pool(env: Env, pool_id: u32) -> Result<PoolConfig, LiquidityError> {
        env.storage()
            .instance()
            .get(&(POOL_COUNT, pool_id))
            .ok_or(LiquidityError::PositionNotFound)
    }

    /// Get total number of pools.
    pub fn get_pool_count(env: Env) -> u32 {
        env.storage().instance().get(&POOL_COUNT).unwrap_or(0)
    }

    /// Add liquidity to a pool within a tick range (concentrated liquidity).
    pub fn add_liquidity(
        env: Env,
        owner: Address,
        pool_id: u32,
        tick_lower: i32,
        tick_upper: i32,
        amount_a: u128,
        amount_b: u128,
    ) -> Result<AddLiquidityResult, LiquidityError> {
        owner.require_auth();

        if tick_lower >= tick_upper {
            return Err(LiquidityError::InvalidRange);
        }
        if amount_a == 0 && amount_b == 0 {
            return Err(LiquidityError::InvalidAmount);
        }

        let mut pool: PoolConfig = env
            .storage()
            .instance()
            .get(&(POOL_COUNT, pool_id))
            .ok_or(LiquidityError::PositionNotFound)?;

        // Calculate liquidity from amounts
        // Simplified: L = amount_a * sqrt(price) for token A
        let liquidity = amount_a
            .checked_add(amount_b)
            .ok_or(LiquidityError::MathOverflow)?;

        pool.total_liquidity = pool
            .total_liquidity
            .checked_add(liquidity)
            .ok_or(LiquidityError::MathOverflow)?;

        let position_id: u64 = env.storage().instance().get(&POOL_COUNT).unwrap_or(0u32) as u64 + 1;

        let position = LiquidityPosition {
            id: position_id,
            owner: owner.clone(),
            pool_id,
            tick_lower,
            tick_upper,
            liquidity,
            fee_growth_inside: 0,
            tokens_owed_a: 0,
            tokens_owed_b: 0,
        };

        env.storage()
            .instance()
            .set(&(FEE_TIERS, position_id), &position);
        env.storage()
            .instance()
            .set(&(POOL_COUNT, pool_id), &pool);

        env.events().publish(
            (symbol_short!("liq_add"),),
            (position_id, pool_id, liquidity, tick_lower, tick_upper),
        );

        Ok(AddLiquidityResult {
            position_id,
            amount_a,
            amount_b,
            liquidity_minted: liquidity,
        })
    }

    /// Remove liquidity from a position.
    pub fn remove_liquidity(
        env: Env,
        owner: Address,
        position_id: u64,
        liquidity_to_remove: u128,
    ) -> Result<RemoveLiquidityResult, LiquidityError> {
        owner.require_auth();

        let mut position: LiquidityPosition = env
            .storage()
            .instance()
            .get(&(FEE_TIERS, position_id))
            .ok_or(LiquidityError::PositionNotFound)?;

        if position.owner != owner {
            return Err(LiquidityError::Unauthorized);
        }

        if liquidity_to_remove > position.liquidity {
            return Err(LiquidityError::InsufficientLiquidity);
        }

        // Calculate proportional token amounts
        let mut pool: PoolConfig = env
            .storage()
            .instance()
            .get(&(POOL_COUNT, position.pool_id))
            .ok_or(LiquidityError::PositionNotFound)?;

        let total_liq = pool.total_liquidity;
        let amount_a = if total_liq > 0 {
            liquidity_to_remove
                .checked_mul(1000)
                .ok_or(LiquidityError::MathOverflow)?
                .checked_div(total_liq)
                .ok_or(LiquidityError::MathOverflow)?
        } else {
            0
        };
        let amount_b = amount_a; // Simplified

        position.liquidity = position
            .liquidity
            .checked_sub(liquidity_to_remove)
            .ok_or(LiquidityError::MathOverflow)?;

        pool.total_liquidity = pool
            .total_liquidity
            .checked_sub(liquidity_to_remove)
            .ok_or(LiquidityError::MathOverflow)?;

        let fees_a = position.tokens_owed_a;
        let fees_b = position.tokens_owed_b;
        position.tokens_owed_a = 0;
        position.tokens_owed_b = 0;

        env.storage()
            .instance()
            .set(&(FEE_TIERS, position_id), &position);
        env.storage()
            .instance()
            .set(&(POOL_COUNT, position.pool_id), &pool);

        env.events().publish(
            (symbol_short!("liq_rem"),),
            (position_id, liquidity_to_remove, amount_a, amount_b),
        );

        Ok(RemoveLiquidityResult {
            amount_a,
            amount_b,
            fees_a,
            fees_b,
        })
    }

    /// Collect accumulated fees from a position.
    pub fn collect_fees(
        env: Env,
        owner: Address,
        position_id: u64,
    ) -> Result<(u128, u128), LiquidityError> {
        owner.require_auth();

        let mut position: LiquidityPosition = env
            .storage()
            .instance()
            .get(&(FEE_TIERS, position_id))
            .ok_or(LiquidityError::PositionNotFound)?;

        if position.owner != owner {
            return Err(LiquidityError::Unauthorized);
        }

        let fees_a = position.tokens_owed_a;
        let fees_b = position.tokens_owed_b;
        position.tokens_owed_a = 0;
        position.tokens_owed_b = 0;

        env.storage()
            .instance()
            .set(&(FEE_TIERS, position_id), &position);

        env.events().publish(
            (symbol_short!("fee_col"),),
            (position_id, fees_a, fees_b),
        );

        Ok((fees_a, fees_b))
    }

    /// Get available fee tiers.
    pub fn get_fee_tiers(env: Env) -> soroban_sdk::Vec<u128> {
        env.storage()
            .instance()
            .get(&FEE_TIERS)
            .unwrap_or(soroban_sdk::vec![&env])
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

        let contract_id = env.register_contract(None, LiquidityContract);
        let client = LiquidityContractClient::new(&env, &contract_id);

        client.initialize(&admin);
        assert_eq!(client.get_pool_count(), 0);
    }

    #[test]
    fn test_create_pool() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let token_a = Address::generate(&env);
        let token_b = Address::generate(&env);

        let contract_id = env.register_contract(None, LiquidityContract);
        let client = LiquidityContractClient::new(&env, &contract_id);

        client.initialize(&admin);

        let pool_id = client.create_pool(&token_a, &token_b, &30, &1, &1000000);
        assert_eq!(pool_id, 1);
        assert_eq!(client.get_pool_count(), 1);

        let pool = client.get_pool(&1).unwrap();
        assert_eq!(pool.fee_tier_bps, 30);
        assert_eq!(pool.total_liquidity, 0);
    }

    #[test]
    fn test_add_liquidity() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let owner = Address::generate(&env);
        let token_a = Address::generate(&env);
        let token_b = Address::generate(&env);

        let contract_id = env.register_contract(None, LiquidityContract);
        let client = LiquidityContractClient::new(&env, &contract_id);

        client.initialize(&admin);
        let pool_id = client.create_pool(&token_a, &token_b, &30, &1, &1000000);

        let result = client.add_liquidity(&owner, &pool_id, &-100, &100, &5000, &5000);
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.amount_a, 5000);
        assert_eq!(result.amount_b, 5000);
    }

    #[test]
    fn test_invalid_tick_range() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let owner = Address::generate(&env);
        let token_a = Address::generate(&env);
        let token_b = Address::generate(&env);

        let contract_id = env.register_contract(None, LiquidityContract);
        let client = LiquidityContractClient::new(&env, &contract_id);

        client.initialize(&admin);
        let pool_id = client.create_pool(&token_a, &token_b, &30, &1, &1000000);

        // tick_lower >= tick_upper should fail
        let result = client.try_add_liquidity(&owner, &pool_id, &100, &100, &5000, &5000);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_liquidity() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let owner = Address::generate(&env);
        let token_a = Address::generate(&env);
        let token_b = Address::generate(&env);

        let contract_id = env.register_contract(None, LiquidityContract);
        let client = LiquidityContractClient::new(&env, &contract_id);

        client.initialize(&admin);
        let pool_id = client.create_pool(&token_a, &token_b, &30, &1, &1000000);

        let add_result = client.add_liquidity(&owner, &pool_id, &-100, &100, &5000, &5000).unwrap();
        let remove_result = client.remove_liquidity(&owner, &add_result.position_id, &add_result.liquidity_minted);
        assert!(remove_result.is_ok());
    }
}
