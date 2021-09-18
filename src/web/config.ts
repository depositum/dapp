import {environment} from './environments/environment'

export interface NetworkConfig {
  networkId: string
  nodeUrl: string
  contractName: string
  walletUrl?: string
  helperUrl?: string
  explorerUrl?: string
  keyPath?: string
  masterAccount?: string
}

const developList = [
  'local',
]

const productionList = [
  'testnet',
  'betanet',
]

export function configList(): NetworkConfig[] {
  if (!environment.production) {
    productionList.push(...developList)
  }
  return productionList.map(it => config(it))
}

export function config(env: string): NetworkConfig {
  switch (env) {
    case 'production':
    case 'mainnet':
      return {
        networkId: 'mainnet',
        nodeUrl: 'https://rpc.mainnet.near.org',
        contractName: 'alpha.depositum.near',
        walletUrl: 'https://wallet.near.org',
        helperUrl: 'https://helper.mainnet.near.org',
        explorerUrl: 'https://explorer.mainnet.near.org',
      }
    case 'development':
    case 'testnet':
      return {
        networkId: 'testnet',
        nodeUrl: 'https://rpc.testnet.near.org',
        contractName: 'alpha.depositum.testnet',
        walletUrl: 'https://wallet.testnet.near.org',
        helperUrl: 'https://helper.testnet.near.org',
        explorerUrl: 'https://explorer.testnet.near.org',
      }
    case 'betanet':
      return {
        networkId: 'betanet',
        nodeUrl: 'https://rpc.betanet.near.org',
        contractName: 'alpha.depositum.betanet',
        walletUrl: 'https://wallet.betanet.near.org',
        helperUrl: 'https://helper.betanet.near.org',
        explorerUrl: 'https://explorer.betanet.near.org',
      }
    case 'local':
      return {
        networkId: 'local',
        nodeUrl: 'http://localhost:3030',
        contractName: 'alpha.depositum.local',
        walletUrl: 'http://localhost:4000/wallet',
        helperUrl: 'http://localhost:3000',
      }
    default:
      throw Error(`Unconfigured environment '${env}'`)
  }
}
