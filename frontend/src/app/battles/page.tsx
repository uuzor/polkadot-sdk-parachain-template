'use client';

import { useState } from 'react';
import { useWalletStore } from '@/stores/walletStore';
import { getInjector, CharacterClass, CharacterClassNames } from '@/lib/polkadot';
import { GiSwordman, GiCrossedSwords, GiMagicHat, GiShield, GiJesterHat } from 'react-icons/gi';

const CHARACTER_CLASSES = [
  {
    id: CharacterClass.Warrior,
    name: CharacterClassNames[CharacterClass.Warrior],
    icon: <GiSwordman className="w-12 h-12" />,
    description: 'High HP and balanced damage. Great for beginners.',
    stats: { hp: 120, damage: '8-15', crit: '15%', dodge: '5%', defense: 10 },
    class: 'character-class-warrior',
  },
  {
    id: CharacterClass.Assassin,
    name: CharacterClassNames[CharacterClass.Assassin],
    icon: <GiCrossedSwords className="w-12 h-12" />,
    description: 'High damage and crit chance. Low HP.',
    stats: { hp: 90, damage: '12-20', crit: '35%', dodge: '15%', defense: 5 },
    class: 'character-class-assassin',
  },
  {
    id: CharacterClass.Mage,
    name: CharacterClassNames[CharacterClass.Mage],
    icon: <GiMagicHat className="w-12 h-12" />,
    description: 'Moderate damage with good crit. Low defense.',
    stats: { hp: 80, damage: '10-18', crit: '20%', dodge: '8%', defense: 5 },
    class: 'character-class-mage',
  },
  {
    id: CharacterClass.Tank,
    name: CharacterClassNames[CharacterClass.Tank],
    icon: <GiShield className="w-12 h-12" />,
    description: 'Highest HP and defense. Lower damage.',
    stats: { hp: 150, damage: '6-12', crit: '10%', dodge: '3%', defense: 20 },
    class: 'character-class-tank',
  },
  {
    id: CharacterClass.Trickster,
    name: CharacterClassNames[CharacterClass.Trickster],
    icon: <GiJesterHat className="w-12 h-12" />,
    description: 'Balanced stats with high dodge chance.',
    stats: { hp: 100, damage: '8-16', crit: '25%', dodge: '12%', defense: 8 },
    class: 'character-class-trickster',
  },
];

export default function BattlesPage() {
  const { api, selectedAccount, isConnected } = useWalletStore();
  const [selectedClass, setSelectedClass] = useState<CharacterClass | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const [txStatus, setTxStatus] = useState<string>('');

  const handleCreateCharacter = async () => {
    if (!api || !selectedAccount || selectedClass === null) return;

    try {
      setIsCreating(true);
      setTxStatus('Preparing transaction...');

      const injector = await getInjector(selectedAccount.address);

      const tx = api.tx.battleChain.createCharacter(selectedClass);

      setTxStatus('Waiting for signature...');

      await tx.signAndSend(
        selectedAccount.address,
        { signer: injector.signer },
        ({ status, events }) => {
          if (status.isInBlock) {
            setTxStatus(`In block: ${status.asInBlock.toString()}`);
          } else if (status.isFinalized) {
            setTxStatus('Character created successfully!');
            setTimeout(() => {
              setTxStatus('');
              setSelectedClass(null);
            }, 3000);
          }
        }
      );
    } catch (error) {
      console.error('Failed to create character:', error);
      setTxStatus('Failed to create character. Please try again.');
    } finally {
      setIsCreating(false);
    }
  };

  if (!isConnected) {
    return (
      <div className="container mx-auto px-4 py-20">
        <div className="card max-w-md mx-auto text-center">
          <GiSwordman className="w-20 h-20 mx-auto mb-6 text-primary-500" />
          <h2 className="text-2xl font-bold mb-4">Connect Your Wallet</h2>
          <p className="text-gray-400">
            Please connect your wallet to create characters and start battling.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="container mx-auto px-4 py-12">
      <div className="text-center mb-12">
        <h1 className="text-4xl lg:text-5xl font-bold mb-4 bg-gradient-to-r from-primary-400 to-accent-400 bg-clip-text text-transparent">
          Create Your Character
        </h1>
        <p className="text-gray-400 text-lg">
          Choose your class and enter the battle arena
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-6 mb-12">
        {CHARACTER_CLASSES.map((char) => (
          <div
            key={char.id}
            onClick={() => setSelectedClass(char.id)}
            className={`card-hover cursor-pointer ${char.class} ${
              selectedClass === char.id ? 'ring-2 ring-primary-500 shadow-glow' : ''
            }`}
          >
            <div className="text-center">
              <div className="mb-4">{char.icon}</div>
              <h3 className="text-xl font-bold mb-2">{char.name}</h3>
              <p className="text-sm text-gray-400 mb-4">{char.description}</p>

              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-gray-400">HP:</span>
                  <span className="font-semibold">{char.stats.hp}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Damage:</span>
                  <span className="font-semibold">{char.stats.damage}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Crit:</span>
                  <span className="font-semibold">{char.stats.crit}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Dodge:</span>
                  <span className="font-semibold">{char.stats.dodge}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Defense:</span>
                  <span className="font-semibold">{char.stats.defense}</span>
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>

      {selectedClass !== null && (
        <div className="card max-w-md mx-auto text-center">
          <h3 className="text-2xl font-bold mb-4">
            Creating: {CharacterClassNames[selectedClass]}
          </h3>

          {txStatus && (
            <div className="mb-4 p-4 bg-dark-bg rounded-lg">
              <p className="text-sm text-gray-300">{txStatus}</p>
            </div>
          )}

          <button
            onClick={handleCreateCharacter}
            disabled={isCreating}
            className="btn-primary w-full"
          >
            {isCreating ? 'Creating...' : 'Create Character'}
          </button>

          {!isCreating && (
            <button
              onClick={() => setSelectedClass(null)}
              className="mt-3 text-gray-400 hover:text-white transition-colors"
            >
              Cancel
            </button>
          )}
        </div>
      )}

      {/* My Characters Section - Placeholder */}
      <div className="mt-20">
        <h2 className="text-3xl font-bold mb-8">My Characters</h2>
        <div className="card text-center py-12">
          <p className="text-gray-400">No characters yet. Create one above to get started!</p>
        </div>
      </div>
    </div>
  );
}
