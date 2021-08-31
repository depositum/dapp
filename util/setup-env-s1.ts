import * as near from '@4ire-labs/near-sdk'
import * as util from '.'

util.config()

async function main() {
    console.log('[SETUP] IN PROGRESS')
    const sender = near.parseAccountNetwork('local')
    const financeContract = near.custodianAccount(near.accountIdBySlug('ref-finance'))
    const finance = await near.fetchContract('v2.ref-finance.near', 'mainnet')
    const result = await near.deployContract(financeContract, finance, {
            init: {
                methodName: 'new',
                args: {
                    owner_id: near.accountIdBySlug('alice'),
                    exchange_fee: 4,
                    referral_fee: 1,
                }
            },
            amount: '100', sender
        }
    )
    console.log('[SETUP] Created helper account:', {
        accountId: result.account.accountId,
        transactionId: result.outcome.transactionId,
    })

    const ftTokenCode = await near.fetchContract('dev-1630432677531-68656683637643', 'testnet')
    const usdContract = near.custodianAccount(near.accountIdBySlug('usdt'))
    const usdTokenResult = await near.deployContract(usdContract, ftTokenCode, {
        init: {methodName: 'new', args: {}},
        amount: '10',
        sender
    })
    console.log('[SETUP] Created helper account:', {
        accountId: usdTokenResult.account.accountId,
        transactionId: usdTokenResult.outcome.transactionId
    })
    const usdToken = await near.Contract.connect(near.Contract, near.accountIdBySlug('usdt'), sender)
    await usdToken.callRaw({
        methodName: 'mint',
        args: {account_id: near.accountIdBySlug('alice'), amount: '10000000000000000000'}
    })
    await usdToken.callRaw({
        methodName: 'mint',
        args: {account_id: near.accountIdBySlug('bob'), amount: '10000000000000000000',}
    })

    const wnearContract = near.custodianAccount(near.accountIdBySlug('wnear'))
    const wnearTokenResult = await near.deployContract(wnearContract, ftTokenCode, {
        init: {methodName: 'new', args: {}},
        amount: '10',
        sender
    })
    console.log('[SETUP] Created helper account:', {
        accountId: wnearTokenResult.account.accountId,
        transactionId: wnearTokenResult.outcome.transactionId
    })
    const wnearToken = await near.Contract.connect(near.Contract, near.accountIdBySlug('wnear'), sender)
    await wnearToken.callRaw({
        methodName: 'mint',
        args: {account_id: near.accountIdBySlug('alice'), amount: '10000000000000000000'}
    })
    await wnearToken.callRaw({
        methodName: 'mint',
        args: {account_id: near.accountIdBySlug('bob'), amount: '10000000000000000000',}
    })
}

main().catch(console.error)
