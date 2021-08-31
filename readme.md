<div align="center">
  <h1><code>Novum Depositum at MetaBUIDL 2021</code></h1>
  <p>
    <strong>WiP TBD</strong>
  </p>
</div>

## Local environment

```shell
yarn install
yarn setup
source util/near-shortcut.sh
near-local send alice.local bob.local 1.42
NEAR_ENV=local npx near --keyPath <(local-key bob.local) call  ref-finance.local storage_deposit '{"account_id": "bob.local", "registration_only": false}' --accountId bob.local --amount 0.1
NEAR_ENV=local npx near --keyPath <(local-key bob.local) call  ref-finance.local storage_deposit '{"account_id": "alice.local", "registration_only": false}' --accountId bob.local --amount 0.1


NEAR_ENV=local npx near --keyPath <(local-key bob.local) call usdt.local ft_transfer_call "{\"receiver_id\": \"ref-finance.local\", \"amount\": \"1000000000000\", \"msg\": \"\"}" --accountId bob.local --amount 0.000000000000000000000001
```

## Develop

```shell
make fix 
make qa
make build
make clean
```

### Run CI local

Installation [act](https://github.com/nektos/act):
```shell
brew install act
```

Setup env vars:
```shell
echo "GITHUB_TOKEN=%GITHUB_TOKEN%" | tee .secrets
```

Run
```shell
act --help
```

## Deploy test

```shell
make build
near dev-deploy
contractName=$(cat neardev/dev-account)
near state $contractName
```

## Usage

```shell
accountId=ilyar.testnet
contractName=$(cat neardev/dev-account)
near view $contractName get_num
near call $contractName increment --accountId $accountId
near view $contractName get_num
near call $contractName decrement --accountId $accountId
near view $contractName get_num
near delete $contractName $accountId
```
