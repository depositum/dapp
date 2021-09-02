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
	bash src/contract/build_in_docker.sh
rebuild: clean_build build

YOCTO=0.000000000000000000000001

CONTRACT=$(shell cat neardev/dev-account)
local_deploy_delete:
	NEAR_ENV=local near delete ${CONTRACT} local || exit 0
	rm -fr neardev
local_deploy_new: local_deploy_delete local_deploy
	NEAR_ENV=local near --account_id local call ${CONTRACT} new
local_deploy: rebuild
	NEAR_ENV=local near --masterAccount local dev-deploy build/depositum-minified.wasm
local_balance_of:
	NEAR_ENV=local near --account_id local view ${CONTRACT} balance_of '{"account_id": "local"}'
local_deposit_usd:
	NEAR_ENV=local near --account_id local call ${CONTRACT} deposit '{"coin":"usd", "amount":"1"}' --amount ${YOCTO}
