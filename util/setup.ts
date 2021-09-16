import * as near from '@4ire-labs/near-sdk'
import * as util from '.'
util.config('local')
const environment = near.environment()
async function main() {
    console.log(`[${environment.networkId}] IN PROGRESS`)
    let code = await near.fetchContract('near', 'mainnet')
    let sender = near.parseAccountNetwork('node0')
    let account = near.custodianAccount('local')
    let result = await near.deployContract(account, code, {amount: '21000000', sender})
    await near.writeUnencryptedFileSystemKeyStore(account)
    console.log(`SETUP [${environment.networkId}] Created helper account:`, {
        accountId: result.account.accountId,
        transactionId: result.outcome.transactionId,
    })
    sender = near.parseAccountNetwork('local')

    account = near.custodianAccount(near.accountIdBySlug('alice'), sender)
    let trx = await near.createAccount(sender, account, '100')
    await near.writeUnencryptedFileSystemKeyStore(account)
    console.log(`SETUP [${environment.networkId}] Created account:`, {
        accountId: account.accountId,
        transactionId: trx.transactionId,
    })
    account = near.custodianAccount(near.accountIdBySlug('bob'), sender)
    trx = await near.createAccount(sender, account, '100')
    await near.writeUnencryptedFileSystemKeyStore(account)
    console.log(`SETUP [${environment.networkId}] Created account:`, {
        accountId: account.accountId,
        transactionId: trx.transactionId,
    })
    account = near.custodianAccount(near.accountIdBySlug('depositum'), sender)
    trx = await near.createAccount(sender, account, '100')
    await near.writeUnencryptedFileSystemKeyStore(account)
    console.log(`SETUP [${environment.networkId}] Created account:`, {
        accountId: account.accountId,
        transactionId: trx.transactionId,
    })
    console.log(`SETUP [${environment.networkId}] DONE`)
}

main().catch(console.error)
