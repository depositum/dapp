import {
  connect,
  keyStores,
  WalletConnection,
} from 'near-api-js'
import {
  config,
} from './config'
import {Depositum} from '../contract/depositum/depositum'
const nearConfig = config('testnet')

// Initialize contract & set global variables
export async function initContract() {
  // Initialize connection to the NEAR
  const near = await connect(Object.assign({
    deps: {
      keyStore: new keyStores.BrowserLocalStorageKeyStore(),
    },
  }, nearConfig))
  // Initializing Wallet based Account
  window.walletConnection = new WalletConnection(near)
  // Getting the Account ID. If still unauthorized, it's just empty string
  window.accountId = window.walletConnection.getAccountId()
  // Initializing our contract APIs by contract name and configuration
  window.contract = new Depositum(window.walletConnection.account(), nearConfig.contractName)
}

export function logout() {
  window.walletConnection.signOut()
  // reload page
  window.location.replace(window.location.origin + window.location.pathname)
}

export function login() {
  window.walletConnection.requestSignIn(nearConfig.contractName)
}
