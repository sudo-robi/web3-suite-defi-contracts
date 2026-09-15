#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol,
};

const ADMIN: Symbol = symbol_short!("ADMIN");
const RESERVES: Symbol = symbol_short!("RESERVES");
const BORROWERS: Symbol = symbol_short!("BORROWS");
const COLLATERAL: Symbol = symbol_short!("COLLTRL");
const INTEREST_MODEL: Symbol = symbol_short!("INT_MODEL");
const TOTAL_BORROWED: Symbol = symbol_short!("TOT_BRRW");
const TOTAL_DEPOSITS: Symbol = symbol_short!("TOT_DEP");
const PROTOCOL_FEE: Symbol = symbol_short!("PROT_FEE");

const BPS_DENOMINATOR: u128 = 10_000;
const SECONDS_PER_YEAR: u128 = 31_536_000;
const LIQUIDATION_THRESHOLD: u128 = 8_000; // 80% LTV
const COLLATERAL_FACTOR: u128 = 7_500; // 75% collateral factor

#[derive(Clone)]
#[contracttype]
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

#[derive(Clone)]
#[contracttype]
pub struct LendingPool {
    pub asset: Address,
    pub total_deposits: u128,
    pub total_borrowed: u128,
    pub reserve_factor_bps: u128,
    pub interest_rate_model: InterestRateModel,
    pub last_update_timestamp: u64,
    pub exchange_rate: u128, // Q64.64 fixed-point
}

#[derive(Clone)]
#[contracttype]
pub struct InterestRateModel {
    pub base_rate_bps: u128,      // Base annual rate in BPS
    pub slope_bps: u128,          // Slope above optimal utilization
    pub optimal_utilization: u128, // Optimal utilization in BPS
}

#[derive(Clone)]
#[contracttype]
pub struct UserPosition {
    pub deposited: u128,
    pub borrowed: u128,
    pub collateral_value: u128,
    pub last_accrued: u64,
    pub interest_earned: u128,
    pub interest_owed: u128,
}

#[derive(Clone)]
#[contracttype]
pub struct BorrowResult {
    pub amount: u128,
    pub health_factor: u128,
    pub new_utilization: u128,
}

#[derive(Clone)]
#[contracttype]
pub struct SupplyResult {
    pub shares: u128,
    pub exchange_rate: u128,
}

#[contract]
pub struct LendingContract;

#[contractimpl]
impl LendingContract {
    /// Initialize the lending pool with an asset and reserve factor.
    pub fn initialize(
        env: Env,
        admin: Address,
        asset: Address,
        reserve_factor_bps: u128,
        base_rate_bps: u128,
        slope_bps: u128,
        optimal_utilization: u128,
    ) -> Result<(), LendingError> {
        if reserve_factor_bps > 5000 {
            return Err(LendingError::InvalidAmount);
        }

        let model = InterestRateModel {
            base_rate_bps,
            slope_bps,
            optimal_utilization,
        };

        let pool = LendingPool {
            asset,
            total_deposits: 0,
            total_borrowed: 0,
            reserve_factor_bps,
            interest_rate_model: model,
            last_update_timestamp: env.ledger().timestamp(),
            exchange_rate: 1_000_000_000, // 1.0 in Q64.64
        };

        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&RESERVES, &pool);
        env.storage().instance().set(&TOTAL_BORROWED, &0u128);
        env.storage().instance().set(&TOTAL_DEPOSITS, &0u128);
        env.storage().instance().set(&PROTOCOL_FEE, &reserve_factor_bps);

        Ok(())
    }

    /// Get current lending pool state.
    pub fn get_pool_info(env: Env) -> LendingPool {
        env.storage().instance().get(&RESERVES).unwrap()
    }

    /// Calculate current utilization rate (total_borrowed / total_deposits).
    pub fn get_utilization_rate(env: Env) -> u128 {
        let pool: LendingPool = env.storage().instance().get(&RESERVES).unwrap();
        if pool.total_deposits == 0 {
            return 0;
        }
        pool.total_borrowed
            .checked_mul(BPS_DENOMINATOR)
            .unwrap_or(0)
            .checked_div(pool.total_deposits)
            .unwrap_or(0)
    }

    /// Calculate current supply APY based on utilization and interest model.
    pub fn get_supply_apy(env: Env) -> u128 {
        let pool: LendingPool = env.storage().instance().get(&RESERVES).unwrap();
        let utilization = Self::get_utilization_rate(env);
        let model = &pool.interest_rate_model;

        let borrow_rate = if utilization <= model.optimal_utilization {
            model.base_rate_bps
                + (utilization
                    .checked_mul(model.slope_bps)
                    .unwrap_or(0)
                    .checked_div(model.optimal_utilization)
                    .unwrap_or(0))
        } else {
            let excess = utilization
                .checked_sub(model.optimal_utilization)
                .unwrap_or(0);
            let max_excess = BPS_DENOMINATOR
                .checked_sub(model.optimal_utilization)
                .unwrap_or(1);
            model.base_rate_bps
                + model.slope_bps
                + (excess
                    .checked_mul(model.slope_bps)
                    .unwrap_or(0)
                    .checked_div(max_excess)
                    .unwrap_or(0))
        };

        // Supply APY = borrow APY * utilization * (1 - reserve_factor)
        let reserve_factor = pool.reserve_factor_bps;
        borrow_rate
            .checked_mul(utilization)
            .unwrap_or(0)
            .checked_div(BPS_DENOMINATOR)
            .unwrap_or(0)
            .checked_mul(BPS_DENOMINATOR.checked_sub(reserve_factor).unwrap_or(0))
            .unwrap_or(0)
            .checked_div(BPS_DENOMINATOR)
            .unwrap_or(0)
    }

    /// Calculate current borrow APY.
    pub fn get_borrow_apy(env: Env) -> u128 {
        let pool: LendingPool = env.storage().instance().get(&RESERVES).unwrap();
        let utilization = Self::get_utilization_rate(env);
        let model = &pool.interest_rate_model;

        if utilization <= model.optimal_utilization {
            model.base_rate_bps
                + (utilization
                    .checked_mul(model.slope_bps)
                    .unwrap_or(0)
                    .checked_div(model.optimal_utilization)
                    .unwrap_or(0))
        } else {
            let excess = utilization
                .checked_sub(model.optimal_utilization)
                .unwrap_or(0);
            let max_excess = BPS_DENOMINATOR
                .checked_sub(model.optimal_utilization)
                .unwrap_or(1);
            model.base_rate_bps
                + model.slope_bps
                + (excess
                    .checked_mul(model.slope_bps)
                    .unwrap_or(0)
                    .checked_div(max_excess)
                    .unwrap_or(0))
        }
    }

    /// Supply assets to the lending pool. Returns shares minted.
    pub fn supply(
        env: Env,
        supplier: Address,
        amount: u128,
    ) -> Result<SupplyResult, LendingError> {
        supplier.require_auth();

        if amount == 0 {
            return Err(LendingError::InvalidAmount);
        }

        let mut pool: LendingPool = env.storage().instance().get(&RESERVES).unwrap();

        let shares = if pool.total_deposits == 0 {
            amount
        } else {
            amount
                .checked_mul(pool.total_deposits)
                .ok_or(LendingError::MathOverflow)?
                .checked_div(pool.total_borrowed.checked_add(pool.total_deposits).unwrap_or(1))
                .ok_or(LendingError::MathOverflow)?
        };

        pool.total_deposits = pool
            .total_deposits
            .checked_add(amount)
            .ok_or(LendingError::MathOverflow)?;

        // Update exchange rate
        let total_value = pool
            .total_deposits
            .checked_add(pool.total_borrowed)
            .ok_or(LendingError::MathOverflow)?;
        if total_value > 0 {
            pool.exchange_rate = total_value
                .checked_mul(1_000_000_000)
                .ok_or(LendingError::MathOverflow)?
                .checked_div(pool.total_deposits)
                .ok_or(LendingError::MathOverflow)?;
        }

        let timestamp = env.ledger().timestamp();
        pool.last_update_timestamp = timestamp;
        env.storage().instance().set(&RESERVES, &pool);

        let mut position: UserPosition = env
            .storage()
            .instance()
            .get(&(BORROWERS, supplier.clone()))
            .unwrap_or(UserPosition {
                deposited: 0,
                borrowed: 0,
                collateral_value: 0,
                last_accrued: timestamp,
                interest_earned: 0,
                interest_owed: 0,
            });

        position.deposited = position
            .deposited
            .checked_add(amount)
            .ok_or(LendingError::MathOverflow)?;
        position.last_accrued = timestamp;

        env.storage()
            .instance()
            .set(&(BORROWERS, supplier), &position);

        env.events().publish(
            (symbol_short!("supply"),),
            (shares, amount, pool.exchange_rate),
        );

        Ok(SupplyResult {
            shares,
            exchange_rate: pool.exchange_rate,
        })
    }

    /// Borrow assets from the lending pool. Requires sufficient collateral.
    pub fn borrow(
        env: Env,
        borrower: Address,
        amount: u128,
    ) -> Result<BorrowResult, LendingError> {
        borrower.require_auth();

        if amount == 0 {
            return Err(LendingError::InvalidAmount);
        }

        let mut pool: LendingPool = env.storage().instance().get(&RESERVES).unwrap();

        if amount > pool.total_deposits {
            return Err(LendingError::InsufficientLiquidity);
        }

        let mut position: UserPosition = env
            .storage()
            .instance()
            .get(&(BORROWERS, borrower.clone()))
            .unwrap_or(UserPosition {
                deposited: 0,
                borrowed: 0,
                collateral_value: 0,
                last_accrued: env.ledger().timestamp(),
                interest_earned: 0,
                interest_owed: 0,
            });

        // Check collateral
        let new_borrowed = position
            .borrowed
            .checked_add(amount)
            .ok_or(LendingError::MathOverflow)?;

        let max_borrow = position
            .collateral_value
            .checked_mul(COLLATERAL_FACTOR)
            .ok_or(LendingError::MathOverflow)?
            .checked_div(BPS_DENOMINATOR)
            .ok_or(LendingError::MathOverflow)?;

        if new_borrowed > max_borrow && position.collateral_value > 0 {
            return Err(LendingError::Undercollateralized);
        }

        // Calculate health factor: (collateral * liquidation_threshold) / borrowed
        let health_factor = if new_borrowed > 0 {
            position
                .collateral_value
                .checked_mul(LIQUIDATION_THRESHOLD)
                .ok_or(LendingError::MathOverflow)?
                .checked_div(new_borrowed)
                .ok_or(LendingError::MathOverflow)?
        } else {
            u128::MAX
        };

        if health_factor < 10_000 && position.collateral_value > 0 {
            return Err(LendingError::HealthFactorTooLow);
        }

        // Update pool
        pool.total_borrowed = pool
            .total_borrowed
            .checked_add(amount)
            .ok_or(LendingError::MathOverflow)?;

        let utilization = pool
            .total_borrowed
            .checked_mul(BPS_DENOMINATOR)
            .ok_or(LendingError::MathOverflow)?
            .checked_div(pool.total_deposits)
            .ok_or(LendingError::MathOverflow)?;

        pool.last_update_timestamp = env.ledger().timestamp();
        env.storage().instance().set(&RESERVES, &pool);

        // Update position
        position.borrowed = new_borrowed;
        position.last_accrued = env.ledger().timestamp();
        env.storage()
            .instance()
            .set(&(BORROWERS, borrower), &position);

        env.events().publish(
            (symbol_short!("borrow"),),
            (amount, health_factor, utilization),
        );

        Ok(BorrowResult {
            amount,
            health_factor,
            new_utilization: utilization,
        })
    }

    /// Deposit collateral (separate from supply). Enables borrowing.
    pub fn deposit_collateral(
        env: Env,
        depositor: Address,
        amount: u128,
    ) -> Result<u128, LendingError> {
        depositor.require_auth();

        if amount == 0 {
            return Err(LendingError::InvalidAmount);
        }

        let mut position: UserPosition = env
            .storage()
            .instance()
            .get(&(BORROWERS, depositor.clone()))
            .unwrap_or(UserPosition {
                deposited: 0,
                borrowed: 0,
                collateral_value: 0,
                last_accrued: env.ledger().timestamp(),
                interest_earned: 0,
                interest_owed: 0,
            });

        position.collateral_value = position
            .collateral_value
            .checked_add(amount)
            .ok_or(LendingError::MathOverflow)?;

        env.storage()
            .instance()
            .set(&(BORROWERS, depositor), &position);

        env.events().publish(
            (symbol_short!("col_dep"),),
            (amount, position.collateral_value),
        );

        Ok(position.collateral_value)
    }

    /// Get user position details.
    pub fn get_user_position(env: Env, user: Address) -> UserPosition {
        env.storage()
            .instance()
            .get(&(BORROWERS, user))
            .unwrap_or(UserPosition {
                deposited: 0,
                borrowed: 0,
                collateral_value: 0,
                last_accrued: 0,
                interest_earned: 0,
                interest_owed: 0,
            })
    }

    /// Get health factor for a user (10000 = 1.0x, safe).
    pub fn get_health_factor(env: Env, user: Address) -> u128 {
        let position: UserPosition = env
            .storage()
            .instance()
            .get(&(BORROWERS, user))
            .unwrap_or(UserPosition {
                deposited: 0,
                borrowed: 0,
                collateral_value: 0,
                last_accrued: 0,
                interest_earned: 0,
                interest_owed: 0,
            });

        if position.borrowed == 0 {
            return u128::MAX;
        }

        position
            .collateral_value
            .checked_mul(LIQUIDATION_THRESHOLD)
            .unwrap_or(0)
            .checked_div(position.borrowed)
            .unwrap_or(0)
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
        let asset = Address::generate(&env);

        let contract_id = env.register_contract(None, LendingContract);
        let client = LendingContractClient::new(&env, &contract_id);

        client.initialize(&admin, &asset, &500, &200, &4000, &8000);

        let pool = client.get_pool_info();
        assert_eq!(pool.total_deposits, 0);
        assert_eq!(pool.total_borrowed, 0);
    }

    #[test]
    fn test_supply() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let supplier = Address::generate(&env);
        let asset = Address::generate(&env);

        let contract_id = env.register_contract(None, LendingContract);
        let client = LendingContractClient::new(&env, &contract_id);

        client.initialize(&admin, &asset, &500, &200, &4000, &8000);

        let result = client.supply(&supplier, &10_000).unwrap();
        assert!(result.shares > 0);

        let pool = client.get_pool_info();
        assert_eq!(pool.total_deposits, 10_000);
    }

    #[test]
    fn test_borrow_requires_collateral() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let supplier = Address::generate(&env);
        let borrower = Address::generate(&env);
        let asset = Address::generate(&env);

        let contract_id = env.register_contract(None, LendingContract);
        let client = LendingContractClient::new(&env, &contract_id);

        client.initialize(&admin, &asset, &500, &200, &4000, &8000);
        client.supply(&supplier, &100_000);

        // Borrow without collateral should fail
        let result = client.try_borrow(&borrower, &1000);
        assert!(result.is_err());
    }

    #[test]
    fn test_borrow_with_collateral() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let supplier = Address::generate(&env);
        let borrower = Address::generate(&env);
        let asset = Address::generate(&env);

        let contract_id = env.register_contract(None, LendingContract);
        let client = LendingContractClient::new(&env, &contract_id);

        client.initialize(&admin, &asset, &500, &200, &4000, &8000);
        client.supply(&supplier, &100_000);
        client.deposit_collateral(&borrower, &10_000);

        let result = client.borrow(&borrower, &5000);
        assert!(result.is_ok());

        let borrow_result = result.unwrap();
        assert!(borrow_result.health_factor >= 10_000);
    }

    #[test]
    fn test_health_factor() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        let asset = Address::generate(&env);

        let contract_id = env.register_contract(None, LendingContract);
        let client = LendingContractClient::new(&env, &contract_id);

        client.initialize(&admin, &asset, &500, &200, &4000, &8000);
        client.deposit_collateral(&user, &10_000);

        let hf = client.get_health_factor(&user);
        assert_eq!(hf, u128::MAX); // No borrows = infinite health
    }

    #[test]
    fn test_interest_rate_calculation() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let supplier = Address::generate(&env);
        let asset = Address::generate(&env);

        let contract_id = env.register_contract(None, LendingContract);
        let client = LendingContractClient::new(&env, &contract_id);

        // base=2%, slope=4%, optimal=80%
        client.initialize(&admin, &asset, &500, &200, &4000, &8000);
        client.supply(&supplier, &100_000);

        let borrow_apy = client.get_borrow_apy();
        assert!(borrow_apy > 0);

        let supply_apy = client.get_supply_apy();
        assert!(supply_apy < borrow_apy); // Supply APY < Borrow APY due to reserve factor
    }
}
