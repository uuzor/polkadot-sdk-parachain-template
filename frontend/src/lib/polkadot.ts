import { ApiPromise, WsProvider } from '@polkadot/api';
import { web3Accounts, web3Enable, web3FromAddress } from '@polkadot/extension-dapp';
import type { InjectedAccountWithMeta } from '@polkadot/extension-inject/types';

const WS_PROVIDER = process.env.NEXT_PUBLIC_WS_PROVIDER || 'ws://127.0.0.1:9944';

let api: ApiPromise | null = null;

export async function initializeApi(): Promise<ApiPromise> {
  if (api) return api;

  const provider = new WsProvider(WS_PROVIDER);
  api = await ApiPromise.create({ provider });

  return api;
}

export async function getApi(): Promise<ApiPromise> {
  if (!api) {
    return initializeApi();
  }
  return api;
}

export async function connectWallet(): Promise<InjectedAccountWithMeta[]> {
  // Enable the extension
  const extensions = await web3Enable('BattleChain');

  if (extensions.length === 0) {
    throw new Error('No extension installed');
  }

  // Get all accounts
  const allAccounts = await web3Accounts();

  return allAccounts;
}

export async function getInjector(address: string) {
  const injector = await web3FromAddress(address);
  return injector;
}

export function disconnectApi() {
  if (api) {
    api.disconnect();
    api = null;
  }
}

// Helper to format balance
export function formatBalance(balance: bigint): string {
  const decimals = 12; // Standard for Polkadot/Substrate
  const balanceStr = balance.toString();

  if (balanceStr.length <= decimals) {
    return `0.${balanceStr.padStart(decimals, '0')}`;
  }

  const integerPart = balanceStr.slice(0, -decimals);
  const decimalPart = balanceStr.slice(-decimals);

  return `${integerPart}.${decimalPart.slice(0, 4)}`;
}

// Character classes
export enum CharacterClass {
  Warrior = 0,
  Assassin = 1,
  Mage = 2,
  Tank = 3,
  Trickster = 4,
}

export const CharacterClassNames = {
  [CharacterClass.Warrior]: 'Warrior',
  [CharacterClass.Assassin]: 'Assassin',
  [CharacterClass.Mage]: 'Mage',
  [CharacterClass.Tank]: 'Tank',
  [CharacterClass.Trickster]: 'Trickster',
};

// Market outcomes
export enum MarketOutcome {
  Player1Wins = 0,
  Player2Wins = 1,
  Draw = 2,
}

export const MarketOutcomeNames = {
  [MarketOutcome.Player1Wins]: 'Player 1 Wins',
  [MarketOutcome.Player2Wins]: 'Player 2 Wins',
  [MarketOutcome.Draw]: 'Draw',
};
