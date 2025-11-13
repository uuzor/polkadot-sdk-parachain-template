use crate as pallet_battlechain;
use frame::{prelude::*, runtime::prelude::*, testing_prelude::*};

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

// Test externalities initialization
pub fn new_test_ext() -> TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 10000),  // Alice
            (2, 10000),  // Bob
            (3, 10000),  // Charlie
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    t.into()
}
