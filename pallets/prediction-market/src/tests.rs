use crate::{mock::*, Error, Event, Markets, Predictions, MarketState, MarketOutcome};
use frame::testing_prelude::*;
use pallet_battlechain::BattleState;

// Helper function to create a battle and return its ID
fn create_battle() -> <Test as frame_system::Config>::Hash {
    // Create characters
    assert_ok!(BattleChain::create_character(RuntimeOrigin::signed(1), 0));
    assert_ok!(BattleChain::create_character(RuntimeOrigin::signed(2), 1));

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
        assert_ok!(BattleChain::accept_battle(RuntimeOrigin::signed(2), bid, 1));

        bid
    } else {
        panic!("Expected BattleOfferCreated event");
    }
}

// Test creating a prediction market
#[test]
fn create_market_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let battle_id = create_battle();

        // Create prediction market
        assert_ok!(PredictionMarket::create_market(
            RuntimeOrigin::signed(1),
            battle_id
        ));

        // Verify market was created
        let events = System::events();
        assert!(events
            .iter()
            .any(|e| matches!(&e.event, RuntimeEvent::PredictionMarket(Event::MarketCreated { .. }))));
    });
}

// Test placing a prediction
#[test]
fn place_prediction_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let battle_id = create_battle();

        // Create market
        assert_ok!(PredictionMarket::create_market(
            RuntimeOrigin::signed(1),
            battle_id
        ));

        // Get market_id from event
        let events = System::events();
        let market_id = if let RuntimeEvent::PredictionMarket(Event::MarketCreated {
            market_id,
            ..
        }) = &events.last().unwrap().event
        {
            *market_id
        } else {
            panic!("Expected MarketCreated event");
        };

        // Place prediction on Player1Wins (outcome_id = 0)
        assert_ok!(PredictionMarket::place_prediction(
            RuntimeOrigin::signed(3),
            market_id,
            0, // Player1Wins
            500
        ));

        // Verify prediction was placed
        let prediction = Predictions::<Test>::get(market_id, 3).unwrap();
        assert_eq!(prediction.outcome_id, 0);
        assert_eq!(prediction.amount, 500);
        assert_eq!(prediction.claimed, false);
    });
}

// Test placing multiple predictions
#[test]
fn multiple_predictions_work() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let battle_id = create_battle();
        assert_ok!(PredictionMarket::create_market(
            RuntimeOrigin::signed(1),
            battle_id
        ));

        let events = System::events();
        let market_id = if let RuntimeEvent::PredictionMarket(Event::MarketCreated {
            market_id,
            ..
        }) = &events.last().unwrap().event
        {
            *market_id
        } else {
            panic!("Expected MarketCreated event");
        };

        // Multiple users place predictions
        assert_ok!(PredictionMarket::place_prediction(
            RuntimeOrigin::signed(3),
            market_id,
            0, // Player1Wins
            500
        ));

        assert_ok!(PredictionMarket::place_prediction(
            RuntimeOrigin::signed(4),
            market_id,
            1, // Player2Wins
            300
        ));

        // Verify market pool was updated
        let market = Markets::<Test>::get(market_id).unwrap();
        assert_eq!(market.total_pool, 800);
        assert_eq!(market.player1_pool, 500);
        assert_eq!(market.player2_pool, 300);
    });
}

// Test invalid outcome ID
#[test]
fn invalid_outcome_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let battle_id = create_battle();
        assert_ok!(PredictionMarket::create_market(
            RuntimeOrigin::signed(1),
            battle_id
        ));

        let events = System::events();
        let market_id = if let RuntimeEvent::PredictionMarket(Event::MarketCreated {
            market_id,
            ..
        }) = &events.last().unwrap().event
        {
            *market_id
        } else {
            panic!("Expected MarketCreated event");
        };

        // Try to place prediction with invalid outcome_id (3 is out of range)
        assert_noop!(
            PredictionMarket::place_prediction(RuntimeOrigin::signed(3), market_id, 3, 500),
            Error::<Test>::InvalidOutcome
        );
    });
}

// Test locking market
#[test]
fn lock_market_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let battle_id = create_battle();
        assert_ok!(PredictionMarket::create_market(
            RuntimeOrigin::signed(1),
            battle_id
        ));

        let events = System::events();
        let market_id = if let RuntimeEvent::PredictionMarket(Event::MarketCreated {
            market_id,
            ..
        }) = &events.last().unwrap().event
        {
            *market_id
        } else {
            panic!("Expected MarketCreated event");
        };

        // Lock the market
        assert_ok!(PredictionMarket::lock_market(
            RuntimeOrigin::signed(1),
            market_id
        ));

        // Verify market is locked
        let market = Markets::<Test>::get(market_id).unwrap();
        assert_eq!(
            MarketState::from_u8(market.state_id).unwrap(),
            MarketState::Locked
        );
    });
}

// Test cannot place prediction on locked market
#[test]
fn cannot_predict_on_locked_market() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let battle_id = create_battle();
        assert_ok!(PredictionMarket::create_market(
            RuntimeOrigin::signed(1),
            battle_id
        ));

        let events = System::events();
        let market_id = if let RuntimeEvent::PredictionMarket(Event::MarketCreated {
            market_id,
            ..
        }) = &events.last().unwrap().event
        {
            *market_id
        } else {
            panic!("Expected MarketCreated event");
        };

        // Lock the market
        assert_ok!(PredictionMarket::lock_market(
            RuntimeOrigin::signed(1),
            market_id
        ));

        // Try to place prediction on locked market
        assert_noop!(
            PredictionMarket::place_prediction(RuntimeOrigin::signed(3), market_id, 0, 500),
            Error::<Test>::MarketNotOpen
        );
    });
}

// Test market cannot be created twice for same battle
#[test]
fn market_already_exists_error() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let battle_id = create_battle();

        // Create first market
        assert_ok!(PredictionMarket::create_market(
            RuntimeOrigin::signed(1),
            battle_id
        ));

        // Try to create second market for same battle
        assert_noop!(
            PredictionMarket::create_market(RuntimeOrigin::signed(2), battle_id),
            Error::<Test>::MarketAlreadyExists
        );
    });
}

// Test market state transitions
#[test]
fn market_state_transitions() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let battle_id = create_battle();
        assert_ok!(PredictionMarket::create_market(
            RuntimeOrigin::signed(1),
            battle_id
        ));

        let events = System::events();
        let market_id = if let RuntimeEvent::PredictionMarket(Event::MarketCreated {
            market_id,
            ..
        }) = &events.last().unwrap().event
        {
            *market_id
        } else {
            panic!("Expected MarketCreated event");
        };

        // Initial state should be Open
        let market = Markets::<Test>::get(market_id).unwrap();
        assert_eq!(
            MarketState::from_u8(market.state_id).unwrap(),
            MarketState::Open
        );

        // Lock market
        assert_ok!(PredictionMarket::lock_market(
            RuntimeOrigin::signed(1),
            market_id
        ));

        let market = Markets::<Test>::get(market_id).unwrap();
        assert_eq!(
            MarketState::from_u8(market.state_id).unwrap(),
            MarketState::Locked
        );
    });
}
