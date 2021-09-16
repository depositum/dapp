import 'regenerator-runtime/runtime'
import {initContract, login, logout,} from './wallet'
import {config,} from './config'
const nearConfig = config('testnet')

document.querySelector('#sign-in-button').onclick = login
document.querySelector('#sign-out-button').onclick = logout

function signedInFlow() {
  document.querySelector('#signed-in-flow').style.display = 'block'
}

function signedOutFlow() {
  document.querySelector('#signed-out-flow').style.display = 'block'
  const contractURL = `${nearConfig.explorerUrl}/accounts/${nearConfig.contractName}`
  const content = `<a href="${contractURL}">${nearConfig.contractName}</a> ${window.accountId}`
  document.querySelectorAll('[data-behavior=account-id]').forEach(elem => {
    elem.innerHTML = content
  })
}

window.nearInitPromise = initContract()
  .then(() => {
    if (window.walletConnection.isSignedIn()) {
      signedOutFlow()
    } else {
      signedInFlow()
    }
  })
  .catch(console.error)
