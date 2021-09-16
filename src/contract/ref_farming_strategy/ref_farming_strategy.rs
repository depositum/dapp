use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::Vector;
use near_sdk::json_types::U128;
use near_sdk::json_types::U64;
use near_sdk::log;
use near_sdk::near_bindgen;
use near_sdk::require;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::AccountId;
use near_sdk::BorshStorageKey;
use near_sdk::PanicOnDefault;
use near_sdk::{env, PromiseOrValue};
use near_sdk::{ext_contract, Gas, PromiseResult};

#[cfg(all(test, not(target_arch = "wasm32")))]
mod sim_tests;

const STORAGE_BALANCE_OFF_VIEW_GAS: Gas = Gas(1_000_000_000_000);
const STORAGE_DEPOSIT_CALL_GAS: Gas = Gas(7_000_000_000_000);
const DEPOSIT_CALL_GAS: Gas = Gas(35_000_000_000_000);
const THIRTY_TGAS: Gas = Gas(30_000_000_000_000);
const TWO_HUNDRED_TGAS: Gas = Gas(210_000_000_000_000);
const SIMPLE_CALLBACK_GAS: Gas = Gas(1_000_000_000_000);
const TWENTY_TGAS: Gas = Gas(20_000_000_000_000);
const TEN_TGAS: Gas = Gas(20_000_000_000_000);
// const STORAGE_DEPOSIT_NEARS: u128 = 1250000000000000000000; // initial
const STORAGE_DEPOSIT_NEARS: u128 = 1_250_000_000_000_000_000_000;

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
    fn callback_swap(&mut self, strategy: Strategy);
    fn callback_ft_transfer_call(&mut self, strategy: Strategy);
    fn callback_add_liquidity(&mut self, strategy: Strategy);
    fn callback_mft_transfer_call(&mut self, strategy: Strategy);
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

#[ext_contract(ref_finance)]
pub trait ExtRefExchane {
    fn storage_balance_of(&mut self, account_id: AccountId) -> Option<StorageBalance>;
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
    ref_finance_account: AccountId,
    strategies: Vector<Strategy>,
    is_initialized: bool,
}

#[near_bindgen]
impl RefFarmingStrategy {
    #[init]
    pub fn new(
        executor: AccountId,
        ref_farming_account: AccountId,
        ref_finance_account: AccountId,
    ) -> Self {
        Self {
            executor,
            ref_farming_account,
            ref_finance_account,
            strategies: Vector::new(StorageKey::Strategies),
            is_initialized: false,
        }
    }

    /*
     1. добавить проверку токен, убедиться что мы поддерживаем и знаем как работать с этим токеном
    */
    pub fn create(&mut self, token: AccountId, amount: U128) -> U64 {
        log!("create strategy for token: {}", token);
        require!(self.is_initialized, "Contract not initialized");
        require!(amount.0 > 0, "Empty amount");
        let strategy = Strategy { token, amount };
        let strategy_id = self.internal_add_strategy(strategy.clone());
        self.farm(strategy.clone());
        strategy_id
    }

    pub fn init(&mut self) {
        // todo check contract owner

        ref_finance::storage_balance_of(
            env::current_account_id(),
            self.ref_finance_account.clone(),
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

        // deposit tokens to ref finance
        ft_token::ft_transfer_call(
            self.ref_finance_account.clone(),
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
            TWO_HUNDRED_TGAS, // todo replace with a proper const
        ));
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
            // todo check balance
            let swap_action = SwapAction {
                pool_id: 0,
                token_in: AccountId::new_unchecked("usdc-rant.testnet".to_string()),
                amount_in: U128(10),
                token_out: AccountId::new_unchecked("wnear-rant.testnet".to_string()),
                min_amount_out: U128(1),
            };

            ref_finance::swap(
                vec![swap_action],
                None,
                self.ref_finance_account.clone(),
                STORAGE_DEPOSIT_NEARS,
                TWENTY_TGAS,
            )
            .then(ext_self::callback_swap(
                strategy,
                env::current_account_id(),
                0,
                THIRTY_TGAS + THIRTY_TGAS + THIRTY_TGAS + THIRTY_TGAS + THIRTY_TGAS, // todo replace with a proper const
            ));
        } else {
            log!("ft_transfer_call not successfull");
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

            ref_finance::storage_deposit(
                env::current_account_id(),
                None,
                self.ref_finance_account.clone(),
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
    pub fn callback_swap(&mut self, strategy: Strategy) {
        log!("callback_swap {:?}", strategy.amount);

        ref_finance::add_liquidity(
            env::current_account_id(),
            vec![U128(10), U128(10)], // todo remove hardcode
            0, // todo get proper pool id
            self.ref_finance_account.clone(),
            STORAGE_DEPOSIT_NEARS,
            TWENTY_TGAS,
        )
        .then(ext_self::callback_add_liquidity(
            strategy,
            env::current_account_id(),
            0,
            THIRTY_TGAS + THIRTY_TGAS + THIRTY_TGAS, // todo replace with a proper const
        ));
    }

    #[private]
    pub fn callback_add_liquidity(&mut self, strategy: Strategy) {
        log!("callback_add_liquidity {:?}", strategy.amount);

         ref_finance::mft_transfer_call(
            ":0".to_string(), // token id
            self.ref_farming_account.clone(), // receiver id
            U128(100), // amount, todo remove hardcode
            None, // memo
            "".to_string(), // msg
            self.ref_finance_account.clone(),
            1,
            THIRTY_TGAS,
        )
        .then(ext_self::callback_mft_transfer_call(
            strategy,
            env::current_account_id(),
            0,
            TEN_TGAS, // todo replace with a proper const
        ));
    }

    #[private]
    pub fn callback_mft_transfer_call(&mut self, strategy: Strategy) {
        log!("callback_mft_transfer_call {:?}", strategy.amount);
    }

    pub fn accounts_list(&self) -> Vec<AccountId> {
        vec![
            self.ref_farming_account.clone(),
            self.ref_finance_account.clone(),
        ]
    }

    pub fn get_strategy(self, id: U64) -> Strategy {
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
            current.ref_finance_account,
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
        log!("ft_on_transfer");
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
