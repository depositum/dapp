#!/usr/bin/env bash

set -o errexit

NEAR_ENV=${NEAR_DEV:-testnet}
ROOT_PATH=$(cd "$( dirname "${BASH_SOURCE[0]}" )" && cd ../ && pwd)

simple_token_deploy() {
  symbol="${1:-SIMPLE}"
  decimals="${2:-24}"
  path="local/$NEAR_ENV/$symbol"
  mkdir -p "$path"
  cd "$path"
  CONTRACT="$(cat neardev/dev-account || true)"
  near delete "$CONTRACT" "$NEAR_DEV_ACCOUNT" || true
  rm -fr neardev
  near dev-deploy "$ROOT_PATH/build/test_token.wasm"

  CONTRACT="$(cat neardev/dev-account)"
  near --account_id "$CONTRACT" call "$CONTRACT" new "$(printf '{"symbol": "%s", "decimals": %s}' "$symbol" "$decimals")"
  near state "$NEAR_DEV_ACCOUNT" | grep formattedAmount | cut -d"'" -f2
  near --account_id "$NEAR_DEV_ACCOUNT" call "$CONTRACT" ft_mint --amount 10.00125
  near state "$NEAR_DEV_ACCOUNT" | grep formattedAmount | cut -d"'" -f2
  near view "$CONTRACT" storage_balance_of "$(printf '{"account_id": "%s"}' "$NEAR_DEV_ACCOUNT")"
  near view "$CONTRACT" ft_balance_of "$(printf '{"account_id": "%s"}' "$NEAR_DEV_ACCOUNT")"
  cd "$ROOT_PATH"
}

make rebuild
simple_token_deploy "USD" 23
simple_token_deploy "DAI" 23
