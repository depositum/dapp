import * as near from '@4ire-labs/near-sdk'
import * as util from '.'

util.config('testnet');

const ACCOUNT: string | undefined = process.env.MAIN_ACCOUNT || '';

const ALICE_ACC_NAME = near.accountIdBySlug(`alice-${ACCOUNT}`);
const BOB_ACC_NAME = near.accountIdBySlug(`bob-${ACCOUNT}`);
const REF_FINANCE_ACC_NAME = near.accountIdBySlug(`ref-finance-${ACCOUNT}`);
const USD_TOKEN_ACC_NAME = near.accountIdBySlug(`usdc-${ACCOUNT}`);
const WNEAR_TOKEN_ACC_NAME = near.accountIdBySlug(`wnear-${ACCOUNT}`);

const sender = near.parseAccountNetwork(near.accountIdBySlug(ACCOUNT));

async function prepareAccounts() {
    console.log('[SETUP] PREPARE ACCOUNTS')

    const aliceAcc = near.custodianAccount(ALICE_ACC_NAME, sender)
    const bobAcc = near.custodianAccount(BOB_ACC_NAME, sender)
    const refFinanceAcc = near.custodianAccount(REF_FINANCE_ACC_NAME, sender)
    const usdcAcc = near.custodianAccount(USD_TOKEN_ACC_NAME, sender)
    const wnearAcc = near.custodianAccount(WNEAR_TOKEN_ACC_NAME, sender)

    if (!(await near.isExistAccount(aliceAcc))) {
        const tx = await near.createAccount(sender, aliceAcc, '10')
        console.log('[SETUP] Created account:', { accountId: aliceAcc.accountId, transactionId: tx.transactionId, });
    }

    if (!(await near.isExistAccount(bobAcc))) {
        const tx = await near.createAccount(sender, bobAcc, '10')
        console.log('[SETUP] Created account:', { accountId: bobAcc.accountId, transactionId: tx.transactionId, });
    }

    if (!(await near.isExistAccount(refFinanceAcc))) {
        const tx = await near.createAccount(sender, refFinanceAcc, '10')
        console.log('[SETUP] Created account:', { accountId: refFinanceAcc.accountId, transactionId: tx.transactionId, });
    }

    if (!(await near.isExistAccount(usdcAcc))) {
        const tx = await near.createAccount(sender, usdcAcc, '10')
        console.log('[SETUP] Created account:', { accountId: usdcAcc.accountId, transactionId: tx.transactionId, });
    }

    if (!(await near.isExistAccount(wnearAcc))) {
        const tx = await near.createAccount(sender, wnearAcc, '10')
        console.log('[SETUP] Created account:', { accountId: wnearAcc.accountId, transactionId: tx.transactionId, });
    }
}

async function createTokens() {
    console.log('[SETUP] CREATE TOKEN CONTRACTS');
    const token = await near.fetchContract('dev-1630516277838-72756524527007', 'testnet');

    // create USDC contract
    try {
        const usdcAcc = near.custodianAccount(USD_TOKEN_ACC_NAME);
        const codeHash = (await (await near.accountConnect(usdcAcc)).state()).code_hash;
        if (codeHash == '11111111111111111111111111111111') {
            const result = await near.deployContract(usdcAcc, token, {
                init: { methodName: 'new', args: {} },
                amount: '10', sender: usdcAcc
            });

            console.log('[SETUP] contract deployed:', { transactionId: result.outcome.transactionId, });
        }
    } catch (err) { }

    // create wNEAR contract
    const wnearAcc = near.custodianAccount(WNEAR_TOKEN_ACC_NAME);
    const codeHash = (await (await near.accountConnect(wnearAcc)).state()).code_hash;
    if (codeHash == '11111111111111111111111111111111') {
        try {
            const result = await near.deployContract(wnearAcc, token, {
                init: { methodName: 'new', args: {} },
                amount: '10', sender: wnearAcc
            });

            console.log('[SETUP] contract deployed:', { transactionId: result.outcome.transactionId, });
        } catch (err) { }
    }

}

async function deleteAccounts(accounts: string[]) {
    for (const accName of accounts) {
        const account = near.custodianAccount(accName);
        await near.deleteAccount(account, sender);
    }
}

async function createRefFinanceContract() {
    console.log('[SETUP] CREATE REF FINANCE CONTRACT');

    const financeAcc = near.custodianAccount(REF_FINANCE_ACC_NAME);
    const codeHash = (await (await near.accountConnect(financeAcc)).state()).code_hash;
    if (codeHash === '11111111111111111111111111111111') {
        const finance = await near.fetchContract('v2.ref-finance.near', 'mainnet');
        try {
            const result = await near.deployContract(financeAcc, finance, {
                init: {
                    methodName: 'new',
                    args: {
                        owner_id: ALICE_ACC_NAME,
                        exchange_fee: 4,
                        referral_fee: 1,
                    }
                },
                amount: '10', sender: financeAcc
            });

            console.log('[SETUP] contract deployed:', {
                transactionId: result.outcome.transactionId,
            });
        } catch (err) { }
    }

}

async function mintTokens() {
    console.log('[SETUP] MINT TOKENS');

    const usdAcc = near.custodianAccount(USD_TOKEN_ACC_NAME);
    const wnearAcc = near.custodianAccount(WNEAR_TOKEN_ACC_NAME);
    const usdToken = await near.Contract.connect(near.Contract, USD_TOKEN_ACC_NAME, usdAcc)

    for (const accId of [ALICE_ACC_NAME, BOB_ACC_NAME, REF_FINANCE_ACC_NAME]) {
        const balance = await usdToken.call<string>({
            methodName: 'ft_balance_of',
            args: { account_id: accId }
        });
        console.log(`${accId} usd balance: ${balance}`);
        if (balance === '0') {
            try {
                const amount = '10000000000000000000';
                const res = await usdToken.callRaw({
                    methodName: 'mint',
                    args: { account_id: accId, amount }
                })
                console.log(`Minted ${amount} USDT for ${accId}, txId: ${res.transactionId}`)
            } catch (err) {
                console.log(`Failed to mint ust token`, err);
            }
        }
    }

    const wnearToken = await near.Contract.connect(near.Contract, WNEAR_TOKEN_ACC_NAME, wnearAcc)

    for (const accId of [ALICE_ACC_NAME, BOB_ACC_NAME, REF_FINANCE_ACC_NAME]) {
        const balance = await wnearToken.call<string>({
            methodName: 'ft_balance_of',
            args: { account_id: accId }
        });
        console.log(`${accId} wnear balance: ${balance}`);
        if (balance === '0') {
            try {
                const amount = '10000000000000000000';
                const res = await wnearToken.callRaw({
                    methodName: 'mint',
                    args: { account_id: accId, amount }
                })
                console.log(`Minted ${amount} USDT for ${accId}, txId: ${res.transactionId}`)
            } catch (err) {
                console.log(`Failed to mint wnear token`, err);
            }

        }
    }

}

async function attachDeposit() {
    for (const accId of [ALICE_ACC_NAME, BOB_ACC_NAME]) {
        const acc = near.custodianAccount(accId);

        const refContract = await near.Contract.connect(near.Contract, REF_FINANCE_ACC_NAME, acc);

        const res = await refContract.call<any>({
            methodName: 'storage_deposit',
            args: { account_id: accId, registration_only: false },
            attachedDeposit: '0.1',
        });
        console.log(`storage_deposit for ${accId}`, res);
    }
}

async function whitelistTokensInRef() {
    const acc = near.custodianAccount(ALICE_ACC_NAME);
    const refContract = await near.Contract.connect(near.Contract, REF_FINANCE_ACC_NAME, acc);

    const res = await refContract.callRaw<any>({
        methodName: 'extend_whitelisted_tokens',
        args: { tokens: [USD_TOKEN_ACC_NAME, WNEAR_TOKEN_ACC_NAME] },
        attachedDeposit: '0.000000000000000000000001',
    });
    console.log(`register_tokens response`, res);
}

async function addPool() {
    const acc = near.custodianAccount(ALICE_ACC_NAME);
    const refContract = await near.Contract.connect(near.Contract, REF_FINANCE_ACC_NAME, acc);

    const res = await refContract.callRaw<any>({
        methodName: 'add_simple_pool',
        args: { tokens: [USD_TOKEN_ACC_NAME, WNEAR_TOKEN_ACC_NAME], fee: 25 },
        attachedDeposit: '0.1',
    });
    console.log(`a add_simple_pool`, res);
}

async function addLiquidityToPool() {
    // near call $CONTRACT_ID add_liquidity '{"pool_id": 0, "amounts": ["10000", "10000"]}' --accountId $USER_ID --amount 0.000000000000000000000001
    const acc = near.custodianAccount(BOB_ACC_NAME);
    const refContract = await near.Contract.connect(near.Contract, REF_FINANCE_ACC_NAME, acc);

    const res = await refContract.callRaw<any>({
        methodName: 'add_liquidity',
        args: {
            pool_id: 0,
            amounts: ['100000', '100000']
        },
        attachedDeposit: '0.1',
    });
    console.log(`add_liquidity response`, res);
}

async function swap(account: string) {
    const acc = near.custodianAccount(account);
    const refContract = await near.Contract.connect(near.Contract, REF_FINANCE_ACC_NAME, acc);

    const res = await refContract.callRaw<any>({
        methodName: 'swap',
        args: {
            actions: [{
                pool_id: 0,
                token_in: WNEAR_TOKEN_ACC_NAME,
                amount_in: '1000',
                token_out: USD_TOKEN_ACC_NAME,
                min_amount_out: '1'
            }],
        },
        attachedDeposit: '0.000000000000000000000001',
    });

    console.log(`swap response`, res);
}

async function withdraw(account: string, token: string, amount: string) {
    const acc = near.custodianAccount(account);
    const refContract = await near.Contract.connect(near.Contract, REF_FINANCE_ACC_NAME, acc);

    const res = await refContract.callRaw<any>({
        methodName: 'withdraw',
        args: {
            token_id: token,
            amount,
        },
        attachedDeposit: '0.000000000000000000000001',
    });

    console.log(`withdraw response`, res);
}

async function transferTokensToRefContract() {
    for (const tokenAccId of [USD_TOKEN_ACC_NAME, WNEAR_TOKEN_ACC_NAME]) {
        for (const userAccId of [ALICE_ACC_NAME, BOB_ACC_NAME]) {
            const acc = near.custodianAccount(userAccId);
            const tokenContract = await near.Contract.connect(near.Contract, tokenAccId, acc);
            const res = await tokenContract.call<any>({
                methodName: 'ft_transfer_call',
                args: {
                    receiver_id: REF_FINANCE_ACC_NAME,
                    amount: '10000000000',
                    msg: ''
                },
                attachedDeposit: '0.000000000000000000000001',
            });
            console.log(`${tokenAccId} ft_transfer_call for ${userAccId}`, res);
        }
    }
}

async function printPools() {
    const aliceAcc = near.custodianAccount(ALICE_ACC_NAME);
    const refContract = await near.Contract.connect(near.Contract, REF_FINANCE_ACC_NAME, aliceAcc);

    const pools = await refContract.call<any>({
        methodName: 'get_pools',
        args: { from_index: 0, limit: 10 }
    });

    console.log('pools', pools);
}

async function printBlance(account: string) {
    const acc = near.custodianAccount(account);
    const refContract = await near.Contract.connect(near.Contract, REF_FINANCE_ACC_NAME, acc);

    const res = await refContract.call<any>({
        methodName: 'get_deposits',
        args: { account_id: account }
    });

    console.log(`${account} balance: ${JSON.stringify(res, null, 2)}`);
}

async function main() {
    console.log('[SETUP] IN PROGRESS');
    const init = async () => {
        await prepareAccounts();
        await createRefFinanceContract();
        await createTokens();
        await mintTokens();
        await whitelistTokensInRef();
    };

    // await init();

    const accountActions = async () => {
        // 1)
        await attachDeposit();
        // 2)
        await transferTokensToRefContract();
        // 3) add pool
        await addPool();
        // 4) add liquidity
        await addLiquidityToPool();
    };

    // await accountActions();


    await swap(BOB_ACC_NAME);
    // await withdraw(BOB_ACC_NAME, USD_TOKEN_ACC_NAME, '1000');

    await printPools();
    await printBlance(BOB_ACC_NAME);
    await printBlance(ALICE_ACC_NAME);
}

main().catch(console.error)
