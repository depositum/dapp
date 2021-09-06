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

near_contract_standards::impl_fungible_token_core!(SimpleToken, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(SimpleToken, token, on_account_closed);

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
