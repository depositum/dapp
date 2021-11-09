use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::Vector;
use near_sdk::json_types::U128;
use near_sdk::json_types::U64;
use near_sdk::{Balance, log, Promise};
use near_sdk::near_bindgen;
use near_sdk::require;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::AccountId;
use near_sdk::BorshStorageKey;
use near_sdk::PanicOnDefault;
use near_sdk::{env, PromiseOrValue};
use near_sdk::{ext_contract, Gas, PromiseResult};
use uint::construct_uint;

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

const RESERVE_TGAS: Gas = Gas(15_000_000_000_000);
const DEPOSIT_CALL_GAS: Gas = Gas(35_000_000_000_000);
const THIRTY_TGAS: Gas = Gas(30_000_000_000_000);
const SWAP_TGAS: Gas = Gas(10_000_000_000_000);
const MFT_TRANSFER_AND_CALL_TGAS: Gas = Gas(60_000_000_000_000);
const TWENTY_TGAS: Gas = Gas(20_000_000_000_000);
const TEN_TGAS: Gas = Gas(10_000_000_000_000);
const GET_DATA_TGAS: Gas = Gas(3_000_000_000_000);
const FEE_DIVISOR: u32 = 10_000;
// const STORAGE_DEPOSIT_NEARS: u128 = 1_250_000_000_000_000_000_000;
const STORAGE_DEPOSIT_NEARS: u128 = 250_000_000_000_000_000_000_000;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct PoolInfo {
    /// Pool kind.
    pub pool_kind: String,
    /// List of tokens in the pool.
    pub token_account_ids: Vec<AccountId>,
    /// How much NEAR this contract has.
    pub amounts: Vec<U128>,
    /// Fee charged for swap.
    pub total_fee: u32,
    /// Total number of shares.
    pub shares_total_supply: U128,
}

// TODO move to better place
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapAction {
    /// Pool which should be used for swapping.
    pub pool_id: u64,
    /// Token to swap from.
    pub token_in: AccountId,
    /// Amount to exchange.
    /// If amount_in is None, it will take amount_out from previous step.
    /// Will fail if amount_in is None on the first step.
    pub amount_in: U128,
    /// Token to swap into.
    pub token_out: AccountId,
    /// Required minimum amount of token_out.
    pub min_amount_out: U128,
}
// TODO move to better place
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageBalance {
    pub total: U128,
    pub available: U128,
}

#[ext_contract(ext_self)]
pub trait RefFinancePostActions {
    fn callback_storage_balance_of(&mut self);
    fn callback_storage_deposit(&mut self);
    fn callback_swap(&mut self, strategy: Strategy, amount_in: U128);
    fn callback_get_pool(&mut self, strategy: Strategy);
    fn callback_ft_transfer_call(&mut self, strategy: Strategy);
    fn callback_add_liquidity(&mut self, strategy: Strategy);
    fn callback_mft_transfer_call(&mut self, strategy: Strategy);
    fn callback_liquidity_shares_balance(&mut self, strategy: Strategy);
}

#[ext_contract(ft_token)]
pub trait FtToken {
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;
}

#[ext_contract(ref_exchange)]
pub trait ExtRefExchane {
    fn get_pool(&self, pool_id: u64) -> PoolInfo;
    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance>;
    fn storage_deposit(
        &mut self,
        account_id: AccountId,
        registration_only: Option<bool>,
    ) -> StorageBalance;
    fn swap(&mut self, actions: Vec<SwapAction>, referral_id: Option<AccountId>) -> U128;
    fn add_liquidity(&mut self, sender_id: AccountId, amounts: Vec<U128>, pool_id: u64) -> Balance;
    fn mft_transfer_call(
        &mut self,
        token_id: String,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128>;
    fn mft_balance_of(&self, token_id: String, account_id: AccountId) -> U128;
}

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    Strategies,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Strategy {
    amount: U128,
    token: AccountId,
}

impl Strategy {
    pub fn new(token: AccountId, amount: U128) -> Self {
        Self { token, amount }
    }
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct RefFarmingStrategy {
    executor: AccountId,
    ref_farming_account: AccountId,
    ref_exchange_account: AccountId,
    strategies: Vector<Strategy>,
    is_initialized: bool,
}

#[near_bindgen]
impl RefFarmingStrategy {
    #[init]
    pub fn new(
        executor: AccountId,
        ref_farming_account: AccountId,
        ref_exchange_account: AccountId,
    ) -> Self {
        Self {
            executor,
            ref_farming_account,
            ref_exchange_account,
            strategies: Vector::new(StorageKey::Strategies),
            is_initialized: false,
        }
    }

    pub fn delete(&self)  -> Promise {
        require!(self.executor == env::predecessor_account_id(), "Need permission");
        Promise::new(env::current_account_id()).delete_account(env::predecessor_account_id())
    }

    /*
     1. добавить проверку токен, убедиться что мы поддерживаем и знаем как работать с этим токеном
    */
    pub fn supply(&mut self, token: AccountId, amount: U128) -> U64 {
        log!("create strategy for token: {}", token);
        log!("step 0, prepaid_gas {:?}", env::prepaid_gas());

        require!(self.is_initialized, "Contract not initialized");
        require!(amount.0 > 0, "Empty amount");
        let strategy = Strategy { token, amount };
        let strategy_id = self.internal_add_strategy(strategy.clone());
        self.farm(strategy);
        strategy_id
    }

    pub fn init(&mut self) {
        // todo check contract owner

        ref_exchange::storage_balance_of(
            env::current_account_id(),
            self.ref_exchange_account.clone(),
            0,
            THIRTY_TGAS, // todo do we need to use gas for the view method?
        )
        .then(ext_self::callback_storage_balance_of(
            env::current_account_id(),
            0,
            THIRTY_TGAS + THIRTY_TGAS + THIRTY_TGAS + THIRTY_TGAS, // todo replace with a proper const
        ));
    }

    fn farm(&self, strategy: Strategy) {
        /*
        1. Deposit to ref exahcnge
        2. Swap 50/50
        3. add liquidity
        4. stake: move tokens to farming contract
        */
        log!("Call farm from strategy");

        let gas_for_next_callback =
            env::prepaid_gas() - env::used_gas() - DEPOSIT_CALL_GAS - RESERVE_TGAS;
        log!("step 1, gas_for_next_callback: {:?}", gas_for_next_callback);
        log!("step 1, used_gas {:?}", env::used_gas());
        log!("step 1, prepaid_gas {:?}", env::prepaid_gas());

        // deposit tokens to ref finance
        ft_token::ft_transfer_call(
            self.ref_exchange_account.clone(),
            strategy.amount.clone(),
            "".to_string(),
            strategy.token.clone(),
            1, // todo create constant
            DEPOSIT_CALL_GAS,
        )
        .then(ext_self::callback_ft_transfer_call(
            strategy,
            env::current_account_id(),
            0,
            gas_for_next_callback,
        ));
    }

    /*

    We can send reward tokens that use claimed to farming

    1. Claim reward
    2. Withdraw reward to strategy contract
    3. call farm function

    */
    pub fn tick() {
        // TODO implement me, please
    }

    /*
    this method should
    1. claim reward
    2. unstake
    3. remove liquidity
    4. swap usdc -> wnear
    5. send wnear to depositium contract
    */
    pub fn redeem() {
        //
        // TODO implement me, please
    }

    #[private]
    pub fn callback_ft_transfer_call(&mut self, strategy: Strategy) {
        log!("callback_ft_transfer_call");

        let is_success = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_value) => true,
            PromiseResult::Failed => false,
        };

        if is_success {
            // todo init in constructor
            let pool_id = 2;

            let gas_for_next_callback =
                env::prepaid_gas() - GET_DATA_TGAS - env::used_gas() - RESERVE_TGAS;

            log!("step 2, gas_for_next_callback: {:?}", gas_for_next_callback);
            log!("step 2, used_gas {:?}", env::used_gas());
            log!("step 2, prepaid_gas {:?}", env::prepaid_gas());

            ref_exchange::get_pool(pool_id, self.ref_exchange_account.clone(), 0, GET_DATA_TGAS)
                .then(ext_self::callback_get_pool(
                    strategy,
                    env::current_account_id(),
                    0,
                    gas_for_next_callback, // todo replace with a proper const
                ));
        } else {
            log!("ft_transfer_call not successfull");
        }
    }

    // todo, calc with decimals
    #[private]
    pub fn calc_swap_amount_out(
        &mut self,
        amount_in: U128,
        pool_info: PoolInfo,
        slippage: u32,
    ) -> U128 {
        let amount_in: u128 = amount_in.into();
        let in_balance = U256::from(pool_info.amounts.get(0).unwrap().0);
        let out_balance = U256::from(pool_info.amounts.get(1).unwrap().0);

        let amount_with_fee =
            U256::from(amount_in) * U256::from(FEE_DIVISOR - pool_info.total_fee + slippage); // todo calculate slippage in othe way
        U128(
            (amount_with_fee * out_balance
                / (U256::from(FEE_DIVISOR) * in_balance + amount_with_fee))
                .as_u128(),
        )
    }

    #[private]
    pub fn callback_get_pool(&mut self, strategy: Strategy) {
        let pool_info: Option<PoolInfo> = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(value) => {
                if let Ok(pool_info) = near_sdk::serde_json::from_slice::<PoolInfo>(&value) {
                    Some(pool_info)
                } else {
                    None
                }
            }
            PromiseResult::Failed => None,
        };
        match pool_info {
            Some(pool_info) => {
                // todo init in constructor
                let pool_id = 2;

                let first_token_amount = pool_info.amounts.get(0).unwrap();
                let second_token_amount = pool_info.amounts.get(1).unwrap();

                let total_fee = pool_info.total_fee;
                log!(
                    "received pool info {:?} {:?} {}",
                    first_token_amount,
                    second_token_amount,
                    total_fee
                );

                let amount_in = strategy.amount.0 / 2;
                let min_amount_out: u128 = self
                    .calc_swap_amount_out(U128(amount_in), pool_info, 10)
                    .into();
                log!("calculated min amount out {}", min_amount_out);
                log!(
                    "calculated amount in: {}, min amount out: {}",
                    amount_in,
                    min_amount_out
                );

                // todo check balance
                let swap_action = SwapAction {
                    pool_id,
                    token_in: AccountId::new_unchecked("wrap_near-aromankov.testnet".to_string()),
                    amount_in: U128(amount_in),
                    token_out: AccountId::new_unchecked("usdc-aromankov.testnet".to_string()),
                    min_amount_out: U128(1),
                };

                // SWAP_TGAS changed from thirty gas
                let gas_for_next_callback =
                    env::prepaid_gas() - env::used_gas() - SWAP_TGAS - RESERVE_TGAS;

                log!("step 3, gas_for_next_callback: {:?}", gas_for_next_callback);
                log!("step 3, used_gas {:?}", env::used_gas());
                log!("step 3, prepaid_gas {:?}", env::prepaid_gas());

                ref_exchange::swap(
                    vec![swap_action],
                    None,
                    self.ref_exchange_account.clone(),
                    STORAGE_DEPOSIT_NEARS,
                    SWAP_TGAS,
                )
                .then(ext_self::callback_swap(
                    strategy,
                    U128(amount_in),
                    env::current_account_id(),
                    0,
                    gas_for_next_callback, // todo replace with a proper const
                ));
            }
            _ => {
                log!("Pool not found");
            }
        }
    }

    #[private]
    pub fn callback_storage_balance_of(&mut self) {
        let zero_balance = StorageBalance {
            total: U128::from(0),
            available: U128::from(0),
        };
        let storage_balance: StorageBalance = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(value) => {
                if let Ok(storage_balance) =
                    near_sdk::serde_json::from_slice::<StorageBalance>(&value)
                {
                    storage_balance
                } else {
                    zero_balance
                }
            }
            PromiseResult::Failed => zero_balance,
        };

        if storage_balance.total.eq(&U128(0)) {
            log!("storage balance is zero, call storage_deposit");

            ref_exchange::storage_deposit(
                env::current_account_id(),
                None,
                self.ref_exchange_account.clone(),
                STORAGE_DEPOSIT_NEARS,
                THIRTY_TGAS,
            )
            .then(ext_self::callback_storage_deposit(
                env::current_account_id(),
                0,
                THIRTY_TGAS, // todo replace with a proper const
            ));
        } else {
            log!(
                "storage balance is {:?}, set is_initialized = true",
                &storage_balance.total
            );
            self.is_initialized = true;
        }
    }

    #[private]
    pub fn callback_storage_deposit(&mut self) {
        let zero_balance = StorageBalance {
            total: U128::from(0),
            available: U128::from(0),
        };
        let storage_balance: StorageBalance = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(value) => {
                if let Ok(storage_balance) =
                    near_sdk::serde_json::from_slice::<StorageBalance>(&value)
                {
                    storage_balance
                } else {
                    zero_balance
                }
            }
            PromiseResult::Failed => zero_balance,
        };

        log!("storage_balance {:?}", storage_balance);

        if storage_balance.total.ne(&U128(0)) {
            self.is_initialized = true;
        }
    }

    #[private]
    pub fn callback_swap(&mut self, strategy: Strategy, amount_in: U128) {
        log!("callback_swap {:?}", strategy.amount);

        let gas_for_next_callback =
            env::prepaid_gas() - env::used_gas() - TWENTY_TGAS - RESERVE_TGAS;

        log!("step 4, gas_for_next_callback: {:?}", gas_for_next_callback);
        log!("step 4, used_gas {:?}", env::used_gas());
        log!("step 4, prepaid_gas {:?}", env::prepaid_gas());

        let swapped_amount: U128 = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(value) => {
                if let Ok(swaped_amount) = near_sdk::serde_json::from_slice::<U128>(&value) {
                    swaped_amount
                } else {
                    U128(0)
                }
            }
            PromiseResult::Failed => U128(0),
        };

        log!("swap amount out {:?}", swapped_amount);
        require!(swapped_amount.0 > 0, "Not succefull swap");
        let pool_id = 2;  // todo get proper pool id
        ref_exchange::add_liquidity(
            env::current_account_id(),
            vec![amount_in, swapped_amount],
            pool_id,
            self.ref_exchange_account.clone(),
            STORAGE_DEPOSIT_NEARS,
            TWENTY_TGAS,
        )
        .then(ext_self::callback_add_liquidity(
            strategy,
            env::current_account_id(),
            0,
            gas_for_next_callback, // todo replace with a proper const
        ));
    }

    #[private]
    pub fn callback_add_liquidity(&mut self, strategy: Strategy) {
        log!("callback_add_liquidity {:?}", strategy.amount);

        log!("step 5, used_gas {:?}", env::used_gas());
        log!("step 5, prepaid_gas {:?}", env::prepaid_gas());
        let gas_for_next_callback =
            env::prepaid_gas() - env::used_gas() - GET_DATA_TGAS - RESERVE_TGAS;
        log!("step 5, gas_for_next_callback: {:?}", gas_for_next_callback);

        let pool_id = ":2".to_string();  // token id

        ref_exchange::mft_balance_of(
            pool_id,
            env::current_account_id(),
            self.ref_exchange_account.clone(),
            1,
            GET_DATA_TGAS,
        ).then(ext_self::callback_liquidity_shares_balance(
            strategy,
            env::current_account_id(),
            0,
            gas_for_next_callback,
        ));
    }

    #[private]
    pub fn callback_liquidity_shares_balance(&mut self, strategy: Strategy) {

        log!("callback_add_liquidity {:?}", strategy.amount);

        log!("step 6, used_gas {:?}", env::used_gas());
        log!("step 6, prepaid_gas {:?}", env::prepaid_gas());
        let gas_for_next_callback =
            env::prepaid_gas() - env::used_gas() - MFT_TRANSFER_AND_CALL_TGAS - RESERVE_TGAS;
        log!("step 6, gas_for_next_callback: {:?}", gas_for_next_callback);

        let pool_id = ":2".to_string();  // token id

        let liquidity_shares: U128 = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(value) => {
                if let Ok(shares) = near_sdk::serde_json::from_slice::<U128>(&value) {
                    shares
                } else {
                    U128(0)
                }
            }
            PromiseResult::Failed => U128(0),
        };
        log!("received liquidity shares: {:?}", liquidity_shares);

        ref_exchange::mft_transfer_call(
            pool_id,
            self.ref_farming_account.clone(), // receiver id
            liquidity_shares,
            None,                             // memo
            "".to_string(),                   // msg
            self.ref_exchange_account.clone(),
            1,
            MFT_TRANSFER_AND_CALL_TGAS,
        )
        .then(ext_self::callback_mft_transfer_call(
            strategy,
            env::current_account_id(),
            0,
            gas_for_next_callback, // todo replace with a proper const
        ));
    }

    #[private]
    pub fn callback_mft_transfer_call(&mut self, strategy: Strategy) {
        log!("callback_mft_transfer_call {:?}", strategy.amount);

        log!("step 7, gas left: {:?}", env::prepaid_gas());
    }

    pub fn accounts_list(&self) -> Vec<AccountId> {
        vec![
            self.ref_farming_account.clone(),
            self.ref_exchange_account.clone(),
        ]
    }

    pub fn get_strategy(&self, id: U64) -> Strategy {
        self.strategies.get(id.0).expect("ERR_NO_STRATEGY")
    }

    fn internal_add_strategy(&mut self, strategy: Strategy) -> U64 {
        let id = self.strategies.len();
        self.strategies.push(&strategy);
        U64::from(id)
    }

    #[init(ignore_state)]
    pub fn migrate() -> Self {
        #[allow(dead_code)]
        #[derive(BorshDeserialize)]
        struct RefFarmingStrategyOld {
            executor: AccountId,
            ref_exchange_account: AccountId,
            ref_finance_account: AccountId,
            strategies: Vector<Strategy>,
            is_initialized: bool,
        }
        let current: RefFarmingStrategyOld =
            env::state_read().expect("Migrate: State doesn't exist");

        let mut next = RefFarmingStrategy::new(
            current.executor,
            AccountId::new_unchecked("ref-farming-rant.testnet".to_string()),
            current.ref_exchange_account,
        );

        next.strategies = current.strategies;

        next
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for RefFarmingStrategy {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        log!("ft_on_transfer {} {} {}", sender_id, amount.0, msg);
        PromiseOrValue::Value(U128(0))
    }
}

/*
#constructor
определяем контракт ref finance и контракт ref farming

#create strategy
тут мы получаем баланс от depositium,
соответсвенно создаем индекс стратегии,
по индексу добавиляем запись о балансе

#run strategy
ох ...


#get balance/state

#stop strategy
*/
