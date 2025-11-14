import { create } from 'zustand';
import type { InjectedAccountWithMeta } from '@polkadot/extension-inject/types';
import { ApiPromise } from '@polkadot/api';

interface WalletState {
  api: ApiPromise | null;
  accounts: InjectedAccountWithMeta[];
  selectedAccount: InjectedAccountWithMeta | null;
  isConnected: boolean;
  isLoading: boolean;
  error: string | null;

  setApi: (api: ApiPromise) => void;
  setAccounts: (accounts: InjectedAccountWithMeta[]) => void;
  setSelectedAccount: (account: InjectedAccountWithMeta | null) => void;
  setIsConnected: (isConnected: boolean) => void;
  setIsLoading: (isLoading: boolean) => void;
  setError: (error: string | null) => void;
  reset: () => void;
}

export const useWalletStore = create<WalletState>((set) => ({
  api: null,
  accounts: [],
  selectedAccount: null,
  isConnected: false,
  isLoading: false,
  error: null,

  setApi: (api) => set({ api }),
  setAccounts: (accounts) => set({ accounts }),
  setSelectedAccount: (account) => set({ selectedAccount: account }),
  setIsConnected: (isConnected) => set({ isConnected }),
  setIsLoading: (isLoading) => set({ isLoading }),
  setError: (error) => set({ error }),
  reset: () =>
    set({
      api: null,
      accounts: [],
      selectedAccount: null,
      isConnected: false,
      isLoading: false,
      error: null,
    }),
}));
