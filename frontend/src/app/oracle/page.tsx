'use client';

import { GiCrystalBall } from 'react-icons/gi';

export default function OraclePage() {
  return (
    <div className="container mx-auto px-4 py-20">
      <div className="text-center mb-12">
        <GiCrystalBall className="w-20 h-20 mx-auto mb-6 text-purple-500" />
        <h1 className="text-4xl lg:text-5xl font-bold mb-4 bg-gradient-to-r from-primary-400 to-purple-400 bg-clip-text text-transparent">
          Game Oracle
        </h1>
        <p className="text-gray-400 text-lg">
          Submit verified game results with staking and slashing protection
        </p>
      </div>

      <div className="card text-center py-12">
        <p className="text-gray-400 text-lg mb-4">
          Developer portal coming soon!
        </p>
        <p className="text-gray-500 text-sm">
          Register as a developer, submit verified results, and earn revenue.
        </p>
      </div>
    </div>
  );
}
