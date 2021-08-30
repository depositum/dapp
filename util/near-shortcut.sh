# shellcheck disable=SC2120
local-key() {
  . local.env
  id="${1:-${NEAR_SENDER_ID}}"
  sk="${NEAR_SENDER_PRIVATE_KEY}"
  printf '{"account_id":"%s", "secret_key":"%s"}' "$id" "$sk"
}
near-local() {
  . local.env
  NEAR_ENV=local npx near --keyPath <(local-key) "$@"
}
near-local-call() {
  . local.env
  NEAR_ENV=local npx near --keyPath <(local-key) --accountId "${NEAR_SENDER_ID}" call --gas 300000000000000 "${NEAR_CONTRACT_ID}" "$@"
}
near-local-view() {
  . local.env
  NEAR_ENV=local npx near view "${NEAR_CONTRACT_ID}" "$@"
}
