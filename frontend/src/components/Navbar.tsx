'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { useWalletStore } from '@/stores/walletStore';
import { connectWallet, initializeApi, formatBalance } from '@/lib/polkadot';
import { GiSwordman, GiTrophyCup, GiCrystalBall } from 'react-icons/gi';
import { FaWallet } from 'react-icons/fa';

export default function Navbar() {
  const {
    api,
    accounts,
    selectedAccount,
    isConnected,
    setApi,
    setAccounts,
    setSelectedAccount,
    setIsConnected,
  } = useWalletStore();

  const [balance, setBalance] = useState<string>('0');
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    // Initialize API on mount
    initializeApi()
      .then((api) => {
        setApi(api);
      })
      .catch((error) => {
        console.error('Failed to initialize API:', error);
      });
  }, [setApi]);

  useEffect(() => {
    // Fetch balance when account changes
    if (api && selectedAccount) {
      const unsubscribe = api.query.system
        .account(selectedAccount.address, ({ data }) => {
          setBalance(formatBalance(data.free.toBigInt()));
        })
        .catch(console.error);

      return () => {
        if (typeof unsubscribe === 'function') {
          unsubscribe();
        }
      };
    }
  }, [api, selectedAccount]);

  const handleConnectWallet = async () => {
    try {
      setIsLoading(true);
      const allAccounts = await connectWallet();

      if (allAccounts.length > 0) {
        setAccounts(allAccounts);
        setSelectedAccount(allAccounts[0]);
        setIsConnected(true);
      }
    } catch (error) {
      console.error('Failed to connect wallet:', error);
      alert('Please install Polkadot.js extension');
    } finally {
      setIsLoading(false);
    }
  };

  const handleSelectAccount = (account: typeof selectedAccount) => {
    setSelectedAccount(account);
  };

  return (
    <nav className="sticky top-0 z-50 bg-dark-card/95 backdrop-blur-lg border-b border-dark-border">
      <div className="container mx-auto px-4">
        <div className="flex items-center justify-between h-16">
          {/* Logo */}
          <Link href="/" className="flex items-center space-x-3 group">
            <div className="w-10 h-10 bg-gradient-to-br from-primary-500 to-accent-500 rounded-lg flex items-center justify-center group-hover:shadow-glow transition-all">
              <GiSwordman className="w-6 h-6 text-white" />
            </div>
            <span className="text-xl font-bold bg-gradient-to-r from-primary-400 to-accent-400 bg-clip-text text-transparent">
              BattleChain
            </span>
          </Link>

          {/* Navigation Links */}
          <div className="hidden md:flex items-center space-x-6">
            <Link
              href="/battles"
              className="flex items-center space-x-2 text-gray-300 hover:text-primary-400 transition-colors"
            >
              <GiSwordman className="w-5 h-5" />
              <span>Battles</span>
            </Link>
            <Link
              href="/markets"
              className="flex items-center space-x-2 text-gray-300 hover:text-accent-400 transition-colors"
            >
              <GiTrophyCup className="w-5 h-5" />
              <span>Markets</span>
            </Link>
            <Link
              href="/oracle"
              className="flex items-center space-x-2 text-gray-300 hover:text-purple-400 transition-colors"
            >
              <GiCrystalBall className="w-5 h-5" />
              <span>Oracle</span>
            </Link>
          </div>

          {/* Wallet Connection */}
          <div className="flex items-center space-x-4">
            {isConnected && selectedAccount ? (
              <div className="flex items-center space-x-3">
                {/* Balance Display */}
                <div className="hidden sm:block text-sm">
                  <div className="text-gray-400">Balance</div>
                  <div className="text-primary-400 font-semibold">{balance} UNIT</div>
                </div>

                {/* Account Selector */}
                <div className="relative">
                  <select
                    className="appearance-none bg-dark-bg border border-dark-border rounded-lg px-4 py-2 pr-8 text-sm focus:outline-none focus:border-primary-500 cursor-pointer"
                    value={selectedAccount.address}
                    onChange={(e) => {
                      const account = accounts.find((acc) => acc.address === e.target.value);
                      if (account) handleSelectAccount(account);
                    }}
                  >
                    {accounts.map((account) => (
                      <option key={account.address} value={account.address}>
                        {account.meta.name || account.address.slice(0, 8) + '...'}
                      </option>
                    ))}
                  </select>
                  <div className="absolute right-2 top-1/2 transform -translate-y-1/2 pointer-events-none">
                    <svg className="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                    </svg>
                  </div>
                </div>
              </div>
            ) : (
              <button
                onClick={handleConnectWallet}
                disabled={isLoading}
                className="btn-primary flex items-center space-x-2"
              >
                <FaWallet className="w-4 h-4" />
                <span>{isLoading ? 'Connecting...' : 'Connect Wallet'}</span>
              </button>
            )}
          </div>
        </div>
      </div>
    </nav>
  );
}
