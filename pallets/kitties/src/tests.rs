use std::ops::Add;

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{new_test_ext, Event as TestEvent, KittiesModule, Origin, System, Test};

const ACCOUNT_WITH_BALANCE_1: u64 = 1;
const ACCOUNT_WITH_BALANCE_2: u64 = 2;
const ACCOUNT_WITH_NO_BALANCE: u64 = 4;

#[test]
fn create_success() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = ACCOUNT_WITH_BALANCE_1;
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
	});
}

#[test]
fn create_failed_not_enough_balance() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = ACCOUNT_WITH_NO_BALANCE;
		assert_noop!(
			KittiesModule::create(Origin::signed(account_id)),
			Error::<Test>::NotEnoughBalance
		);
	});
}

#[test]
fn create_failed_invalid_kitty_id() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = ACCOUNT_WITH_BALANCE_1;
		let max_index = <Test as Config>::KittyIndex::max_value();
		NextKittyId::<Test>::set(max_index);
		assert_noop!(
			KittiesModule::create(Origin::signed(account_id)),
			Error::<Test>::InvalidKittyId
		);
	});
}

#[test]
fn create_failed_own_too_many_kitties() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = ACCOUNT_WITH_BALANCE_1;
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));

		assert_noop!(
			KittiesModule::create(Origin::signed(account_id)),
			Error::<Test>::OwnTooManyKitties
		);
	});
}

#[test]
fn breed_success() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = ACCOUNT_WITH_BALANCE_1;

		let kitty_id_1 = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));

		let kitty_id_2 = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));

		assert_ok!(KittiesModule::breed(Origin::signed(account_id), kitty_id_1, kitty_id_2));
	});
}

#[test]
fn breed_failed_same_kitty_id() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = ACCOUNT_WITH_BALANCE_1;

		let kitty_id = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));

		assert_noop!(
			KittiesModule::breed(Origin::signed(account_id), kitty_id, kitty_id),
			Error::<Test>::SameKittyId
		);
	});
}

#[test]
fn breed_failed_invalid_kitty_id() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = ACCOUNT_WITH_BALANCE_1;

		let kitty_id = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));

		let invalid_kitty_id = NextKittyId::<Test>::get();

		assert_noop!(
			KittiesModule::breed(Origin::signed(account_id), kitty_id, invalid_kitty_id),
			Error::<Test>::InvalidKittyId
		);
	});
}

#[test]
fn breed_failed_own_too_many_kitties() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = ACCOUNT_WITH_BALANCE_1;

		let kitty_id_1 = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));

		let kitty_id_2 = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));

		assert_ok!(KittiesModule::create(Origin::signed(account_id)));

		assert_noop!(
			KittiesModule::breed(Origin::signed(account_id), kitty_id_1, kitty_id_2),
			Error::<Test>::OwnTooManyKitties
		);
	});
}

#[test]
fn transfor_success() {
	new_test_ext().execute_with(|| {
		let account_id_1: u64 = ACCOUNT_WITH_BALANCE_1;
		let account_id_2: u64 = ACCOUNT_WITH_BALANCE_2;

		let kitty_id = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(account_id_1)));

		assert_ok!(KittiesModule::transfer(Origin::signed(account_id_1), kitty_id, account_id_2));
	});
}

#[test]
fn transfer_failed_not_enough_balance() {
	new_test_ext().execute_with(|| {
		let account_id_1: u64 = ACCOUNT_WITH_BALANCE_1;
		let account_id_2: u64 = ACCOUNT_WITH_NO_BALANCE;

		let kitty_id = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(account_id_1)));

		assert_noop!(
			KittiesModule::transfer(Origin::signed(account_id_1), kitty_id, account_id_2),
			Error::<Test>::NotEnoughBalance
		);
	});
}

#[test]
fn transfer_failed_not_owner() {
	new_test_ext().execute_with(|| {
		let account_id_1: u64 = ACCOUNT_WITH_BALANCE_1;
		let account_id_2: u64 = ACCOUNT_WITH_BALANCE_2;

		let kitty_id = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(account_id_2)));

		assert_noop!(
			KittiesModule::transfer(Origin::signed(account_id_1), kitty_id, account_id_2),
			Error::<Test>::NotOwner
		);
	});
}

#[test]
fn transfer_failed_own_too_many_kitties() {
	new_test_ext().execute_with(|| {
		let account_id_1: u64 = ACCOUNT_WITH_BALANCE_2;
		let account_id_2: u64 = ACCOUNT_WITH_BALANCE_1;

		let kitty_id = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(account_id_1)));

		assert_ok!(KittiesModule::create(Origin::signed(account_id_2)));
		assert_ok!(KittiesModule::create(Origin::signed(account_id_2)));
		assert_ok!(KittiesModule::create(Origin::signed(account_id_2)));

		assert_noop!(
			KittiesModule::transfer(Origin::signed(account_id_1), kitty_id, account_id_2),
			Error::<Test>::OwnTooManyKitties
		);
	});
}