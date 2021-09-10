use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadataProvider;
use near_contract_standards::fungible_token::metadata::FT_METADATA_SPEC;
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh;
use near_sdk::borsh::BorshDeserialize;
use near_sdk::borsh::BorshSerialize;
use near_sdk::collections::LazyOption;
use near_sdk::env;
use near_sdk::json_types::U128;
use near_sdk::log;
use near_sdk::near_bindgen;
use near_sdk::require;
use near_sdk::AccountId;
use near_sdk::Balance;
use near_sdk::BorshStorageKey;
use near_sdk::PanicOnDefault;
use near_sdk::Promise;
use near_sdk::PromiseOrValue;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct SimpleToken {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    FungibleToken,
    Metadata,
}

#[near_bindgen]
impl SimpleToken {
    #[init]
    pub fn new(symbol: String, decimals: Option<u8>) -> Self {
        require!(!env::state_exists(), "Already initialized");
        Self {
            token: FungibleToken::new(StorageKey::FungibleToken),
            metadata: LazyOption::new(
                StorageKey::Metadata,
                Some(&FungibleTokenMetadata {
                    spec: FT_METADATA_SPEC.to_string(),
                    name: symbol.clone(),
                    symbol: symbol.clone(),
                    icon: Some(format!("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 250 250'%3E%3Ccircle cx='125' cy='125' r='100' fill='%2300'/%3E%3Ctext x='50%25' y='50%25' text-anchor='middle' fill='%23fff' font-size='150' font-family='arial' dy='.35em'%3E{}%3C/text%3E%3C/svg%3E", symbol.chars().next().unwrap_or_default())),
                    reference: None,
                    reference_hash: None,
                    decimals: decimals.unwrap_or(24),
                }),
            ),
        }
    }

    #[payable]
    pub fn ft_mint(&mut self) {
        let holder_id = env::predecessor_account_id();
        let mut supply = env::attached_deposit();
        if !self.token.accounts.contains_key(&holder_id) {
            supply = supply
                .checked_sub(self.token.storage_balance_bounds().min.into())
                .unwrap_or_else(|| env::panic_str("Not enough attached deposit"));
            self.token.internal_register_account(&holder_id);
        }
        self.token.internal_deposit(&holder_id, supply);
    }

    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        if balance > 0 {
            self.internal_burn(&account_id, balance);
        }
        log!("Closed @{}", account_id);
    }

    fn internal_burn(&mut self, account_id: &AccountId, amount: Balance) {
        let account_balance = self.token.accounts.get(&account_id).unwrap_or(0);
        if amount > 0 && amount >= account_balance {
            Promise::new(account_id.clone()).transfer(amount);
            log!("Account @{} burned {}", account_id, amount)
        }
    }

    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        self.internal_burn(&account_id, amount)
    }
}

use near_contract_standards::fungible_token::core::FungibleTokenCore;
use near_contract_standards::fungible_token::resolver::FungibleTokenResolver;

#[near_bindgen]
impl FungibleTokenCore for SimpleToken {
    #[payable]
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        log!("ft_transfer");
        self.token.ft_transfer(receiver_id, amount, memo)
    }

    #[payable]
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        log!("ft_transfer_call");
        self.token.ft_transfer_call(receiver_id, amount, memo, msg)
    }

    fn ft_total_supply(&self) -> U128 {
        log!("ft_total_supply");
        self.token.ft_total_supply()
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        log!("ft_balance_of");
        self.token.ft_balance_of(account_id)
    }
}

#[near_bindgen]
impl FungibleTokenResolver for SimpleToken {
    #[private]
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128 {
        log!("ft_resolve_transfer");
        let (used_amount, burned_amount) =
            self.token
                .internal_ft_resolve_transfer(&sender_id, receiver_id, amount);
        if burned_amount > 0 {
            self.on_tokens_burned(sender_id, burned_amount);
        }
        used_amount.into()
    }
}

use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};

#[near_bindgen]
impl StorageManagement for SimpleToken {
    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        log!("storage_deposit");
        self.token.storage_deposit(account_id, registration_only)
    }

    #[payable]
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        log!("storage_withdraw");
        self.token.storage_withdraw(amount)
    }

    #[payable]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        log!("storage_unregister");
        #[allow(unused_variables)]
        if let Some((account_id, balance)) = self.token.internal_storage_unregister(force) {
            self.on_account_closed(account_id, balance);
            true
        } else {
            false
        }
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        log!("storage_balance_bounds");
        self.token.storage_balance_bounds()
    }

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        log!("storage_balance_of");
        self.token.storage_balance_of(account_id)
    }
}

#[near_bindgen]
impl FungibleTokenMetadataProvider for SimpleToken {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod unit {
    use near_sdk::test_utils::accounts;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;
    use near_sdk::Balance;

    use super::*;

    const YOCTO: Balance = 1_000_000_000_000_000_000_000_000;
    const STORAGE_COVER: Balance = 1_250_000_000_000_000_000_000;
    const SUPPLY: Balance = 100 * YOCTO;
    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        context.attached_deposit(SUPPLY + STORAGE_COVER);
        testing_env!(context.build());
        let mut contract = SimpleToken::new("T".to_string(), None);
        contract.ft_mint();
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.ft_total_supply().0, SUPPLY);
        assert_eq!(contract.ft_balance_of(accounts(1)).0, SUPPLY);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = SimpleToken::default();
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(2));
        context.attached_deposit(SUPPLY + STORAGE_COVER);
        testing_env!(context.build());
        let mut contract = SimpleToken::new("T".to_string(), None);
        contract.ft_mint();
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(2))
            .build());
        let transfer_amount = SUPPLY / 3;
        contract.ft_transfer(accounts(1), transfer_amount.into(), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert_eq!(
            contract.ft_balance_of(accounts(2)).0,
            (SUPPLY - transfer_amount)
        );
        assert_eq!(contract.ft_balance_of(accounts(1)).0, transfer_amount);
    }
}
