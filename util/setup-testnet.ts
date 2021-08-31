import * as near from '@4ire-labs/near-sdk'
import * as util from '.'

util.config('testnet');

const ACCOUNT: string | undefined = process.env.MAIN_ACCOUNT || '';

const ALICE_ACC_NAME = near.accountIdBySlug(`alice-${ACCOUNT}`);
const BOB_ACC_NAME = near.accountIdBySlug(`bob-${ACCOUNT}`);
const REF_FINANCE_ACC_NAME = near.accountIdBySlug(`ref-finance-${ACCOUNT}`);
const sender = near.parseAccountNetwork(near.accountIdBySlug(ACCOUNT));

async function prepareAccounts() {
    console.log('[SETUP] PREPARE ACCOUNTS')

    const aliceAcc = near.custodianAccount(ALICE_ACC_NAME, sender)
    const bobAcc = near.custodianAccount(BOB_ACC_NAME, sender)
    const refFinanceAcc = near.custodianAccount(REF_FINANCE_ACC_NAME, sender)

    if (!(await near.isExistAccount(aliceAcc))) {
        const trx1 = await near.createAccount(sender, aliceAcc, '10')
        console.log('[SETUP] Created account:', {
            accountId: aliceAcc.accountId,
            transactionId: trx1.transactionId,
        });
    }

    if (!(await near.isExistAccount(bobAcc))) {
        const trx2 = await near.createAccount(sender, bobAcc, '10')
        console.log('[SETUP] Created account:', {
            accountId: bobAcc.accountId,
            transactionId: trx2.transactionId,
        });
    }

    if (!(await near.isExistAccount(refFinanceAcc))) {
        const trx2 = await near.createAccount(sender, refFinanceAcc, '10')
        console.log('[SETUP] Created account:', {
            accountId: refFinanceAcc.accountId,
            transactionId: trx2.transactionId,
        });
    }

    console.log(``);
}

async function createRefFinanceContract() {
    console.log('[SETUP] CREATE REF FINANCE CONTRACT');

    const financeContract = near.custodianAccount(REF_FINANCE_ACC_NAME)
    const finance = await near.fetchContract('v2.ref-finance.near', 'mainnet')
    try {
        const result = await near.deployContract(financeContract, finance, {
            init: {
                methodName: 'new',
                args: {
                    owner_id: ALICE_ACC_NAME,
                    exchange_fee: 4,
                    referral_fee: 1,
                }
            },
            amount: '10', sender: financeContract
        });

        console.log('[SETUP] contract deployed:', {
            transactionId: result.outcome.transactionId,
        });
    } catch (err) { }
}

async function main() {
    console.log('[SETUP] IN PROGRESS')

    await prepareAccounts();

    await createRefFinanceContract();

    // await near.deleteAccount(financeContract, sender)

    // console.log('[SETUP] Created helper account:', {
    //     accountId: result.account.accountId,
    //     transactionId: result.outcome.transactionId,
    // })

    // const ftTokenCode = await near.fetchContract('dev-1630432677531-68656683637643', 'testnet')
    // const usdContract = near.custodianAccount(near.accountIdBySlug('usdt'))
    // const usdTokenResult = await near.deployContract(usdContract, ftTokenCode, {
    //     init: {methodName: 'new', args: {}},
    //     amount: '10',
    //     sender
    // })
    // console.log('[SETUP] Created helper account:', {
    //     accountId: usdTokenResult.account.accountId,
    //     transactionId: usdTokenResult.outcome.transactionId
    // })
    // const usdToken = await near.Contract.connect(near.Contract, near.accountIdBySlug('usdt'), sender)
    // await usdToken.callRaw({
    //     methodName: 'mint',
    //     args: {account_id: near.accountIdBySlug('alice'), amount: '10000000000000000000'}
    // })
    // await usdToken.callRaw({
    //     methodName: 'mint',
    //     args: {account_id: near.accountIdBySlug('bob'), amount: '10000000000000000000',}
    // })

    // const wnearContract = near.custodianAccount(near.accountIdBySlug('wnear'))
    // const wnearTokenResult = await near.deployContract(wnearContract, ftTokenCode, {
    //     init: {methodName: 'new', args: {}},
    //     amount: '10',
    //     sender
    // })
    // console.log('[SETUP] Created helper account:', {
    //     accountId: wnearTokenResult.account.accountId,
    //     transactionId: wnearTokenResult.outcome.transactionId
    // })
    // const wnearToken = await near.Contract.connect(near.Contract, near.accountIdBySlug('wnear'), sender)
    // await wnearToken.callRaw({
    //     methodName: 'mint',
    //     args: {account_id: near.accountIdBySlug('alice'), amount: '10000000000000000000'}
    // })
    // await wnearToken.callRaw({
    //     methodName: 'mint',
    //     args: {account_id: near.accountIdBySlug('bob'), amount: '10000000000000000000',}
    // })
}

main().catch(console.error)
