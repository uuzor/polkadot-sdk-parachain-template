#![cfg_attr(not(feature = "std"), no_std)]

//! # Game Oracle Pallet - Enhanced with Staking & Slashing
//!
//! **Trust Model**: Staking + Slashing with dispute period
//! **Result Format**: Standardized schema with extension field
//!
//! ## Architecture:
//! - Developers must stake tokens to register
//! - Additional stake required per result submission
//! - 7-day dispute period for challenges
//! - Slashing mechanism for fraudulent results
//! - Revenue sharing: 70% to developer, 30% to protocol

pub use pallet::*;

#[frame::pallet]
pub mod pallet {
    use frame::prelude::*;
    use frame::traits::Currency;
    use frame::deps::frame_support;
    use sp_runtime::traits::{Hash, Zero, AccountIdConversion, Saturating};
    use sp_runtime::Perbill;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_battlechain::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The pallet's module ID for holding funds
        #[pallet::constant]
        type PalletId: Get<frame_support::PalletId>;

        /// Revenue share for data providers (basis points, 7000 = 70%)
        #[pallet::constant]
        type ProviderRevShareBps: Get<u16>;

        /// Fee for querying oracle data
        #[pallet::constant]
        type QueryFee: Get<BalanceOf<Self>>;

        /// Required stake for developer registration
        #[pallet::constant]
        type DeveloperStake: Get<BalanceOf<Self>>;

        /// Required stake per result submission
        #[pallet::constant]
        type ResultStake: Get<BalanceOf<Self>>;

        /// Dispute period in blocks (7 days ≈ 100,800 blocks)
        #[pallet::constant]
        type DisputePeriod: Get<BlockNumberFor<Self>>;

        /// Dispute challenge stake (must be >= ResultStake)
        #[pallet::constant]
        type DisputeStake: Get<BalanceOf<Self>>;
    }

    // ========== TYPES ==========

    /// Developer registration status
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum DeveloperStatus {
        Pending,        // Awaiting verification
        Verified,       // Active and can submit results
        Suspended,      // Temporarily disabled
        Slashed,        // Penalized for fraud
    }

    impl DeveloperStatus {
        pub fn to_u8(&self) -> u8 {
            match self {
                DeveloperStatus::Pending => 0,
                DeveloperStatus::Verified => 1,
                DeveloperStatus::Suspended => 2,
                DeveloperStatus::Slashed => 3,
            }
        }

        pub fn from_u8(value: u8) -> Result<Self, ()> {
            match value {
                0 => Ok(DeveloperStatus::Pending),
                1 => Ok(DeveloperStatus::Verified),
                2 => Ok(DeveloperStatus::Suspended),
                3 => Ok(DeveloperStatus::Slashed),
                _ => Err(()),
            }
        }
    }

    /// Result verification status
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum ResultStatus {
        Submitted,      // Waiting for dispute period
        Verified,       // Dispute period passed, data is trusted
        Disputed,       // Under challenge
        Slashed,        // Challenge won, developer penalized
    }

    impl ResultStatus {
        pub fn to_u8(&self) -> u8 {
            match self {
                ResultStatus::Submitted => 0,
                ResultStatus::Verified => 1,
                ResultStatus::Disputed => 2,
                ResultStatus::Slashed => 3,
            }
        }

        pub fn from_u8(value: u8) -> Result<Self, ()> {
            match value {
                0 => Ok(ResultStatus::Submitted),
                1 => Ok(ResultStatus::Verified),
                2 => Ok(ResultStatus::Disputed),
                3 => Ok(ResultStatus::Slashed),
                _ => Err(()),
            }
        }
    }

    /// Dispute outcome
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum DisputeOutcome {
        Pending,
        DeveloperWins,  // Developer keeps stake, challenger loses
        ChallengerWins, // Developer slashed, challenger gets reward
    }

    impl DisputeOutcome {
        pub fn to_u8(&self) -> u8 {
            match self {
                DisputeOutcome::Pending => 0,
                DisputeOutcome::DeveloperWins => 1,
                DisputeOutcome::ChallengerWins => 2,
            }
        }

        pub fn from_u8(value: u8) -> Result<Self, ()> {
            match value {
                0 => Ok(DisputeOutcome::Pending),
                1 => Ok(DisputeOutcome::DeveloperWins),
                2 => Ok(DisputeOutcome::ChallengerWins),
                _ => Err(()),
            }
        }
    }

    /// Standardized game result format
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct StandardizedResult<T: Config> {
        // Core fields (standardized)
        pub battle_id: T::Hash,
        pub winner: Option<T::AccountId>,
        pub player1: T::AccountId,
        pub player2: T::AccountId,
        pub player1_score: u32,
        pub player2_score: u32,
        pub rounds_played: u8,
        pub timestamp: BlockNumberFor<T>,

        // Metadata
        pub game_name: BoundedVec<u8, ConstU32<32>>,
        pub game_version: u32,

        // Extension field for custom data
        pub extension_data: BoundedVec<u8, ConstU32<256>>,
    }

    /// Developer profile with staking
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct DeveloperProfile<T: Config> {
        pub developer: T::AccountId,
        pub game_name: BoundedVec<u8, ConstU32<64>>,
        pub status: DeveloperStatus,
        pub reputation_score: u32,          // 0-10000 basis points
        pub staked_amount: BalanceOf<T>,    // Total stake locked
        pub total_submissions: u64,
        pub successful_disputes: u32,        // Times they were right in disputes
        pub failed_disputes: u32,            // Times they were wrong
        pub total_revenue: BalanceOf<T>,
        pub registered_at: BlockNumberFor<T>,
    }

    /// Game result with staking
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct GameResult<T: Config> {
        pub result_id: T::Hash,
        pub developer: T::AccountId,
        pub result_data: StandardizedResult<T>,
        pub data_hash: T::Hash,              // Hash for integrity verification
        pub stake_amount: BalanceOf<T>,      // Stake locked for this result
        pub status: ResultStatus,
        pub query_count: u64,
        pub revenue_earned: BalanceOf<T>,
        pub submitted_at: BlockNumberFor<T>,
        pub dispute_deadline: BlockNumberFor<T>, // When dispute period ends
    }

    /// Dispute record with staking
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Dispute<T: Config> {
        pub dispute_id: T::Hash,
        pub result_id: T::Hash,
        pub challenger: T::AccountId,
        pub challenger_stake: BalanceOf<T>,  // Stake from challenger
        pub reason: BoundedVec<u8, ConstU32<512>>,
        pub evidence_hash: T::Hash,          // Hash of evidence data
        pub outcome: DisputeOutcome,
        pub resolved_at: Option<BlockNumberFor<T>>,
        pub created_at: BlockNumberFor<T>,
    }

    type BalanceOf<T> = <<T as pallet_battlechain::Config>::Currency as frame::traits::Currency<<T as frame_system::Config>::AccountId>>::Balance;

    // ========== STORAGE ==========

    /// Registered game developers
    #[pallet::storage]
    pub type Developers<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, DeveloperProfile<T>>;

    /// Game results by result ID
    #[pallet::storage]
    pub type Results<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, GameResult<T>>;

    /// Results for a specific battle
    #[pallet::storage]
    pub type BattleResults<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, T::Hash>;

    /// Active disputes
    #[pallet::storage]
    pub type Disputes<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, Dispute<T>>;

    /// Pending revenue for developers
    #[pallet::storage]
    pub type PendingRevenue<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

    /// Total staked by developers
    #[pallet::storage]
    pub type TotalStaked<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    // ========== EVENTS ==========

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Developer registered [developer, game_name, stake]
        DeveloperRegistered {
            developer: T::AccountId,
            game_name: BoundedVec<u8, ConstU32<64>>,
            stake: BalanceOf<T>,
        },
        /// Developer verified by governance [developer]
        DeveloperVerified {
            developer: T::AccountId,
        },
        /// Result submitted [result_id, developer, battle_id, stake]
        ResultSubmitted {
            result_id: T::Hash,
            developer: T::AccountId,
            battle_id: T::Hash,
            stake: BalanceOf<T>,
        },
        /// Result verified after dispute period [result_id]
        ResultVerified {
            result_id: T::Hash,
        },
        /// Data queried [result_id, querier, fee, developer_revenue]
        DataQueried {
            result_id: T::Hash,
            querier: T::AccountId,
            fee: BalanceOf<T>,
            developer_revenue: BalanceOf<T>,
        },
        /// Dispute created [dispute_id, result_id, challenger, stake]
        DisputeCreated {
            dispute_id: T::Hash,
            result_id: T::Hash,
            challenger: T::AccountId,
            stake: BalanceOf<T>,
        },
        /// Dispute resolved [dispute_id, outcome_id, slashed_amount]
        DisputeResolved {
            dispute_id: T::Hash,
            outcome_id: u8,
            slashed_amount: BalanceOf<T>,
        },
        /// Developer slashed [developer, result_id, amount]
        DeveloperSlashed {
            developer: T::AccountId,
            result_id: T::Hash,
            amount: BalanceOf<T>,
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
        DeveloperAlreadyRegistered,
        DeveloperNotFound,
        DeveloperNotVerified,
        DeveloperSlashed,
        ResultNotFound,
        ResultAlreadyExists,
        BattleNotFound,
        InsufficientStake,
        DisputeAlreadyExists,
        DisputeNotFound,
        DisputePeriodNotEnded,
        DisputePeriodEnded,
        NoRevenueToClaim,
        InvalidGameName,
        ResultStillDisputed,
        UnauthorizedResolver,
    }

    // ========== EXTRINSICS ==========

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register as a game developer (requires staking)
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

            let stake = T::DeveloperStake::get();

            // Lock developer stake
            T::Currency::transfer(
                &who,
                &Self::account_id(),
                stake,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )?;

            let profile = DeveloperProfile {
                developer: who.clone(),
                game_name: game_name.clone(),
                status: DeveloperStatus::Pending,
                reputation_score: 5000, // Start at 50%
                staked_amount: stake,
                total_submissions: 0,
                successful_disputes: 0,
                failed_disputes: 0,
                total_revenue: Zero::zero(),
                registered_at: frame_system::Pallet::<T>::block_number(),
            };

            Developers::<T>::insert(&who, profile);
            TotalStaked::<T>::mutate(|total| *total = total.saturating_add(stake));

            Self::deposit_event(Event::DeveloperRegistered {
                developer: who,
                game_name,
                stake,
            });

            Ok(())
        }

        /// Verify a developer (governance only)
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

        /// Submit standardized game result (requires additional stake)
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn submit_result(
            origin: OriginFor<T>,
            result_data: StandardizedResult<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let profile = Developers::<T>::get(&who).ok_or(Error::<T>::DeveloperNotFound)?;
            ensure!(
                profile.status == DeveloperStatus::Verified,
                Error::<T>::DeveloperNotVerified
            );

            // Ensure battle exists
            let _battle = pallet_battlechain::Battles::<T>::get(result_data.battle_id)
                .ok_or(Error::<T>::BattleNotFound)?;

            ensure!(
                !BattleResults::<T>::contains_key(result_data.battle_id),
                Error::<T>::ResultAlreadyExists
            );

            let result_stake = T::ResultStake::get();

            // Lock result stake
            T::Currency::transfer(
                &who,
                &Self::account_id(),
                result_stake,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )?;

            let result_id = T::Hashing::hash_of(&(&result_data.battle_id, &who, &frame_system::Pallet::<T>::block_number()));
            let data_hash = T::Hashing::hash_of(&result_data);
            let now = frame_system::Pallet::<T>::block_number();
            let dispute_deadline = now.saturating_add(T::DisputePeriod::get());

            let battle_id = result_data.battle_id;

            let result = GameResult {
                result_id,
                developer: who.clone(),
                result_data,
                data_hash,
                stake_amount: result_stake,
                status: ResultStatus::Submitted,
                query_count: 0,
                revenue_earned: Zero::zero(),
                submitted_at: now,
                dispute_deadline,
            };

            Results::<T>::insert(result_id, result);
            BattleResults::<T>::insert(battle_id, result_id);

            Developers::<T>::try_mutate(&who, |maybe_profile| {
                if let Some(profile) = maybe_profile {
                    profile.total_submissions = profile.total_submissions.saturating_add(1);
                    profile.staked_amount = profile.staked_amount.saturating_add(result_stake);
                }
                Ok::<(), DispatchError>(())
            })?;

            TotalStaked::<T>::mutate(|total| *total = total.saturating_add(result_stake));

            Self::deposit_event(Event::ResultSubmitted {
                result_id,
                developer: who,
                battle_id,
                stake: result_stake,
            });

            Ok(())
        }

        /// Finalize result after dispute period
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn finalize_result(
            origin: OriginFor<T>,
            result_id: T::Hash,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            Results::<T>::try_mutate(result_id, |maybe_result| {
                let result = maybe_result.as_mut().ok_or(Error::<T>::ResultNotFound)?;

                ensure!(
                    result.status == ResultStatus::Submitted,
                    Error::<T>::ResultStillDisputed
                );

                let now = frame_system::Pallet::<T>::block_number();
                ensure!(
                    now >= result.dispute_deadline,
                    Error::<T>::DisputePeriodNotEnded
                );

                result.status = ResultStatus::Verified;

                // Return stake to developer
                let developer = result.developer.clone();
                let stake = result.stake_amount;

                T::Currency::transfer(
                    &Self::account_id(),
                    &developer,
                    stake,
                    frame_support::traits::ExistenceRequirement::AllowDeath,
                )?;

                Developers::<T>::try_mutate(&developer, |maybe_profile| {
                    if let Some(profile) = maybe_profile {
                        profile.staked_amount = profile.staked_amount.saturating_sub(stake);
                    }
                    Ok::<(), DispatchError>(())
                })?;

                TotalStaked::<T>::mutate(|total| *total = total.saturating_sub(stake));

                Self::deposit_event(Event::ResultVerified { result_id });

                Ok(())
            })
        }

        /// Query game result data (pays fee)
        #[pallet::call_index(4)]
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
                let provider_revenue = Perbill::from_rational(provider_share_bps, 10000u128)
                    .mul_floor(query_fee);

                // Transfer query fee
                T::Currency::transfer(
                    &who,
                    &Self::account_id(),
                    query_fee,
                    frame_support::traits::ExistenceRequirement::KeepAlive,
                )?;

                // Update pending revenue for developer
                PendingRevenue::<T>::mutate(&result.developer, |balance| {
                    *balance = balance.saturating_add(provider_revenue);
                });

                result.query_count = result.query_count.saturating_add(1);
                result.revenue_earned = result.revenue_earned.saturating_add(provider_revenue);

                Self::deposit_event(Event::DataQueried {
                    result_id,
                    querier: who,
                    fee: query_fee,
                    developer_revenue: provider_revenue,
                });

                Ok(())
            })
        }

        /// Create dispute (requires stake)
        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn create_dispute(
            origin: OriginFor<T>,
            result_id: T::Hash,
            reason: BoundedVec<u8, ConstU32<512>>,
            evidence_hash: T::Hash,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let result = Results::<T>::get(result_id).ok_or(Error::<T>::ResultNotFound)?;

            let now = frame_system::Pallet::<T>::block_number();
            ensure!(
                now < result.dispute_deadline,
                Error::<T>::DisputePeriodEnded
            );

            let dispute_stake = T::DisputeStake::get();

            // Lock challenger stake
            T::Currency::transfer(
                &who,
                &Self::account_id(),
                dispute_stake,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )?;

            let dispute_id = T::Hashing::hash_of(&(&result_id, &who, &now));

            let dispute = Dispute {
                dispute_id,
                result_id,
                challenger: who.clone(),
                challenger_stake: dispute_stake,
                reason,
                evidence_hash,
                outcome: DisputeOutcome::Pending,
                resolved_at: None,
                created_at: now,
            };

            Disputes::<T>::insert(dispute_id, dispute);

            // Mark result as disputed
            Results::<T>::try_mutate(result_id, |maybe_result| {
                if let Some(result) = maybe_result {
                    result.status = ResultStatus::Disputed;
                }
                Ok::<(), DispatchError>(())
            })?;

            Self::deposit_event(Event::DisputeCreated {
                dispute_id,
                result_id,
                challenger: who,
                stake: dispute_stake,
            });

            Ok(())
        }

        /// Resolve dispute (governance only)
        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn resolve_dispute(
            origin: OriginFor<T>,
            dispute_id: T::Hash,
            challenger_wins: bool,
        ) -> DispatchResult {
            ensure_root(origin)?;

            Disputes::<T>::try_mutate(dispute_id, |maybe_dispute| {
                let dispute = maybe_dispute.as_mut().ok_or(Error::<T>::DisputeNotFound)?;

                let outcome = if challenger_wins {
                    DisputeOutcome::ChallengerWins
                } else {
                    DisputeOutcome::DeveloperWins
                };

                dispute.outcome = outcome.clone();
                dispute.resolved_at = Some(frame_system::Pallet::<T>::block_number());

                let result_id = dispute.result_id;
                let challenger = dispute.challenger.clone();
                let challenger_stake = dispute.challenger_stake;

                // Get result and developer
                let result = Results::<T>::get(result_id).ok_or(Error::<T>::ResultNotFound)?;
                let developer = result.developer.clone();
                let result_stake = result.stake_amount;

                if challenger_wins {
                    // Slash developer, reward challenger
                    let total_pot = result_stake.saturating_add(challenger_stake);

                    // Challenger gets their stake back + developer's stake
                    T::Currency::transfer(
                        &Self::account_id(),
                        &challenger,
                        total_pot,
                        frame_support::traits::ExistenceRequirement::AllowDeath,
                    )?;

                    // Update developer
                    Developers::<T>::try_mutate(&developer, |maybe_profile| {
                        if let Some(profile) = maybe_profile {
                            profile.staked_amount = profile.staked_amount.saturating_sub(result_stake);
                            profile.failed_disputes = profile.failed_disputes.saturating_add(1);
                            profile.status = DeveloperStatus::Slashed;
                        }
                        Ok::<(), DispatchError>(())
                    })?;

                    // Mark result as slashed
                    Results::<T>::try_mutate(result_id, |maybe_result| {
                        if let Some(result) = maybe_result {
                            result.status = ResultStatus::Slashed;
                        }
                        Ok::<(), DispatchError>(())
                    })?;

                    Self::deposit_event(Event::DeveloperSlashed {
                        developer: developer.clone(),
                        result_id,
                        amount: result_stake,
                    });
                } else {
                    // Developer wins, gets challenger's stake
                    let total_pot = result_stake.saturating_add(challenger_stake);

                    T::Currency::transfer(
                        &Self::account_id(),
                        &developer,
                        total_pot,
                        frame_support::traits::ExistenceRequirement::AllowDeath,
                    )?;

                    Developers::<T>::try_mutate(&developer, |maybe_profile| {
                        if let Some(profile) = maybe_profile {
                            profile.staked_amount = profile.staked_amount.saturating_sub(result_stake);
                            profile.successful_disputes = profile.successful_disputes.saturating_add(1);
                        }
                        Ok::<(), DispatchError>(())
                    })?;

                    // Mark result as verified
                    Results::<T>::try_mutate(result_id, |maybe_result| {
                        if let Some(result) = maybe_result {
                            result.status = ResultStatus::Verified;
                        }
                        Ok::<(), DispatchError>(())
                    })?;
                }

                TotalStaked::<T>::mutate(|total| *total = total.saturating_sub(result_stake.saturating_add(challenger_stake)));

                Self::deposit_event(Event::DisputeResolved {
                    dispute_id,
                    outcome_id: outcome.to_u8(),
                    slashed_amount: if challenger_wins { result_stake } else { Zero::zero() },
                });

                Ok(())
            })
        }

        /// Claim pending revenue
        #[pallet::call_index(7)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn claim_revenue(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let amount = PendingRevenue::<T>::take(&who);
            ensure!(amount > Zero::zero(), Error::<T>::NoRevenueToClaim);

            T::Currency::transfer(
                &Self::account_id(),
                &who,
                amount,
                frame_support::traits::ExistenceRequirement::AllowDeath,
            )?;

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
        /// Get the pallet's account ID
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }
    }
}
