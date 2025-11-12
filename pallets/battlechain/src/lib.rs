#![cfg_attr(not(feature = "std"), no_std)]

//! # BattleChain Pallet - Rounds-Based Battle System
//!
//! Matches the Solidity BattleManager architecture:
//! - Rounds-based battles (3, 5, or 8 rounds)
//! - 3 turns per round executed together
//! - HP reset at start of each round
//! - Winner determined by rounds won (best of X)

pub use pallet::*;

#[frame::pallet(dev_mode)]
pub mod pallet {
    use frame::prelude::*;
    use sp_runtime::traits::{Hash, Zero, Saturating};
    use sp_runtime::Perbill;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type Currency: frame::traits::Currency<Self::AccountId>;
    }

    // ========== TYPES ==========

    /// Character class types
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum CharacterClass {
        Warrior,
        Assassin,
        Mage,
        Tank,
        Trickster,
    }

    /// Battle stance types (affects damage and defense)
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum StanceType {
        Balanced,      // No modifiers
        Aggressive,    // +30% damage, +50% damage taken
        Defensive,     // -30% damage, -50% damage taken
        Berserker,     // +100% damage, +25% self-harm
        Counter,       // -10% damage, 40% reflection
    }

    /// Battle state
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum BattleState {
        Active,
        Finished,
    }

    /// Wildcard effect types
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum WildcardType {
        DamageBoost,    // +40% damage to both players
        Heal,           // +10 HP to both
        Stun,           // Skip next turn
        NoEffect,       // Nothing happens
        DoubleDamage,   // 2x damage to both
    }

    /// Character NFT data
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Character<T: Config> {
        pub character_id: u64,
        pub owner: T::AccountId,
        pub class: CharacterClass,
        pub max_hp: u32,
        pub damage_min: u16,
        pub damage_max: u16,
        pub crit_chance: u16,      // basis points (0-10000)
        pub dodge_chance: u16,     // basis points
        pub defense: u16,
        pub level: u16,
        pub xp: u64,
        pub wins: u32,
        pub losses: u32,
        pub mmr: i32,
    }

    /// Battle character (temporary stats during battle)
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct BattleCharacter {
        pub character_id: u64,
        pub max_hp: u32,
        pub current_hp: u32,
        pub damage_min: u16,
        pub damage_max: u16,
        pub crit_chance: u16,
        pub dodge_chance: u16,
    }

    /// Round result
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct RoundResult<T: Config> {
        pub winner: Option<T::AccountId>,
        pub turns_executed: u8,
        pub player1_final_hp: u32,
        pub player2_final_hp: u32,
        pub completed_at: BlockNumberFor<T>,
    }

    /// Battle data
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Battle<T: Config> {
        pub battle_id: T::Hash,
        pub player1: T::AccountId,
        pub player2: T::AccountId,
        pub character1_id: u64,
        pub character2_id: u64,
        pub max_rounds: u8,              // 3, 5, or 8
        pub current_round: u8,
        pub rounds_completed: u8,
        pub player1_rounds_won: u8,
        pub player2_rounds_won: u8,
        pub state: BattleState,
        pub stake_amount: BalanceOf<T>,
        pub created_at: BlockNumberFor<T>,
    }

    type BalanceOf<T> = <<T as Config>::Currency as frame::traits::Currency<<T as frame_system::Config>::AccountId>>::Balance;

    // ========== STORAGE ==========

    /// Character NFTs by ID
    #[pallet::storage]
    pub type Characters<T: Config> = StorageMap<_, Blake2_128Concat, u64, Character<T>>;

    /// Next character ID
    #[pallet::storage]
    pub type NextCharacterId<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Characters owned by account
    #[pallet::storage]
    pub type OwnedCharacters<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, ConstU32<100>>,
        ValueQuery,
    >;

    /// Active battles
    #[pallet::storage]
    pub type Battles<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, Battle<T>>;

    /// Battle characters (temporary stats during battle)
    #[pallet::storage]
    pub type BattleCharacters<T: Config> = StorageMap<_, Blake2_128Concat, u64, BattleCharacter>;

    /// Round results for battles
    #[pallet::storage]
    pub type RoundResults<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::Hash,      // battle_id
        Blake2_128Concat,
        u8,           // round number
        RoundResult<T>,
    >;

    /// Battle offers (matchmaking)
    #[pallet::storage]
    pub type BattleOffers<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::Hash,
        (T::AccountId, u64, BalanceOf<T>, u8), // (creator, character_id, stake, max_rounds)
    >;

    // ========== EVENTS ==========

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Character created [owner, character_id, class]
        CharacterCreated {
            owner: T::AccountId,
            character_id: u64,
            class: CharacterClass,
        },
        /// Battle offer created [battle_id, creator, character_id, stake, max_rounds]
        BattleOfferCreated {
            battle_id: T::Hash,
            creator: T::AccountId,
            character_id: u64,
            stake: BalanceOf<T>,
            max_rounds: u8,
        },
        /// Battle started [battle_id, player1, player2, max_rounds]
        BattleStarted {
            battle_id: T::Hash,
            player1: T::AccountId,
            player2: T::AccountId,
            max_rounds: u8,
        },
        /// Round executed [battle_id, round, winner, turns_executed]
        RoundExecuted {
            battle_id: T::Hash,
            round: u8,
            winner: Option<T::AccountId>,
            turns_executed: u8,
        },
        /// Battle completed [battle_id, winner, total_rounds]
        BattleCompleted {
            battle_id: T::Hash,
            winner: Option<T::AccountId>,
            total_rounds: u8,
        },
        /// Turn executed [battle_id, round, turn, damage_dealt]
        TurnExecuted {
            battle_id: T::Hash,
            round: u8,
            turn: u8,
            damage_dealt: u64,
        },
    }

    // ========== ERRORS ==========

    #[pallet::error]
    pub enum Error<T> {
        CharacterNotFound,
        NotCharacterOwner,
        BattleNotFound,
        BattleNotActive,
        InvalidRoundCount,
        RoundAlreadyInProgress,
        MaxRoundsReached,
        CannotBattleSelf,
        TooManyCharacters,
        BattleOfferNotFound,
    }

    // ========== EXTRINSICS ==========

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new character NFT
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn create_character(
            origin: OriginFor<T>,
            class: CharacterClass,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let character_id = NextCharacterId::<T>::get();
            NextCharacterId::<T>::put(character_id.saturating_add(1));

            let (max_hp, damage_min, damage_max, crit, dodge, defense) = match class {
                CharacterClass::Warrior => (120, 8, 15, 1500, 500, 10),
                CharacterClass::Assassin => (90, 12, 20, 3500, 1500, 5),
                CharacterClass::Mage => (80, 10, 18, 2000, 800, 5),
                CharacterClass::Tank => (150, 6, 12, 1000, 300, 20),
                CharacterClass::Trickster => (100, 8, 16, 2500, 1200, 8),
            };

            let character = Character {
                character_id,
                owner: who.clone(),
                class: class.clone(),
                max_hp,
                damage_min,
                damage_max,
                crit_chance: crit,
                dodge_chance: dodge,
                defense,
                level: 1,
                xp: 0,
                wins: 0,
                losses: 0,
                mmr: 1000,
            };

            Characters::<T>::insert(character_id, character);
            OwnedCharacters::<T>::try_mutate(&who, |chars| {
                chars.try_push(character_id).map_err(|_| Error::<T>::TooManyCharacters)
            })?;

            Self::deposit_event(Event::CharacterCreated {
                owner: who,
                character_id,
                class,
            });

            Ok(())
        }

        /// Create a battle offer
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn create_battle_offer(
            origin: OriginFor<T>,
            character_id: u64,
            stake: BalanceOf<T>,
            max_rounds: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                max_rounds == 3 || max_rounds == 5 || max_rounds == 8,
                Error::<T>::InvalidRoundCount
            );

            let character = Characters::<T>::get(character_id)
                .ok_or(Error::<T>::CharacterNotFound)?;
            ensure!(character.owner == who, Error::<T>::NotCharacterOwner);

            let battle_id = T::Hashing::hash_of(&(&who, &character_id, &frame_system::Pallet::<T>::block_number()));

            BattleOffers::<T>::insert(battle_id, (who.clone(), character_id, stake, max_rounds));

            Self::deposit_event(Event::BattleOfferCreated {
                battle_id,
                creator: who,
                character_id,
                stake,
                max_rounds,
            });

            Ok(())
        }

        /// Accept a battle offer (starts the battle)
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn accept_battle(
            origin: OriginFor<T>,
            battle_id: T::Hash,
            character_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let (creator, creator_char_id, stake, max_rounds) = BattleOffers::<T>::get(battle_id)
                .ok_or(Error::<T>::BattleOfferNotFound)?;

            ensure!(who != creator, Error::<T>::CannotBattleSelf);

            let character = Characters::<T>::get(character_id)
                .ok_or(Error::<T>::CharacterNotFound)?;
            ensure!(character.owner == who, Error::<T>::NotCharacterOwner);

            // Create battle
            let battle = Battle {
                battle_id,
                player1: creator.clone(),
                player2: who.clone(),
                character1_id: creator_char_id,
                character2_id: character_id,
                max_rounds,
                current_round: 1,
                rounds_completed: 0,
                player1_rounds_won: 0,
                player2_rounds_won: 0,
                state: BattleState::Active,
                stake_amount: stake,
                created_at: frame_system::Pallet::<T>::block_number(),
            };

            Battles::<T>::insert(battle_id, battle);
            BattleOffers::<T>::remove(battle_id);

            // Initialize battle characters
            Self::initialize_battle_characters(creator_char_id, character_id)?;

            Self::deposit_event(Event::BattleStarted {
                battle_id,
                player1: creator,
                player2: who,
                max_rounds,
            });

            Ok(())
        }

        /// Execute a complete round (3 turns) in the battle
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn execute_round(
            origin: OriginFor<T>,
            battle_id: T::Hash,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            let mut battle = Battles::<T>::get(battle_id).ok_or(Error::<T>::BattleNotFound)?;
            ensure!(battle.state == BattleState::Active, Error::<T>::BattleNotActive);
            ensure!(battle.current_round <= battle.max_rounds, Error::<T>::MaxRoundsReached);

            // Reset characters to full HP for this round
            Self::reset_characters_for_round(battle.character1_id, battle.character2_id)?;

            // Execute 3 turns
            let mut round_winner: Option<T::AccountId> = None;
            let mut turns_executed = 0u8;

            for turn in 1..=3 {
                turns_executed = turn;

                // Execute single turn
                let damage_dealt = Self::execute_turn(battle_id, &battle, battle.current_round, turn)?;

                Self::deposit_event(Event::TurnExecuted {
                    battle_id,
                    round: battle.current_round,
                    turn,
                    damage_dealt,
                });

                // Check if someone died (round ends early)
                let char1 = BattleCharacters::<T>::get(battle.character1_id).unwrap();
                let char2 = BattleCharacters::<T>::get(battle.character2_id).unwrap();

                if char1.current_hp == 0 {
                    round_winner = Some(battle.player2.clone());
                    break;
                } else if char2.current_hp == 0 {
                    round_winner = Some(battle.player1.clone());
                    break;
                }
            }

            // If no one died, winner is whoever has more HP
            if round_winner.is_none() {
                let char1 = BattleCharacters::<T>::get(battle.character1_id).unwrap();
                let char2 = BattleCharacters::<T>::get(battle.character2_id).unwrap();

                if char1.current_hp > char2.current_hp {
                    round_winner = Some(battle.player1.clone());
                } else if char2.current_hp > char1.current_hp {
                    round_winner = Some(battle.player2.clone());
                }
            }

            // Award round to winner
            if let Some(ref winner) = round_winner {
                if winner == &battle.player1 {
                    battle.player1_rounds_won = battle.player1_rounds_won.saturating_add(1);
                } else {
                    battle.player2_rounds_won = battle.player2_rounds_won.saturating_add(1);
                }
            }

            battle.rounds_completed = battle.rounds_completed.saturating_add(1);

            // Store round result
            let char1 = BattleCharacters::<T>::get(battle.character1_id).unwrap();
            let char2 = BattleCharacters::<T>::get(battle.character2_id).unwrap();

            RoundResults::<T>::insert(
                battle_id,
                battle.current_round,
                RoundResult {
                    winner: round_winner.clone(),
                    turns_executed,
                    player1_final_hp: char1.current_hp,
                    player2_final_hp: char2.current_hp,
                    completed_at: frame_system::Pallet::<T>::block_number(),
                },
            );

            Self::deposit_event(Event::RoundExecuted {
                battle_id,
                round: battle.current_round,
                winner: round_winner.clone(),
                turns_executed,
            });

            // Check if battle is complete
            let rounds_to_win = (battle.max_rounds / 2) + 1;
            let mut battle_complete = false;
            let mut battle_winner: Option<T::AccountId> = None;

            if battle.player1_rounds_won >= rounds_to_win {
                battle_winner = Some(battle.player1.clone());
                battle_complete = true;
            } else if battle.player2_rounds_won >= rounds_to_win {
                battle_winner = Some(battle.player2.clone());
                battle_complete = true;
            } else if battle.rounds_completed >= battle.max_rounds {
                if battle.player1_rounds_won > battle.player2_rounds_won {
                    battle_winner = Some(battle.player1.clone());
                } else if battle.player2_rounds_won > battle.player1_rounds_won {
                    battle_winner = Some(battle.player2.clone());
                }
                battle_complete = true;
            }

            if battle_complete {
                battle.state = BattleState::Finished;

                Self::deposit_event(Event::BattleCompleted {
                    battle_id,
                    winner: battle_winner.clone(),
                    total_rounds: battle.rounds_completed,
                });

                // Update character stats
                Self::update_characters_after_battle(&battle, battle_winner)?;
            } else {
                battle.current_round = battle.current_round.saturating_add(1);
            }

            Battles::<T>::insert(battle_id, battle);

            Ok(())
        }
    }

    // ========== HELPER FUNCTIONS ==========

    impl<T: Config> Pallet<T> {
        /// Initialize battle characters from NFT characters
        fn initialize_battle_characters(char1_id: u64, char2_id: u64) -> DispatchResult {
            let char1 = Characters::<T>::get(char1_id).ok_or(Error::<T>::CharacterNotFound)?;
            let char2 = Characters::<T>::get(char2_id).ok_or(Error::<T>::CharacterNotFound)?;

            BattleCharacters::<T>::insert(char1_id, BattleCharacter {
                character_id: char1_id,
                max_hp: char1.max_hp,
                current_hp: char1.max_hp,
                damage_min: char1.damage_min,
                damage_max: char1.damage_max,
                crit_chance: char1.crit_chance,
                dodge_chance: char1.dodge_chance,
            });

            BattleCharacters::<T>::insert(char2_id, BattleCharacter {
                character_id: char2_id,
                max_hp: char2.max_hp,
                current_hp: char2.max_hp,
                damage_min: char2.damage_min,
                damage_max: char2.damage_max,
                crit_chance: char2.crit_chance,
                dodge_chance: char2.dodge_chance,
            });

            Ok(())
        }

        /// Reset characters to full HP for new round
        fn reset_characters_for_round(char1_id: u64, char2_id: u64) -> DispatchResult {
            BattleCharacters::<T>::try_mutate(char1_id, |maybe_char| {
                if let Some(char) = maybe_char {
                    char.current_hp = char.max_hp;
                }
                Ok::<(), DispatchError>(())
            })?;

            BattleCharacters::<T>::try_mutate(char2_id, |maybe_char| {
                if let Some(char) = maybe_char {
                    char.current_hp = char.max_hp;
                }
                Ok::<(), DispatchError>(())
            })?;

            Ok(())
        }

        /// Execute a single turn
        fn execute_turn(
            _battle_id: T::Hash,
            battle: &Battle<T>,
            _round: u8,
            _turn: u8,
        ) -> Result<u64, DispatchError> {
            // Get battle characters
            let mut char1 = BattleCharacters::<T>::get(battle.character1_id).unwrap();
            let mut char2 = BattleCharacters::<T>::get(battle.character2_id).unwrap();

            // Generate random stances (simplified - using block number as entropy)
            let stance1 = StanceType::Balanced;
            let stance2 = StanceType::Balanced;

            // Calculate damage (simplified)
            let damage1 = ((char1.damage_min + char1.damage_max) as u64) / 2;
            let damage2 = ((char2.damage_min + char2.damage_max) as u64) / 2;

            // Apply damage
            char2.current_hp = char2.current_hp.saturating_sub(damage1 as u32);
            char1.current_hp = char1.current_hp.saturating_sub(damage2 as u32);

            // Update battle characters
            BattleCharacters::<T>::insert(battle.character1_id, char1);
            BattleCharacters::<T>::insert(battle.character2_id, char2);

            Ok(damage1.saturating_add(damage2))
        }

        /// Update characters after battle completion
        fn update_characters_after_battle(
            battle: &Battle<T>,
            winner: Option<T::AccountId>,
        ) -> DispatchResult {
            // Update player 1
            Characters::<T>::try_mutate(battle.character1_id, |maybe_char| {
                if let Some(char) = maybe_char {
                    let won = winner.as_ref() == Some(&battle.player1);
                    let xp = if won { 100 } else { 25 };
                    char.xp = char.xp.saturating_add(xp);
                    if won {
                        char.wins = char.wins.saturating_add(1);
                    } else {
                        char.losses = char.losses.saturating_add(1);
                    }
                }
                Ok::<(), DispatchError>(())
            })?;

            // Update player 2
            Characters::<T>::try_mutate(battle.character2_id, |maybe_char| {
                if let Some(char) = maybe_char {
                    let won = winner.as_ref() == Some(&battle.player2);
                    let xp = if won { 100 } else { 25 };
                    char.xp = char.xp.saturating_add(xp);
                    if won {
                        char.wins = char.wins.saturating_add(1);
                    } else {
                        char.losses = char.losses.saturating_add(1);
                    }
                }
                Ok::<(), DispatchError>(())
            })?;

            Ok(())
        }
    }
}
