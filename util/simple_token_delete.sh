#!/usr/bin/env bash

set -o errexit

NEAR_ENV=${NEAR_DEV:-testnet}
ROOT_PATH=$(cd "$( dirname "${BASH_SOURCE[0]}" )" && cd ../ && pwd)
YOCTO=0.000000000000000000000001

simple_token_delete() {
  symbol="${1:-SIMPLE}"
  path="local/$NEAR_ENV/$symbol"
  cd "$path"
  CONTRACT="$(cat neardev/dev-account)"
  near state "$NEAR_DEV_ACCOUNT" | grep formattedAmount | cut -d"'" -f2
  near --account_id "$NEAR_DEV_ACCOUNT" call "$CONTRACT" storage_unregister '{"force": true}' --amount $YOCTO
  near state "$NEAR_DEV_ACCOUNT" | grep formattedAmount | cut -d"'" -f2
  near view "$CONTRACT" storage_balance_of "$(printf '{"account_id": "%s"}' "$NEAR_DEV_ACCOUNT")"
  near view "$CONTRACT" ft_balance_of "$(printf '{"account_id": "%s"}' "$NEAR_DEV_ACCOUNT")"
  near state "$NEAR_DEV_ACCOUNT" | grep formattedAmount | cut -d"'" -f2
  near delete "$CONTRACT" "$NEAR_DEV_ACCOUNT"
  cd "$ROOT_PATH"
  rm -fr "$path"
}

simple_token_delete "USD" 23
simple_token_delete "DAI" 23
