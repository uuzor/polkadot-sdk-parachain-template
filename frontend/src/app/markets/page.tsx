'use client';

import { GiTrophyCup } from 'react-icons/gi';

export default function MarketsPage() {
  return (
    <div className="container mx-auto px-4 py-20">
      <div className="text-center mb-12">
        <GiTrophyCup className="w-20 h-20 mx-auto mb-6 text-accent-500" />
        <h1 className="text-4xl lg:text-5xl font-bold mb-4 bg-gradient-to-r from-accent-400 to-blue-400 bg-clip-text text-transparent">
          Prediction Markets
        </h1>
        <p className="text-gray-400 text-lg">
          Bet on battle outcomes with AMM-style liquidity pools
        </p>
      </div>

      <div className="card text-center py-12">
        <p className="text-gray-400 text-lg mb-4">
          Prediction markets coming soon!
        </p>
        <p className="text-gray-500 text-sm">
          Create markets for battles, place predictions, and earn rewards.
        </p>
      </div>
    </div>
  );
}
