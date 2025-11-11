#![cfg_attr(not(feature = "std"), no_std)]

//! # Game Oracle Pallet
//!
//! On-chain oracle system for game developers to submit verified game results
//! and earn revenue when other applications consume their data.

pub use pallet::*;

#[frame::pallet(dev_mode)]
pub mod pallet {
    use frame::prelude::*;
    use sp_runtime::traits::{Hash, Zero, AccountIdConversion};
    use sp_runtime::Saturating;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_battlechain::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The pallet's module ID for revenue sharing
        #[pallet::constant]
        type PalletId: Get<frame_support::PalletId>;

        /// Revenue share for data providers (in basis points, e.g., 7000 = 70%)
        #[pallet::constant]
        type ProviderRevShareBps: Get<u16>;

        /// Fee for querying oracle data
        #[pallet::constant]
        type QueryFee: Get<BalanceOf<Self>>;
    }

    // ========== STORAGE ==========

    /// Game developer registration status
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum DeveloperStatus {
        Pending,
        Verified,
        Suspended,
    }

    /// Game result status
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum ResultStatus {
        Submitted,
        Verified,
        Disputed,
        Rejected,
    }

    /// Game developer profile
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct DeveloperProfile<T: Config> {
        pub developer: T::AccountId,
        pub game_name: BoundedVec<u8, ConstU32<64>>,
        pub status: DeveloperStatus,
        pub reputation_score: u32, // 0-10000 basis points
        pub total_submissions: u64,
        pub total_revenue: BalanceOf<T>,
        pub registered_at: BlockNumberFor<T>,
    }

    /// Game result data
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct GameResult<T: Config> {
        pub result_id: T::Hash,
        pub developer: T::AccountId,
        pub battle_id: T::Hash,
        pub winner: Option<T::AccountId>,
        pub player1_score: u64,
        pub player2_score: u64,
        pub data_hash: T::Hash, // Hash of detailed game data
        pub status: ResultStatus,
        pub query_count: u64,
        pub revenue_earned: BalanceOf<T>,
        pub submitted_at: BlockNumberFor<T>,
    }

    /// Dispute record
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Dispute<T: Config> {
        pub result_id: T::Hash,
        pub challenger: T::AccountId,
        pub stake: BalanceOf<T>,
        pub reason: BoundedVec<u8, ConstU32<256>>,
        pub resolved: bool,
        pub challenge_won: bool,
    }

    type BalanceOf<T> = <<T as pallet_battlechain::Config>::Currency as frame::traits::Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// Registered game developers
    #[pallet::storage]
    pub type Developers<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, DeveloperProfile<T>>;

    /// Game results submitted by developers
    #[pallet::storage]
    pub type Results<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, GameResult<T>>;

    /// Results for a specific battle
    #[pallet::storage]
    pub type BattleResults<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, T::Hash>;

    /// Active disputes
    #[pallet::storage]
    pub type Disputes<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, Dispute<T>>;

    /// Pending revenue for developers to claim
    #[pallet::storage]
    pub type PendingRevenue<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

    // ========== EVENTS ==========

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Developer registered [developer, game_name]
        DeveloperRegistered {
            developer: T::AccountId,
            game_name: BoundedVec<u8, ConstU32<64>>,
        },
        /// Developer verified [developer]
        DeveloperVerified {
            developer: T::AccountId,
        },
        /// Game result submitted [result_id, developer, battle_id]
        ResultSubmitted {
            result_id: T::Hash,
            developer: T::AccountId,
            battle_id: T::Hash,
        },
        /// Game result verified [result_id]
        ResultVerified {
            result_id: T::Hash,
        },
        /// Data queried [result_id, querier, fee_paid, provider_revenue]
        DataQueried {
            result_id: T::Hash,
            querier: T::AccountId,
            fee_paid: BalanceOf<T>,
            provider_revenue: BalanceOf<T>,
        },
        /// Dispute created [result_id, challenger, stake]
        DisputeCreated {
            result_id: T::Hash,
            challenger: T::AccountId,
            stake: BalanceOf<T>,
        },
        /// Dispute resolved [result_id, challenger_won]
        DisputeResolved {
            result_id: T::Hash,
            challenger_won: bool,
        },
        /// Revenue claimed [developer, amount]
        RevenueClaimed {
            developer: T::AccountId,
            amount: BalanceOf<T>,
        },
    }

    // ========== ERRORS ==========

    #[pallet::error]
    pub enum Error<T> {
        /// Developer already registered
        DeveloperAlreadyRegistered,
        /// Developer not found
        DeveloperNotFound,
        /// Developer not verified
        DeveloperNotVerified,
        /// Result not found
        ResultNotFound,
        /// Result already exists for this battle
        ResultAlreadyExists,
        /// Battle not found
        BattleNotFound,
        /// Insufficient query fee
        InsufficientQueryFee,
        /// Dispute already exists
        DisputeAlreadyExists,
        /// Dispute not found
        DisputeNotFound,
        /// No revenue to claim
        NoRevenueToClaim,
        /// Invalid game name
        InvalidGameName,
    }

    // ========== EXTRINSICS ==========

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register as a game developer
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn register_developer(
            origin: OriginFor<T>,
            game_name: BoundedVec<u8, ConstU32<64>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                !Developers::<T>::contains_key(&who),
                Error::<T>::DeveloperAlreadyRegistered
            );
            ensure!(!game_name.is_empty(), Error::<T>::InvalidGameName);

            let profile = DeveloperProfile {
                developer: who.clone(),
                game_name: game_name.clone(),
                status: DeveloperStatus::Pending,
                reputation_score: 5000, // Start at 50%
                total_submissions: 0,
                total_revenue: Zero::zero(),
                registered_at: frame_system::Pallet::<T>::block_number(),
            };

            Developers::<T>::insert(&who, profile);

            Self::deposit_event(Event::DeveloperRegistered {
                developer: who,
                game_name,
            });

            Ok(())
        }

        /// Verify a developer (requires governance or authority)
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn verify_developer(
            origin: OriginFor<T>,
            developer: T::AccountId,
        ) -> DispatchResult {
            ensure_root(origin)?;

            Developers::<T>::try_mutate(&developer, |maybe_profile| {
                let profile = maybe_profile.as_mut().ok_or(Error::<T>::DeveloperNotFound)?;
                profile.status = DeveloperStatus::Verified;

                Self::deposit_event(Event::DeveloperVerified {
                    developer: developer.clone(),
                });

                Ok(())
            })
        }

        /// Submit game result
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn submit_result(
            origin: OriginFor<T>,
            battle_id: T::Hash,
            winner: Option<T::AccountId>,
            player1_score: u64,
            player2_score: u64,
            data_hash: T::Hash,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let profile = Developers::<T>::get(&who).ok_or(Error::<T>::DeveloperNotFound)?;
            ensure!(
                profile.status == DeveloperStatus::Verified,
                Error::<T>::DeveloperNotVerified
            );

            // Ensure battle exists
            let battle = pallet_battlechain::Battles::<T>::get(battle_id)
                .ok_or(Error::<T>::BattleNotFound)?;

            // Ensure result doesn't already exist
            ensure!(
                !BattleResults::<T>::contains_key(battle_id),
                Error::<T>::ResultAlreadyExists
            );

            // Generate result ID
            let result_id = T::Hashing::hash_of(&(&battle_id, &who, &frame_system::Pallet::<T>::block_number()));

            let result = GameResult {
                result_id,
                developer: who.clone(),
                battle_id,
                winner: winner.clone(),
                player1_score,
                player2_score,
                data_hash,
                status: ResultStatus::Submitted,
                query_count: 0,
                revenue_earned: Zero::zero(),
                submitted_at: frame_system::Pallet::<T>::block_number(),
            };

            Results::<T>::insert(result_id, result);
            BattleResults::<T>::insert(battle_id, result_id);

            // Update developer stats
            Developers::<T>::try_mutate(&who, |maybe_profile| {
                if let Some(profile) = maybe_profile {
                    profile.total_submissions = profile.total_submissions.saturating_add(1);
                }
                Ok::<(), DispatchError>(())
            })?;

            Self::deposit_event(Event::ResultSubmitted {
                result_id,
                developer: who,
                battle_id,
            });

            Ok(())
        }

        /// Query game result data (pays fee to developer)
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn query_result(
            origin: OriginFor<T>,
            result_id: T::Hash,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let query_fee = T::QueryFee::get();

            Results::<T>::try_mutate(result_id, |maybe_result| {
                let result = maybe_result.as_mut().ok_or(Error::<T>::ResultNotFound)?;

                // Calculate revenue split
                let provider_share_bps = T::ProviderRevShareBps::get() as u128;
                let provider_revenue = sp_runtime::Permill::from_rational(provider_share_bps, 10000u128)
                    .mul_floor(query_fee);

                // Transfer query fee
                <pallet_battlechain::pallet::Config as pallet_battlechain::Config>::Currency::transfer(
                    &who,
                    &Self::account_id(),
                    query_fee,
                    frame_support::traits::ExistenceRequirement::KeepAlive,
                )?;

                // Update pending revenue for developer
                PendingRevenue::<T>::mutate(&result.developer, |balance| {
                    *balance = balance.saturating_add(provider_revenue);
                });

                // Update result stats
                result.query_count = result.query_count.saturating_add(1);
                result.revenue_earned = result.revenue_earned.saturating_add(provider_revenue);

                Self::deposit_event(Event::DataQueried {
                    result_id,
                    querier: who,
                    fee_paid: query_fee,
                    provider_revenue,
                });

                Ok(())
            })
        }

        /// Create a dispute for a result
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn create_dispute(
            origin: OriginFor<T>,
            result_id: T::Hash,
            stake: BalanceOf<T>,
            reason: BoundedVec<u8, ConstU32<256>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                !Disputes::<T>::contains_key(result_id),
                Error::<T>::DisputeAlreadyExists
            );
            let result = Results::<T>::get(result_id).ok_or(Error::<T>::ResultNotFound)?;

            // Lock stake
            <pallet_battlechain::pallet::Config as pallet_battlechain::Config>::Currency::transfer(
                &who,
                &Self::account_id(),
                stake,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )?;

            let dispute = Dispute {
                result_id,
                challenger: who.clone(),
                stake,
                reason,
                resolved: false,
                challenge_won: false,
            };

            Disputes::<T>::insert(result_id, dispute);

            // Mark result as disputed
            Results::<T>::try_mutate(result_id, |maybe_result| {
                if let Some(result) = maybe_result {
                    result.status = ResultStatus::Disputed;
                }
                Ok::<(), DispatchError>(())
            })?;

            Self::deposit_event(Event::DisputeCreated {
                result_id,
                challenger: who,
                stake,
            });

            Ok(())
        }

        /// Claim pending revenue
        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn claim_revenue(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let amount = PendingRevenue::<T>::take(&who);
            ensure!(amount > Zero::zero(), Error::<T>::NoRevenueToClaim);

            // Transfer revenue
            <pallet_battlechain::pallet::Config as pallet_battlechain::Config>::Currency::transfer(
                &Self::account_id(),
                &who,
                amount,
                frame_support::traits::ExistenceRequirement::AllowDeath,
            )?;

            // Update developer total revenue
            Developers::<T>::try_mutate(&who, |maybe_profile| {
                if let Some(profile) = maybe_profile {
                    profile.total_revenue = profile.total_revenue.saturating_add(amount);
                }
                Ok::<(), DispatchError>(())
            })?;

            Self::deposit_event(Event::RevenueClaimed {
                developer: who,
                amount,
            });

            Ok(())
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
