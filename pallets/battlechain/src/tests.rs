use crate::{mock::*, Error, Event, Characters, Battles, BattleOffers};
use frame::testing_prelude::*;

// Import RuntimeEvent from mock
use mock::RuntimeEvent;

// Test creating a character with each class
#[test]
fn create_character_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Create Warrior (class_id = 0)
        assert_ok!(BattleChain::create_character(RuntimeOrigin::signed(1), 0));
        System::assert_last_event(
            Event::CharacterCreated {
                owner: 1,
                character_id: 0,
                class_id: 0,
            }
            .into(),
        );

        // Verify character was stored correctly
        let character = Characters::<Test>::get(0).unwrap();
        assert_eq!(character.owner, 1);
        assert_eq!(character.class_id, 0);
        assert_eq!(character.level, 1);
        assert_eq!(character.wins, 0);
        assert_eq!(character.losses, 0);
    });
}

// Test creating characters of all classes
#[test]
fn create_all_character_classes() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Create one of each class
        for class_id in 0..=4 {
            assert_ok!(BattleChain::create_character(RuntimeOrigin::signed(1), class_id));
        }

        // Verify 5 characters were created
        assert_eq!(Characters::<Test>::iter().count(), 5);
    });
}

// Test invalid character class
#[test]
fn create_character_invalid_class() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Try to create character with invalid class_id (5 is out of range)
        assert_noop!(
            BattleChain::create_character(RuntimeOrigin::signed(1), 5),
            Error::<Test>::InvalidCharacterClass
        );
    });
}

// Test creating a battle offer
#[test]
fn create_battle_offer_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Create a character first
        assert_ok!(BattleChain::create_character(RuntimeOrigin::signed(1), 0));

        // Create battle offer with 100 tokens stake, 3 rounds
        assert_ok!(BattleChain::create_battle_offer(RuntimeOrigin::signed(1), 0, 100, 3));

        // Verify battle offer was created
        assert!(BattleOffers::<Test>::iter().count() > 0);
    });
}

// Test invalid round count in battle offer
#[test]
fn create_battle_offer_invalid_rounds() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Create a character first
        assert_ok!(BattleChain::create_character(RuntimeOrigin::signed(1), 0));

        // Try to create battle offer with invalid round count (4 is not allowed)
        assert_noop!(
            BattleChain::create_battle_offer(RuntimeOrigin::signed(1), 0, 100, 4),
            Error::<Test>::InvalidRoundCount
        );
    });
}

// Test accepting a battle offer
#[test]
fn accept_battle_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Create characters for both players
        assert_ok!(BattleChain::create_character(RuntimeOrigin::signed(1), 0)); // char 0
        assert_ok!(BattleChain::create_character(RuntimeOrigin::signed(2), 1)); // char 1

        // Player 1 creates battle offer
        assert_ok!(BattleChain::create_battle_offer(RuntimeOrigin::signed(1), 0, 100, 3));

        // Get the battle_id from the event
        let events = System::events();
        let battle_id = if let RuntimeEvent::BattleChain(Event::BattleOfferCreated {
            battle_id,
            ..
        }) = &events.last().unwrap().event
        {
            *battle_id
        } else {
            panic!("Expected BattleOfferCreated event");
        };

        // Player 2 accepts the battle
        assert_ok!(BattleChain::accept_battle(RuntimeOrigin::signed(2), battle_id, 1));

        // Verify battle was created
        let battle = Battles::<Test>::get(battle_id).unwrap();
        assert_eq!(battle.player1, 1);
        assert_eq!(battle.player2, 2);
        assert_eq!(battle.max_rounds, 3);
        assert_eq!(battle.current_round, 1);
    });
}

// Test cannot battle self
#[test]
fn cannot_create_battle_with_self() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Create two characters owned by same player
        assert_ok!(BattleChain::create_character(RuntimeOrigin::signed(1), 0)); // char 0
        assert_ok!(BattleChain::create_character(RuntimeOrigin::signed(1), 1)); // char 1

        // Player 1 creates battle offer
        assert_ok!(BattleChain::create_battle_offer(RuntimeOrigin::signed(1), 0, 100, 3));

        // Get the battle_id
        let events = System::events();
        let battle_id = if let RuntimeEvent::BattleChain(Event::BattleOfferCreated {
            battle_id,
            ..
        }) = &events.last().unwrap().event
        {
            *battle_id
        } else {
            panic!("Expected BattleOfferCreated event");
        };

        // Try to accept own battle offer
        assert_noop!(
            BattleChain::accept_battle(RuntimeOrigin::signed(1), battle_id, 1),
            Error::<Test>::CannotBattleSelf
        );
    });
}

// Test character ownership verification
#[test]
fn must_own_character_to_create_offer() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Player 1 creates a character
        assert_ok!(BattleChain::create_character(RuntimeOrigin::signed(1), 0));

        // Player 2 tries to create battle offer with Player 1's character
        assert_noop!(
            BattleChain::create_battle_offer(RuntimeOrigin::signed(2), 0, 100, 3),
            Error::<Test>::NotCharacterOwner
        );
    });
}

// Test character not found error
#[test]
fn character_not_found_error() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Try to create battle offer with non-existent character
        assert_noop!(
            BattleChain::create_battle_offer(RuntimeOrigin::signed(1), 999, 100, 3),
            Error::<Test>::CharacterNotFound
        );
    });
}

// Test battle execution (basic round execution)
#[test]
fn execute_round_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Create characters
        assert_ok!(BattleChain::create_character(RuntimeOrigin::signed(1), 0)); // Warrior
        assert_ok!(BattleChain::create_character(RuntimeOrigin::signed(2), 1)); // Assassin

        // Create and accept battle
        assert_ok!(BattleChain::create_battle_offer(RuntimeOrigin::signed(1), 0, 100, 3));

        let events = System::events();
        let battle_id = if let RuntimeEvent::BattleChain(Event::BattleOfferCreated {
            battle_id,
            ..
        }) = &events.last().unwrap().event
        {
            *battle_id
        } else {
            panic!("Expected BattleOfferCreated event");
        };

        assert_ok!(BattleChain::accept_battle(RuntimeOrigin::signed(2), battle_id, 1));

        // Execute first round
        assert_ok!(BattleChain::execute_round(RuntimeOrigin::signed(1), battle_id));

        // Verify round was executed
        let battle = Battles::<Test>::get(battle_id).unwrap();
        assert!(battle.current_round >= 1);
    });
}
