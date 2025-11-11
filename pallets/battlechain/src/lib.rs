#![cfg_attr(not(feature = "std"), no_std)]

//! # BattleChain Pallet
//!
//! NFT-based battle game with turn-based combat, character classes, and staking.

pub use pallet::*;

#[frame::pallet(dev_mode)]
pub mod pallet {
    use frame::prelude::*;
    use sp_runtime::traits::{Hash, Zero};
    use sp_runtime::Saturating;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The currency mechanism for staking
        type Currency: frame::traits::Currency<Self::AccountId>;

        /// Maximum number of active battles per player
        #[pallet::constant]
        type MaxActiveBattles: Get<u32>;

        /// Battle inactivity timeout in blocks
        #[pallet::constant]
        type InactivityTimeout: Get<BlockNumberFor<Self>>;
    }

    // ========== STORAGE ==========

    /// Character classes
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum CharacterClass {
        Warrior,
        Assassin,
        Mage,
        Tank,
        Trickster,
    }

    /// Battle stance types
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum StanceType {
        Balanced,
        Aggressive,
        Defensive,
        Berserker,
        Counter,
    }

    /// Battle state
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum BattleState {
        Pending,
        Active,
        Finished,
    }

    /// Character data
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Character<T: Config> {
        pub owner: T::AccountId,
        pub class: CharacterClass,
        pub max_hp: u32,
        pub current_hp: u32,
        pub base_damage_min: u16,
        pub base_damage_max: u16,
        pub crit_chance: u16, // basis points (0-10000)
        pub dodge_chance: u16, // basis points
        pub defense: u16,
        pub level: u16,
        pub xp: u64,
        pub wins: u32,
        pub losses: u32,
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
        pub player1_health: u64,
        pub player2_health: u64,
        pub current_turn: u8, // 1 or 2
        pub turn_number: u64,
        pub state: BattleState,
        pub player1_stance: StanceType,
        pub player2_stance: StanceType,
        pub stake_amount: BalanceOf<T>,
        pub winner: Option<T::AccountId>,
        pub created_at: BlockNumberFor<T>,
        pub last_action: BlockNumberFor<T>,
    }

    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// Character storage by ID
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

    /// Battle offers waiting for opponents
    #[pallet::storage]
    pub type BattleOffers<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::Hash,
        (T::AccountId, u64, BalanceOf<T>), // (creator, character_id, stake)
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
        /// Battle offer created [battle_id, creator, character_id, stake]
        BattleOfferCreated {
            battle_id: T::Hash,
            creator: T::AccountId,
            character_id: u64,
            stake: BalanceOf<T>,
        },
        /// Battle started [battle_id, player1, player2, stake]
        BattleStarted {
            battle_id: T::Hash,
            player1: T::AccountId,
            player2: T::AccountId,
            stake: BalanceOf<T>,
        },
        /// Turn executed [battle_id, player, damage_dealt, attacker_hp, defender_hp]
        TurnExecuted {
            battle_id: T::Hash,
            player: T::AccountId,
            damage_dealt: u64,
            attacker_hp: u64,
            defender_hp: u64,
        },
        /// Battle ended [battle_id, winner, total_payout]
        BattleEnded {
            battle_id: T::Hash,
            winner: Option<T::AccountId>,
            total_payout: BalanceOf<T>,
        },
        /// Character leveled up [character_id, new_level]
        CharacterLeveledUp {
            character_id: u64,
            new_level: u16,
        },
    }

    // ========== ERRORS ==========

    #[pallet::error]
    pub enum Error<T> {
        /// Character not found
        CharacterNotFound,
        /// Not the character owner
        NotCharacterOwner,
        /// Battle not found
        BattleNotFound,
        /// Not your turn
        NotYourTurn,
        /// Battle already finished
        BattleAlreadyFinished,
        /// Invalid battle state
        InvalidBattleState,
        /// Character already in battle
        CharacterInBattle,
        /// Insufficient stake
        InsufficientStake,
        /// Battle offer not found
        BattleOfferNotFound,
        /// Cannot battle own character
        CannotBattleSelf,
        /// Too many characters
        TooManyCharacters,
    }

    // ========== EXTRINSICS ==========

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new character
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn create_character(
            origin: OriginFor<T>,
            class: CharacterClass,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let character_id = NextCharacterId::<T>::get();
            let next_id = character_id.checked_add(1).ok_or(Error::<T>::TooManyCharacters)?;

            let (max_hp, damage_min, damage_max, crit, dodge, defense) = match class {
                CharacterClass::Warrior => (120, 8, 15, 1500, 500, 10),
                CharacterClass::Assassin => (90, 12, 20, 3500, 1500, 5),
                CharacterClass::Mage => (80, 10, 18, 2000, 800, 5),
                CharacterClass::Tank => (150, 6, 12, 1000, 300, 20),
                CharacterClass::Trickster => (100, 8, 16, 2500, 1200, 8),
            };

            let character = Character {
                owner: who.clone(),
                class: class.clone(),
                max_hp,
                current_hp: max_hp,
                base_damage_min: damage_min,
                base_damage_max: damage_max,
                crit_chance: crit,
                dodge_chance: dodge,
                defense,
                level: 1,
                xp: 0,
                wins: 0,
                losses: 0,
            };

            Characters::<T>::insert(character_id, character);
            OwnedCharacters::<T>::try_mutate(&who, |chars| {
                chars.try_push(character_id).map_err(|_| Error::<T>::TooManyCharacters)
            })?;
            NextCharacterId::<T>::put(next_id);

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
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let character = Characters::<T>::get(character_id)
                .ok_or(Error::<T>::CharacterNotFound)?;
            ensure!(character.owner == who, Error::<T>::NotCharacterOwner);

            // Generate battle ID
            let battle_id = T::Hashing::hash_of(&(&who, &character_id, &frame_system::Pallet::<T>::block_number()));

            // Store offer
            BattleOffers::<T>::insert(battle_id, (who.clone(), character_id, stake));

            Self::deposit_event(Event::BattleOfferCreated {
                battle_id,
                creator: who,
                character_id,
                stake,
            });

            Ok(())
        }

        /// Accept a battle offer
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn accept_battle(
            origin: OriginFor<T>,
            battle_id: T::Hash,
            character_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let (creator, creator_char_id, stake) = BattleOffers::<T>::get(battle_id)
                .ok_or(Error::<T>::BattleOfferNotFound)?;

            ensure!(who != creator, Error::<T>::CannotBattleSelf);

            let character = Characters::<T>::get(character_id)
                .ok_or(Error::<T>::CharacterNotFound)?;
            ensure!(character.owner == who, Error::<T>::NotCharacterOwner);

            let creator_char = Characters::<T>::get(creator_char_id)
                .ok_or(Error::<T>::CharacterNotFound)?;

            // Create battle
            let battle = Battle {
                battle_id,
                player1: creator.clone(),
                player2: who.clone(),
                character1_id: creator_char_id,
                character2_id: character_id,
                player1_health: creator_char.max_hp as u64,
                player2_health: character.max_hp as u64,
                current_turn: 1,
                turn_number: 0,
                state: BattleState::Active,
                player1_stance: StanceType::Balanced,
                player2_stance: StanceType::Balanced,
                stake_amount: stake,
                winner: None,
                created_at: frame_system::Pallet::<T>::block_number(),
                last_action: frame_system::Pallet::<T>::block_number(),
            };

            Battles::<T>::insert(battle_id, battle);
            BattleOffers::<T>::remove(battle_id);

            Self::deposit_event(Event::BattleStarted {
                battle_id,
                player1: creator,
                player2: who,
                stake,
            });

            Ok(())
        }

        /// Execute a turn in battle
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn execute_turn(
            origin: OriginFor<T>,
            battle_id: T::Hash,
            stance: StanceType,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Battles::<T>::try_mutate(battle_id, |maybe_battle| {
                let battle = maybe_battle.as_mut().ok_or(Error::<T>::BattleNotFound)?;

                ensure!(battle.state == BattleState::Active, Error::<T>::InvalidBattleState);

                let is_player1 = battle.player1 == who;
                let is_player2 = battle.player2 == who;
                ensure!(is_player1 || is_player2, Error::<T>::NotCharacterOwner);

                let correct_turn = (battle.current_turn == 1 && is_player1) ||
                                   (battle.current_turn == 2 && is_player2);
                ensure!(correct_turn, Error::<T>::NotYourTurn);

                // Get characters
                let attacker_id = if is_player1 { battle.character1_id } else { battle.character2_id };
                let defender_id = if is_player1 { battle.character2_id } else { battle.character1_id };

                let attacker = Characters::<T>::get(attacker_id).ok_or(Error::<T>::CharacterNotFound)?;
                let defender = Characters::<T>::get(defender_id).ok_or(Error::<T>::CharacterNotFound)?;

                // Update stance
                if is_player1 {
                    battle.player1_stance = stance.clone();
                } else {
                    battle.player2_stance = stance.clone();
                }

                // Calculate damage (simplified from Solana version)
                let base_damage = Self::calculate_damage(&attacker, &defender, &stance, battle.turn_number);
                let final_damage = base_damage.saturating_sub(defender.defense as u64);

                // Apply damage
                if is_player1 {
                    battle.player2_health = battle.player2_health.saturating_sub(final_damage);
                } else {
                    battle.player1_health = battle.player1_health.saturating_sub(final_damage);
                }

                let attacker_hp = if is_player1 { battle.player1_health } else { battle.player2_health };
                let defender_hp = if is_player1 { battle.player2_health } else { battle.player1_health };

                Self::deposit_event(Event::TurnExecuted {
                    battle_id,
                    player: who.clone(),
                    damage_dealt: final_damage,
                    attacker_hp,
                    defender_hp,
                });

                // Check for winner
                if battle.player1_health == 0 || battle.player2_health == 0 {
                    battle.state = BattleState::Finished;
                    battle.winner = if battle.player1_health > battle.player2_health {
                        Some(battle.player1.clone())
                    } else if battle.player2_health > battle.player1_health {
                        Some(battle.player2.clone())
                    } else {
                        None
                    };

                    // Award XP
                    if let Some(ref winner) = battle.winner {
                        let winner_char_id = if winner == &battle.player1 {
                            battle.character1_id
                        } else {
                            battle.character2_id
                        };
                        Self::award_xp(winner_char_id, 100)?;
                    }

                    Self::deposit_event(Event::BattleEnded {
                        battle_id,
                        winner: battle.winner.clone(),
                        total_payout: battle.stake_amount.saturating_mul(2u32.into()),
                    });
                } else {
                    // Switch turn
                    battle.current_turn = if battle.current_turn == 1 { 2 } else { 1 };
                    battle.turn_number = battle.turn_number.saturating_add(1);
                }

                battle.last_action = frame_system::Pallet::<T>::block_number();

                Ok(())
            })
        }
    }

    // ========== HELPER FUNCTIONS ==========

    impl<T: Config> Pallet<T> {
        /// Calculate damage based on character stats and stance
        fn calculate_damage(
            attacker: &Character<T>,
            _defender: &Character<T>,
            stance: &StanceType,
            turn_number: u64,
        ) -> u64 {
            let base_damage = (attacker.base_damage_min + attacker.base_damage_max) as u64 / 2;

            // Apply stance multiplier
            let multiplier = match stance {
                StanceType::Aggressive => 130,
                StanceType::Defensive => 70,
                StanceType::Berserker => 200,
                StanceType::Counter => 90,
                StanceType::Balanced => 100,
            };

            let damage = base_damage.saturating_mul(multiplier) / 100;

            // Add level bonus
            let level_bonus = (attacker.level as u64).saturating_mul(2);
            damage.saturating_add(level_bonus).saturating_add(turn_number % 5) // Add some randomness
        }

        /// Award XP to character and check for level up
        fn award_xp(character_id: u64, xp: u64) -> DispatchResult {
            Characters::<T>::try_mutate(character_id, |maybe_char| {
                let character = maybe_char.as_mut().ok_or(Error::<T>::CharacterNotFound)?;

                character.xp = character.xp.saturating_add(xp);

                // Level up check (simplified)
                let xp_needed = (character.level as u64).saturating_mul(100);
                if character.xp >= xp_needed {
                    character.level = character.level.saturating_add(1);
                    character.xp = 0;
                    character.max_hp = character.max_hp.saturating_add(character.max_hp / 20);
                    character.current_hp = character.max_hp;
                    character.base_damage_max = character.base_damage_max.saturating_add(1);

                    Self::deposit_event(Event::CharacterLeveledUp {
                        character_id,
                        new_level: character.level,
                    });
                }

                Ok(())
            })
        }
    }
}
