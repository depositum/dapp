import * as near from '@4ire-labs/near-sdk'
import * as util from '.'
util.config()

async function main() {
    console.log('[SETUP] IN PROGRESS')
    let code = await near.fetchContract('near', 'mainnet')
    let sender = near.parseAccountNetwork('node0')
    let account = near.custodianAccount('local')
    let result = await near.deployContract(account, code, {amount: '21000000', sender})
    console.log('[SETUP] Created helper account:', {
        accountId: result.account.accountId,
        transactionId: result.outcome.transactionId,
    })
    sender = near.parseAccountNetwork('local')

    account = near.custodianAccount(near.accountIdBySlug('alice'), sender)
    let trx = await near.createAccount(sender, account, '100')
    console.log('[SETUP] Created account:', {
        accountId: account.accountId,
        transactionId: trx.transactionId,
    })
    account = near.custodianAccount(near.accountIdBySlug('bob'), sender)
    trx = await near.createAccount(sender, account, '100')
    console.log('[SETUP] Created account:', {
        accountId: account.accountId,
        transactionId: trx.transactionId,
    })
    console.log('[SETUP] DONE')
}

main().catch(console.error)
