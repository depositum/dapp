SHELL=bash
YOCTO=0.000000000000000000000001
-include neardev/dev-account.env
-include .env
in_docker_%:
	bash src/contract/build_in_docker.sh make $*

clean_build:
	rm -fr build

clean: clean_build
	cargo clean

lint:
	cargo fmt --all -- --check
	cargo clippy --all-targets

fmt:
	cargo fmt

audit_fix:
	cargo audit fix

audit:
	cargo audit

test_contract_integration: build
	cargo test --lib simulator

test_contract_unit:
	cargo test --lib unit

test_contract:\
test_contract_integration \
test_contract_unit

test:\
test_contract

qa:\
lint \
test

fix:\
audit-fix\
fmt

rustup:
	rustup component add clippy
	rustup component add rustfmt
	rustup component add rust-src
	rustup target add wasm32-unknown-unknown
	cargo install cargo-audit --features=fix

check:
	cargo check
build:
	bash src/contract/build.sh
rebuild: clean_build build
near_deploy_delete:
	near delete ${CONTRACT_NAME} ${NEAR_DEV_ACCOUNT} || exit 0
	rm -fr neardev
near_deploy_new: near_deploy_delete near_deploy
	NEAR_ENV=${NEAR_ENV} near --accountId ${NEAR_DEV_ACCOUNT} call ${CONTRACT_NAME} new
near_deploy:
	NEAR_ENV=${NEAR_ENV} near --masterAccount ${NEAR_DEV_ACCOUNT} dev-deploy build/depositum.wasm
near_balance_of:
	NEAR_ENV=${NEAR_ENV} near --accountId ${NEAR_DEV_ACCOUNT} view ${CONTRACT_NAME} balance_of "{\"account_id\": \"${NEAR_DEV_ACCOUNT}\"}"
local_deposit_usd:
	NEAR_ENV=${NEAR_ENV} near --accountId ${NEAR_DEV_ACCOUNT} call ${CONTRACT_NAME} deposit '{"coin":"usd", "amount":"1"}' --amount ${YOCTO}
#	near --accountId $NEAR_DEV_ACCOUNT call ${CONTRACT_NAME} deposit '{"coin":"dev-1630760424867-24569744671419", "amount":"1"}' --amount ${YOCTO}
#    near --accountId "$CONTRACT_NAME" call dev-1630760424867-24569744671419 ft_mint --amount 10.00125
#    near --accountId "$NEAR_DEV_ACCOUNT" call dev-1630760424867-24569744671419 ft_mint --amount 10.00125
#    near view dev-1630760424867-24569744671419 ft_balance_of "$(printf '{"account_id": "%s"}' ${CONTRACT_NAME})"
#    near view dev-1630760424867-24569744671419 ft_balance_of "$(printf '{"account_id": "%s"}' $NEAR_DEV_ACCOUNT)"
