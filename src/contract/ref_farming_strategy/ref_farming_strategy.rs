use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::Vector;
use near_sdk::{Gas, PromiseResult, ext_contract};
use near_sdk::json_types::ValidAccountId;
use near_sdk::json_types::U128;
use near_sdk::json_types::U64;
use near_sdk::log;
use near_sdk::near_bindgen;
use near_sdk::require;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::AccountId;
use near_sdk::env;
use near_sdk::BorshStorageKey;
use near_sdk::PanicOnDefault;

#[cfg(all(test, not(target_arch = "wasm32")))]
mod sim_tests;

const STORAGE_DEPOSIT_CALL_GAS: Gas = Gas(7000000000000);
const STORAGE_DEPOSIT_NEARS: u128 = 18000000000000;

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
    pub amount_in: Option<U128>,
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
    fn callback_storage_deposit(
        &mut self,
        strategy: Strategy,
    );
}

#[ext_contract(ref_exchange)]
pub trait ExtRefExchane {
    fn storage_deposit(
        &mut self,
        account_id: ValidAccountId,
        registration_only: Option<bool>,
    ) -> StorageBalance;
    fn swap(&mut self, action: Vec<SwapAction>, referral_id: Option<ValidAccountId>) -> U128;
    fn add_liquidity(&mut self, sender_id: AccountId, amounts: Vec<Balance>) -> Balance;
    fn mft_transfer_call(
        &mut self,
        token_id: String,
        receiver_id: ValidAccountId,
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
    ref_exchange_account: AccountId,
    ref_finance_account: AccountId,
    strategies: Vector<Strategy>,
}

#[near_bindgen]
impl RefFarmingStrategy {
    #[init]
    pub fn new(executor: AccountId, ref_exchange_account: AccountId, ref_finance_account: AccountId) -> Self {
        Self { 
            executor,
            ref_exchange_account,
            ref_finance_account,
            strategies: Vector::new(StorageKey::Strategies),
        }
    }

    /*
     1. добавить проверку токен, убедиться что мы поддерживаем и знаем как работать с этим токеном
    */
    pub fn create(&mut self, token: AccountId, amount: U128) -> U64 {
        log!("create strategy for token: {}", token);
        require!(amount.0 > 0, "Empty amount");
        let strategy = Strategy { token, amount };
        let strategy_id = self.internal_add_strategy(strategy.clone());
        self.farm(strategy.clone());
        strategy_id
        
    }

    fn farm(&self, strategy: Strategy) {
        /*
        1. Deposit to ref exahcnge
        2. Swap 50/50
        3. add liquidity
        4. stake: move tokens to farming contract
        */
        log!("Call farm from strategy");

        ref_exchange::storage_deposit(
            env::current_account_id(),
            None,
            self.ref_exchange_account.clone(),
            STORAGE_DEPOSIT_NEARS,
            STORAGE_DEPOSIT_CALL_GAS,
        ).then(ext_self::callback_storage_deposit(
            strategy,
            env::current_account_id(),
            0,
            STORAGE_DEPOSIT_CALL_GAS, // todo replace with a proper const
        ));
    }


    #[private]
    pub fn callback_storage_deposit(
        &mut self,
        strategy: Strategy
    ) {
        log!("Call callback_storage_deposit: {:?}", strategy.amount);

        let storage_balance: StorageBalance = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(value) => {
                if let Ok(storage_balance) = near_sdk::serde_json::from_slice::<StorageBalance>(&value) {
                    storage_balance
                } else {
                    StorageBalance { // todo create better default value 
                        total: U128::from(0),
                        available: U128::from(0)
                    }
                }
            }
            PromiseResult::Failed => StorageBalance { // todo create better default value 
                total: U128::from(0),
                available: U128::from(0)
            }
        };

        log!("storage_balance {:?}", storage_balance);
    }

    pub fn accounts_list(&self) -> Vec<AccountId> {
        vec![
            self.ref_exchange_account.clone(),
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
