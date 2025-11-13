#![cfg_attr(not(feature = "std"), no_std)]

//! # Prediction Market Pallet
//!
//! Decentralized prediction markets for battle outcomes and gaming events.
//! Users can bet on battle winners, with AMM-style liquidity pooling.

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame::pallet]
pub mod pallet {
    use frame::prelude::*;
    use frame::traits::Currency;
    use frame::deps::frame_support;
    use sp_runtime::traits::{Hash, Zero, AccountIdConversion};
    use sp_runtime::{Saturating, Permill};

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_battlechain::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The pallet's module ID for holding funds
        #[pallet::constant]
        type PalletId: Get<frame_support::PalletId>;

        /// Platform fee in basis points (e.g., 200 = 2%)
        #[pallet::constant]
        type PlatformFeeBps: Get<u16>;
    }

    // ========== STORAGE ==========

    /// Market outcome type
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum MarketOutcome {
        Player1Wins,
        Player2Wins,
        Draw,
    }

    impl MarketOutcome {
        pub fn to_u8(&self) -> u8 {
            match self {
                MarketOutcome::Player1Wins => 0,
                MarketOutcome::Player2Wins => 1,
                MarketOutcome::Draw => 2,
            }
        }

        pub fn from_u8(value: u8) -> Result<Self, ()> {
            match value {
                0 => Ok(MarketOutcome::Player1Wins),
                1 => Ok(MarketOutcome::Player2Wins),
                2 => Ok(MarketOutcome::Draw),
                _ => Err(()),
            }
        }
    }

    /// Market state
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum MarketState {
        Open,
        Locked,
        Resolved,
        Cancelled,
    }

    impl MarketState {
        pub fn to_u8(&self) -> u8 {
            match self {
                MarketState::Open => 0,
                MarketState::Locked => 1,
                MarketState::Resolved => 2,
                MarketState::Cancelled => 3,
            }
        }

        pub fn from_u8(value: u8) -> Result<Self, ()> {
            match value {
                0 => Ok(MarketState::Open),
                1 => Ok(MarketState::Locked),
                2 => Ok(MarketState::Resolved),
                3 => Ok(MarketState::Cancelled),
                _ => Err(()),
            }
        }
    }

    /// Prediction market data
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct PredictionMarket<T: Config> {
        pub market_id: T::Hash,
        pub battle_id: T::Hash,
        pub creator: T::AccountId,
        pub state_id: u8,             // 0=Open, 1=Locked, 2=Resolved, 3=Cancelled
        pub total_pool: BalanceOf<T>,
        pub player1_pool: BalanceOf<T>,
        pub player2_pool: BalanceOf<T>,
        pub draw_pool: BalanceOf<T>,
        pub outcome_id: Option<u8>,   // 0=Player1Wins, 1=Player2Wins, 2=Draw
        pub created_at: BlockNumberFor<T>,
        pub resolve_block: Option<BlockNumberFor<T>>,
    }

    /// Individual prediction/bet
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Prediction<T: Config> {
        pub predictor: T::AccountId,
        pub market_id: T::Hash,
        pub outcome_id: u8,           // 0=Player1Wins, 1=Player2Wins, 2=Draw
        pub amount: BalanceOf<T>,
        pub claimed: bool,
    }

    type BalanceOf<T> = <<T as pallet_battlechain::Config>::Currency as frame::traits::Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// Active prediction markets
    #[pallet::storage]
    pub type Markets<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, PredictionMarket<T>>;

    /// User predictions
    #[pallet::storage]
    pub type Predictions<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::Hash, // market_id
        Blake2_128Concat,
        T::AccountId, // predictor
        Prediction<T>,
    >;

    /// Markets for a specific battle
    #[pallet::storage]
    pub type BattleMarkets<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::Hash, // battle_id
        T::Hash, // market_id
    >;

    // ========== EVENTS ==========

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Market created [market_id, battle_id, creator]
        MarketCreated {
            market_id: T::Hash,
            battle_id: T::Hash,
            creator: T::AccountId,
        },
        /// Prediction placed [market_id, predictor, outcome_id, amount]
        PredictionPlaced {
            market_id: T::Hash,
            predictor: T::AccountId,
            outcome_id: u8,
            amount: BalanceOf<T>,
        },
        /// Market locked [market_id]
        MarketLocked {
            market_id: T::Hash,
        },
        /// Market resolved [market_id, outcome_id, total_pool]
        MarketResolved {
            market_id: T::Hash,
            outcome_id: u8,
            total_pool: BalanceOf<T>,
        },
        /// Winnings claimed [market_id, predictor, payout]
        WinningsClaimed {
            market_id: T::Hash,
            predictor: T::AccountId,
            payout: BalanceOf<T>,
        },
    }

    // ========== ERRORS ==========

    #[pallet::error]
    pub enum Error<T> {
        /// Market not found
        MarketNotFound,
        /// Market not open for predictions
        MarketNotOpen,
        /// Market already exists for this battle
        MarketAlreadyExists,
        /// Battle not found
        BattleNotFound,
        /// Prediction not found
        PredictionNotFound,
        /// Already claimed
        AlreadyClaimed,
        /// Market not resolved yet
        MarketNotResolved,
        /// No winnings to claim
        NoWinnings,
        /// Insufficient balance
        InsufficientBalance,
        /// Battle not finished
        BattleNotFinished,
        /// Market not locked
        MarketNotLocked,
        /// Invalid outcome ID
        InvalidOutcome,
    }

    // ========== EXTRINSICS ==========

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a prediction market for a battle
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn create_market(
            origin: OriginFor<T>,
            battle_id: T::Hash,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Ensure battle exists and is active
            let battle = pallet_battlechain::Battles::<T>::get(battle_id)
                .ok_or(Error::<T>::BattleNotFound)?;
            ensure!(
                battle.state == pallet_battlechain::pallet::BattleState::Active,
                Error::<T>::BattleNotFinished
            );

            // Ensure market doesn't already exist
            ensure!(
                !BattleMarkets::<T>::contains_key(battle_id),
                Error::<T>::MarketAlreadyExists
            );

            // Generate market ID
            let market_id = T::Hashing::hash_of(&(&battle_id, &who, &frame_system::Pallet::<T>::block_number()));

            let market = PredictionMarket {
                market_id,
                battle_id,
                creator: who.clone(),
                state_id: MarketState::Open.to_u8(),
                total_pool: Zero::zero(),
                player1_pool: Zero::zero(),
                player2_pool: Zero::zero(),
                draw_pool: Zero::zero(),
                outcome_id: None,
                created_at: frame_system::Pallet::<T>::block_number(),
                resolve_block: None,
            };

            Markets::<T>::insert(market_id, market);
            BattleMarkets::<T>::insert(battle_id, market_id);

            Self::deposit_event(Event::MarketCreated {
                market_id,
                battle_id,
                creator: who,
            });

            Ok(())
        }

        /// Place a prediction on a market
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn place_prediction(
            origin: OriginFor<T>,
            market_id: T::Hash,
            outcome_id: u8,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Validate outcome_id
            let outcome = MarketOutcome::from_u8(outcome_id)
                .map_err(|_| Error::<T>::InvalidOutcome)?;

            Markets::<T>::try_mutate(market_id, |maybe_market| {
                let market = maybe_market.as_mut().ok_or(Error::<T>::MarketNotFound)?;

                let market_state = MarketState::from_u8(market.state_id)
                    .map_err(|_| Error::<T>::MarketNotFound)?;
                ensure!(market_state == MarketState::Open, Error::<T>::MarketNotOpen);

                // Transfer funds to pallet account
                T::Currency::transfer(
                    &who,
                    &Self::account_id(),
                    amount,
                    frame_support::traits::ExistenceRequirement::KeepAlive,
                )?;

                // Update pools
                market.total_pool = market.total_pool.saturating_add(amount);
                match outcome {
                    MarketOutcome::Player1Wins => {
                        market.player1_pool = market.player1_pool.saturating_add(amount);
                    }
                    MarketOutcome::Player2Wins => {
                        market.player2_pool = market.player2_pool.saturating_add(amount);
                    }
                    MarketOutcome::Draw => {
                        market.draw_pool = market.draw_pool.saturating_add(amount);
                    }
                }

                // Store or update prediction
                Predictions::<T>::insert(
                    market_id,
                    &who,
                    Prediction {
                        predictor: who.clone(),
                        market_id,
                        outcome_id,
                        amount,
                        claimed: false,
                    },
                );

                Self::deposit_event(Event::PredictionPlaced {
                    market_id,
                    predictor: who,
                    outcome_id,
                    amount,
                });

                Ok(())
            })
        }

        /// Lock market when battle starts (no more bets)
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn lock_market(
            origin: OriginFor<T>,
            market_id: T::Hash,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            Markets::<T>::try_mutate(market_id, |maybe_market| {
                let market = maybe_market.as_mut().ok_or(Error::<T>::MarketNotFound)?;

                let market_state = MarketState::from_u8(market.state_id)
                    .map_err(|_| Error::<T>::MarketNotFound)?;
                ensure!(market_state == MarketState::Open, Error::<T>::MarketNotOpen);

                market.state_id = MarketState::Locked.to_u8();

                Self::deposit_event(Event::MarketLocked { market_id });

                Ok(())
            })
        }

        /// Resolve market based on battle outcome
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn resolve_market(
            origin: OriginFor<T>,
            market_id: T::Hash,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            Markets::<T>::try_mutate(market_id, |maybe_market| {
                let market = maybe_market.as_mut().ok_or(Error::<T>::MarketNotFound)?;

                let market_state = MarketState::from_u8(market.state_id)
                    .map_err(|_| Error::<T>::MarketNotFound)?;
                ensure!(market_state == MarketState::Locked, Error::<T>::MarketNotLocked);

                // Get battle outcome
                let battle = pallet_battlechain::Battles::<T>::get(market.battle_id)
                    .ok_or(Error::<T>::BattleNotFound)?;
                ensure!(
                    battle.state == pallet_battlechain::pallet::BattleState::Finished,
                    Error::<T>::BattleNotFinished
                );

                // Determine outcome based on rounds won
                let outcome = if battle.player1_rounds_won > battle.player2_rounds_won {
                    MarketOutcome::Player1Wins
                } else if battle.player2_rounds_won > battle.player1_rounds_won {
                    MarketOutcome::Player2Wins
                } else {
                    MarketOutcome::Draw
                };

                market.outcome_id = Some(outcome.to_u8());
                market.state_id = MarketState::Resolved.to_u8();
                market.resolve_block = Some(frame_system::Pallet::<T>::block_number());

                Self::deposit_event(Event::MarketResolved {
                    market_id,
                    outcome_id: outcome.to_u8(),
                    total_pool: market.total_pool,
                });

                Ok(())
            })
        }

        /// Claim winnings from resolved market
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn claim_winnings(
            origin: OriginFor<T>,
            market_id: T::Hash,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let market = Markets::<T>::get(market_id).ok_or(Error::<T>::MarketNotFound)?;

            let market_state = MarketState::from_u8(market.state_id)
                .map_err(|_| Error::<T>::MarketNotFound)?;
            ensure!(market_state == MarketState::Resolved, Error::<T>::MarketNotResolved);

            Predictions::<T>::try_mutate(market_id, &who, |maybe_prediction| {
                let prediction = maybe_prediction.as_mut().ok_or(Error::<T>::PredictionNotFound)?;
                ensure!(!prediction.claimed, Error::<T>::AlreadyClaimed);

                // Check if prediction was correct
                let outcome_id = market.outcome_id.ok_or(Error::<T>::MarketNotResolved)?;
                ensure!(prediction.outcome_id == outcome_id, Error::<T>::NoWinnings);

                let outcome = MarketOutcome::from_u8(outcome_id)
                    .map_err(|_| Error::<T>::InvalidOutcome)?;

                // Calculate payout
                let winning_pool = match outcome {
                    MarketOutcome::Player1Wins => market.player1_pool,
                    MarketOutcome::Player2Wins => market.player2_pool,
                    MarketOutcome::Draw => market.draw_pool,
                };

                ensure!(winning_pool > Zero::zero(), Error::<T>::NoWinnings);

                // Payout = (user_bet / winning_pool) * total_pool * (1 - fee)
                let fee_bps = T::PlatformFeeBps::get() as u128;
                let total_after_fee = market.total_pool.saturating_sub(
                    Permill::from_rational(fee_bps, 10000u128).mul_floor(market.total_pool)
                );

                // Simple proportion calculation
                let user_share = prediction.amount.saturating_mul(total_after_fee.into()) / winning_pool;

                // Transfer winnings
                T::Currency::transfer(
                    &Self::account_id(),
                    &who,
                    user_share,
                    frame_support::traits::ExistenceRequirement::AllowDeath,
                )?;

                prediction.claimed = true;

                Self::deposit_event(Event::WinningsClaimed {
                    market_id,
                    predictor: who.clone(),
                    payout: user_share,
                });

                Ok(())
            })
        }
    }

    // ========== HELPER FUNCTIONS ==========

    impl<T: Config> Pallet<T> {
        /// Get the account ID of the pallet for holding funds
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }
    }
}
