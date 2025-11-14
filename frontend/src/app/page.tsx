'use client';

import Link from 'next/link';
import { GiSwordman, GiTrophyCup, GiCrystalBall } from 'react-icons/gi';
import { FaArrowRight } from 'react-icons/fa';

export default function Home() {
  const features = [
    {
      icon: <GiSwordman className="w-12 h-12" />,
      title: 'NFT Battles',
      description: 'Create unique characters and engage in thrilling rounds-based combat',
      href: '/battles',
      color: 'from-red-500 to-orange-500',
      bgClass: 'character-class-warrior',
    },
    {
      icon: <GiTrophyCup className="w-12 h-12" />,
      title: 'Prediction Markets',
      description: 'Bet on battle outcomes with AMM-style liquidity pools',
      href: '/markets',
      color: 'from-accent-500 to-blue-500',
      bgClass: 'bg-gradient-to-br from-accent-500/20 to-blue-700/20',
    },
    {
      icon: <GiCrystalBall className="w-12 h-12" />,
      title: 'Game Oracle',
      description: 'Submit verified results with staking and slashing protection',
      href: '/oracle',
      color: 'from-primary-500 to-purple-500',
      bgClass: 'bg-gradient-to-br from-primary-500/20 to-purple-700/20',
    },
  ];

  return (
    <div className="min-h-screen">
      {/* Hero Section */}
      <section className="relative overflow-hidden py-20 lg:py-32">
        <div className="absolute inset-0 bg-gradient-to-br from-primary-500/10 to-accent-500/10 animate-pulse-slow"></div>

        <div className="container mx-auto px-4 relative z-10">
          <div className="text-center max-w-4xl mx-auto">
            <h1 className="text-5xl lg:text-7xl font-bold mb-6 bg-gradient-to-r from-primary-400 via-accent-400 to-primary-400 bg-clip-text text-transparent animate-float">
              BattleChain
            </h1>
            <p className="text-xl lg:text-2xl text-gray-300 mb-8">
              The Ultimate Gaming Prediction Market Platform
            </p>
            <p className="text-lg text-gray-400 mb-12 max-w-2xl mx-auto">
              Create NFT characters, battle in rounds-based combat, and participate in prediction
              markets. Built on Polkadot SDK with verifiable on-chain results.
            </p>

            <div className="flex flex-col sm:flex-row gap-4 justify-center">
              <Link href="/battles" className="btn-primary inline-flex items-center justify-center space-x-2">
                <span>Start Battling</span>
                <FaArrowRight />
              </Link>
              <Link href="/markets" className="btn-outline inline-flex items-center justify-center">
                Explore Markets
              </Link>
            </div>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="py-20 bg-dark-card/50">
        <div className="container mx-auto px-4">
          <h2 className="text-3xl lg:text-4xl font-bold text-center mb-12">
            Platform Features
          </h2>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            {features.map((feature, index) => (
              <Link
                key={index}
                href={feature.href}
                className={`card-hover ${feature.bgClass} border-2 group`}
              >
                <div className={`w-20 h-20 bg-gradient-to-br ${feature.color} rounded-xl flex items-center justify-center mb-6 mx-auto group-hover:scale-110 transition-transform`}>
                  {feature.icon}
                </div>
                <h3 className="text-2xl font-bold text-center mb-4">{feature.title}</h3>
                <p className="text-gray-400 text-center">{feature.description}</p>
              </Link>
            ))}
          </div>
        </div>
      </section>

      {/* Stats Section */}
      <section className="py-20">
        <div className="container mx-auto px-4">
          <div className="grid grid-cols-2 lg:grid-cols-4 gap-6">
            <div className="stat-card">
              <div className="text-4xl font-bold text-primary-400 mb-2">0</div>
              <div className="text-gray-400 text-sm uppercase tracking-wider">Total Battles</div>
            </div>
            <div className="stat-card">
              <div className="text-4xl font-bold text-accent-400 mb-2">0</div>
              <div className="text-gray-400 text-sm uppercase tracking-wider">Active Markets</div>
            </div>
            <div className="stat-card">
              <div className="text-4xl font-bold text-purple-400 mb-2">0</div>
              <div className="text-gray-400 text-sm uppercase tracking-wider">Verified Results</div>
            </div>
            <div className="stat-card">
              <div className="text-4xl font-bold text-green-400 mb-2">0</div>
              <div className="text-gray-400 text-sm uppercase tracking-wider">Total Volume</div>
            </div>
          </div>
        </div>
      </section>

      {/* How It Works */}
      <section className="py-20 bg-dark-card/50">
        <div className="container mx-auto px-4">
          <h2 className="text-3xl lg:text-4xl font-bold text-center mb-12">How It Works</h2>

          <div className="max-w-4xl mx-auto space-y-8">
            <div className="flex items-start space-x-4">
              <div className="flex-shrink-0 w-12 h-12 bg-gradient-to-br from-primary-500 to-primary-700 rounded-full flex items-center justify-center text-white font-bold text-xl">
                1
              </div>
              <div>
                <h3 className="text-xl font-bold mb-2">Create Your Character</h3>
                <p className="text-gray-400">
                  Choose from 5 unique classes: Warrior, Assassin, Mage, Tank, or Trickster. Each has unique stats and abilities.
                </p>
              </div>
            </div>

            <div className="flex items-start space-x-4">
              <div className="flex-shrink-0 w-12 h-12 bg-gradient-to-br from-accent-500 to-accent-700 rounded-full flex items-center justify-center text-white font-bold text-xl">
                2
              </div>
              <div>
                <h3 className="text-xl font-bold mb-2">Battle & Predict</h3>
                <p className="text-gray-400">
                  Engage in rounds-based battles or place predictions on ongoing fights. Win rounds to claim victory!
                </p>
              </div>
            </div>

            <div className="flex items-start space-x-4">
              <div className="flex-shrink-0 w-12 h-12 bg-gradient-to-br from-purple-500 to-purple-700 rounded-full flex items-center justify-center text-white font-bold text-xl">
                3
              </div>
              <div>
                <h3 className="text-xl font-bold mb-2">Earn Rewards</h3>
                <p className="text-gray-400">
                  Win battles, profit from predictions, or earn as a verified game developer submitting results.
                </p>
              </div>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
}
