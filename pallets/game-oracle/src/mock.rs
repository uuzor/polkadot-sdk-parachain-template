use crate as pallet_game_oracle;
use frame::{prelude::*, runtime::prelude::*, testing_prelude::*};
use frame::deps::frame_support::PalletId;

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet
#[frame_construct_runtime]
mod runtime {
    #[runtime::runtime]
    #[runtime::derive(
        RuntimeCall,
        RuntimeEvent,
        RuntimeError,
        RuntimeOrigin,
        RuntimeFreezeReason,
        RuntimeHoldReason,
        RuntimeSlashReason,
        RuntimeLockId,
        RuntimeTask
    )]
    pub struct Test;

    #[runtime::pallet_index(0)]
    pub type System = frame_system;

    #[runtime::pallet_index(1)]
    pub type Balances = pallet_balances;

    #[runtime::pallet_index(2)]
    pub type BattleChain = pallet_battlechain;

    #[runtime::pallet_index(3)]
    pub type GameOracle = pallet_game_oracle;
}

// System pallet configuration
#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
}

// Balances pallet configuration
#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    type AccountStore = System;
}

// BattleChain pallet configuration
impl pallet_battlechain::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
}

// GameOracle pallet configuration
parameter_types! {
    pub const GameOraclePalletId: PalletId = PalletId(*b"gameorac");
    pub const ProviderRevShareBps: u16 = 7000; // 70%
    pub const OracleQueryFee: u128 = 100;
    pub const DeveloperStakeAmount: u128 = 1000;
    pub const ResultStakeAmount: u128 = 100;
    pub const DisputePeriodBlocks: u64 = 100;
    pub const DisputeStakeAmount: u128 = 200;
}

impl pallet_game_oracle::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type PalletId = GameOraclePalletId;
    type ProviderRevShareBps = ProviderRevShareBps;
    type QueryFee = OracleQueryFee;
    type DeveloperStake = DeveloperStakeAmount;
    type ResultStake = ResultStakeAmount;
    type DisputePeriod = DisputePeriodBlocks;
    type DisputeStake = DisputeStakeAmount;
}

// Test externalities initialization
pub fn new_test_ext() -> TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 10000),  // Alice - developer
            (2, 10000),  // Bob - challenger
            (3, 10000),  // Charlie - querier
            (4, 10000),  // Dave - player
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    t.into()
}
