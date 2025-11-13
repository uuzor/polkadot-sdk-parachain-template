use crate::{mock::*, Error, Event, Developers, Results, DeveloperStatus};
use frame::testing_prelude::*;
use sp_core::ConstU32;
use codec::alloc::string::ToString;

// Helper function to create a battle and return its ID
fn create_battle() -> <Test as frame_system::Config>::Hash {
    // Create characters
    assert_ok!(BattleChain::create_character(RuntimeOrigin::signed(1), 0));
    assert_ok!(BattleChain::create_character(RuntimeOrigin::signed(4), 1));

    // Create battle offer
    assert_ok!(BattleChain::create_battle_offer(RuntimeOrigin::signed(1), 0, 100, 3));

    // Get battle_id from event
    let events = System::events();
    if let RuntimeEvent::BattleChain(pallet_battlechain::Event::BattleOfferCreated {
        battle_id,
        ..
    }) = &events.last().unwrap().event
    {
        let bid = *battle_id;

        // Accept battle
        assert_ok!(BattleChain::accept_battle(RuntimeOrigin::signed(4), bid, 1));

        bid
    } else {
        panic!("Expected BattleOfferCreated event");
    }
}

// Test developer registration
#[test]
fn register_developer_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let game_name = BoundedVec::try_from("TestGame".as_bytes().to_vec()).unwrap();

        // Register developer
        assert_ok!(GameOracle::register_developer(
            RuntimeOrigin::signed(1),
            game_name.clone()
        ));

        // Verify developer was registered
        let developer = Developers::<Test>::get(1).unwrap();
        assert_eq!(developer.developer, 1);
        assert_eq!(developer.game_name, game_name);
        assert_eq!(
            DeveloperStatus::from_u8(developer.status.to_u8()).unwrap(),
            DeveloperStatus::Pending
        );
        assert_eq!(developer.staked_amount, 1000); // DeveloperStakeAmount
    });
}

// Test developer already registered error
#[test]
fn cannot_register_twice() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let game_name = BoundedVec::try_from("TestGame".as_bytes().to_vec()).unwrap();

        // Register first time
        assert_ok!(GameOracle::register_developer(
            RuntimeOrigin::signed(1),
            game_name.clone()
        ));

        // Try to register again
        assert_noop!(
            GameOracle::register_developer(RuntimeOrigin::signed(1), game_name),
            Error::<Test>::DeveloperAlreadyRegistered
        );
    });
}

// Test verifying a developer (requires root)
#[test]
fn verify_developer_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let game_name = BoundedVec::try_from("TestGame".as_bytes().to_vec()).unwrap();

        // Register developer
        assert_ok!(GameOracle::register_developer(
            RuntimeOrigin::signed(1),
            game_name
        ));

        // Verify developer (requires root)
        assert_ok!(GameOracle::verify_developer(RuntimeOrigin::root(), 1));

        // Check status changed to Verified
        let developer = Developers::<Test>::get(1).unwrap();
        assert_eq!(
            DeveloperStatus::from_u8(developer.status.to_u8()).unwrap(),
            DeveloperStatus::Verified
        );
    });
}

// Test only root can verify developers
#[test]
fn verify_developer_requires_root() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let game_name = BoundedVec::try_from("TestGame".as_bytes().to_vec()).unwrap();

        // Register developer
        assert_ok!(GameOracle::register_developer(
            RuntimeOrigin::signed(1),
            game_name
        ));

        // Try to verify with non-root account
        assert_noop!(
            GameOracle::verify_developer(RuntimeOrigin::signed(2), 1),
            sp_runtime::traits::BadOrigin
        );
    });
}

// Test submitting a result
#[test]
fn submit_result_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let game_name = BoundedVec::try_from("TestGame".as_bytes().to_vec()).unwrap();

        // Register and verify developer
        assert_ok!(GameOracle::register_developer(
            RuntimeOrigin::signed(1),
            game_name.clone()
        ));
        assert_ok!(GameOracle::verify_developer(RuntimeOrigin::root(), 1));

        // Create a battle
        let battle_id = create_battle();

        // Submit result
        assert_ok!(GameOracle::submit_result(
            RuntimeOrigin::signed(1),
            battle_id,
            Some(1),          // winner
            1,                // player1
            4,                // player2
            3,                // player1_score
            2,                // player2_score
            3,                // rounds_played
            game_name,
            1,                // game_version
            BoundedVec::try_from(vec![]).unwrap(), // extension_data
        ));

        // Verify result was stored
        let events = System::events();
        assert!(events
            .iter()
            .any(|e| matches!(&e.event, RuntimeEvent::GameOracle(Event::ResultSubmitted { .. }))));
    });
}

// Test cannot submit result if not verified
#[test]
fn submit_result_requires_verified_developer() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let game_name = BoundedVec::try_from("TestGame".as_bytes().to_vec()).unwrap();

        // Register but don't verify developer
        assert_ok!(GameOracle::register_developer(
            RuntimeOrigin::signed(1),
            game_name.clone()
        ));

        // Create a battle
        let battle_id = create_battle();

        // Try to submit result without being verified
        assert_noop!(
            GameOracle::submit_result(
                RuntimeOrigin::signed(1),
                battle_id,
                Some(1),
                1,
                4,
                3,
                2,
                3,
                game_name,
                1,
                BoundedVec::try_from(vec![]).unwrap(),
            ),
            Error::<Test>::DeveloperNotVerified
        );
    });
}

// Test cannot submit result for non-existent battle
#[test]
fn submit_result_battle_must_exist() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let game_name = BoundedVec::try_from("TestGame".as_bytes().to_vec()).unwrap();

        // Register and verify developer
        assert_ok!(GameOracle::register_developer(
            RuntimeOrigin::signed(1),
            game_name.clone()
        ));
        assert_ok!(GameOracle::verify_developer(RuntimeOrigin::root(), 1));

        // Create fake battle_id
        let fake_battle_id = <Test as frame_system::Config>::Hash::default();

        // Try to submit result for non-existent battle
        assert_noop!(
            GameOracle::submit_result(
                RuntimeOrigin::signed(1),
                fake_battle_id,
                Some(1),
                1,
                4,
                3,
                2,
                3,
                game_name,
                1,
                BoundedVec::try_from(vec![]).unwrap(),
            ),
            Error::<Test>::BattleNotFound
        );
    });
}

// Test developer stake is locked on registration
#[test]
fn developer_stake_is_locked() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let game_name = BoundedVec::try_from("TestGame".as_bytes().to_vec()).unwrap();
        let initial_balance = Balances::free_balance(1);

        // Register developer
        assert_ok!(GameOracle::register_developer(
            RuntimeOrigin::signed(1),
            game_name
        ));

        // Check balance was reduced by stake amount
        let new_balance = Balances::free_balance(1);
        assert_eq!(initial_balance - new_balance, 1000); // DeveloperStakeAmount
    });
}

// Test result stake is locked on submission
#[test]
fn result_stake_is_locked() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let game_name = BoundedVec::try_from("TestGame".as_bytes().to_vec()).unwrap();

        // Register and verify developer
        assert_ok!(GameOracle::register_developer(
            RuntimeOrigin::signed(1),
            game_name.clone()
        ));
        assert_ok!(GameOracle::verify_developer(RuntimeOrigin::root(), 1));

        let battle_id = create_battle();

        let balance_before_submit = Balances::free_balance(1);

        // Submit result
        assert_ok!(GameOracle::submit_result(
            RuntimeOrigin::signed(1),
            battle_id,
            Some(1),
            1,
            4,
            3,
            2,
            3,
            game_name,
            1,
            BoundedVec::try_from(vec![]).unwrap(),
        ));

        // Check balance was reduced by result stake
        let balance_after_submit = Balances::free_balance(1);
        assert_eq!(balance_before_submit - balance_after_submit, 100); // ResultStakeAmount
    });
}

// Test cannot submit duplicate result for same battle
#[test]
fn cannot_submit_duplicate_result() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let game_name = BoundedVec::try_from("TestGame".as_bytes().to_vec()).unwrap();

        // Register and verify developer
        assert_ok!(GameOracle::register_developer(
            RuntimeOrigin::signed(1),
            game_name.clone()
        ));
        assert_ok!(GameOracle::verify_developer(RuntimeOrigin::root(), 1));

        let battle_id = create_battle();

        // Submit first result
        assert_ok!(GameOracle::submit_result(
            RuntimeOrigin::signed(1),
            battle_id,
            Some(1),
            1,
            4,
            3,
            2,
            3,
            game_name.clone(),
            1,
            BoundedVec::try_from(vec![]).unwrap(),
        ));

        // Try to submit duplicate result
        assert_noop!(
            GameOracle::submit_result(
                RuntimeOrigin::signed(1),
                battle_id,
                Some(1),
                1,
                4,
                3,
                2,
                3,
                game_name,
                1,
                BoundedVec::try_from(vec![]).unwrap(),
            ),
            Error::<Test>::ResultAlreadyExists
        );
    });
}

// Test developer interactions tracking
#[test]
fn developer_submission_count_increments() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let game_name = BoundedVec::try_from("TestGame".as_bytes().to_vec()).unwrap();

        // Register and verify developer
        assert_ok!(GameOracle::register_developer(
            RuntimeOrigin::signed(1),
            game_name.clone()
        ));
        assert_ok!(GameOracle::verify_developer(RuntimeOrigin::root(), 1));

        // Initial submission count should be 0
        let developer = Developers::<Test>::get(1).unwrap();
        assert_eq!(developer.total_submissions, 0);

        // Create multiple battles and submit results
        for _ in 0..3 {
            let battle_id = create_battle();

            assert_ok!(GameOracle::submit_result(
                RuntimeOrigin::signed(1),
                battle_id,
                Some(1),
                1,
                4,
                3,
                2,
                3,
                game_name.clone(),
                1,
                BoundedVec::try_from(vec![]).unwrap(),
            ));
        }

        // Check submission count increased
        let developer = Developers::<Test>::get(1).unwrap();
        assert_eq!(developer.total_submissions, 3);
    });
}
