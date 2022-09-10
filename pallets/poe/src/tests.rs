use super::*;
use crate::{mock::*, Error};
use frame_support::pallet_prelude::Get;
use frame_support::{assert_noop, assert_ok, BoundedVec};

#[test]
fn get_length() {
	// create claim OK
	new_test_ext().execute_with(|| {
		assert_eq!(<<Test as Config>::MaxClaimLength as Get<u32>>::get(), 512);
	})
}

#[test]
fn create_claim_works() {
	// create claim OK
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

		let bounded_claim =
			BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();

		assert_eq!(
			Proofs::<Test>::get(&bounded_claim),
			Some((1, frame_system::Pallet::<Test>::block_number()))
		);
	})
}

#[test]
fn create_claim_failed_when_claim_too_long() {
	// create claim Error ClaimTooLong
	new_test_ext().execute_with(|| {
		let claim = vec![0; 1000];
		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ClaimTooLong
		);
	})
}

#[test]
fn create_claim_failed_when_claim_already_exist() {
	// create claim Error ProofAlreadyExist
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ProofAlreadyExist
		);
	})
}

#[test]
fn revoke_claim_works() {
	// revoke claim OK
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		let bounded_claim =
			BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();

		assert_ok!(PoeModule::revoke_claim(Origin::signed(1), claim.clone()));

		assert_eq!(Proofs::<Test>::get(&bounded_claim), None);
	})
}

#[test]
fn revoke_claim_failed_when_claim_not_exist() {
	// revoke claim Error ClaimNotExist
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];

		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ClaimNotExist
		);
	})
}

#[test]
fn revoke_claim_failed_when_not_claim_owner() {
	// revoke claim Error NotClaimOwner
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(2), claim.clone()),
			Error::<Test>::NotClaimOwner
		);
	})
}

#[test]
fn transfer_claim_works() {
	// transfer claim OK
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		let bounded_claim =
			BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();

		assert_eq!(
			Proofs::<Test>::get(&bounded_claim),
			Some((1, frame_system::Pallet::<Test>::block_number()))
		);

		assert_ok!(PoeModule::transfer_claim(Origin::signed(1), 2, claim.clone()));

		assert_eq!(
			Proofs::<Test>::get(&bounded_claim),
			Some((2, frame_system::Pallet::<Test>::block_number()))
		);
	})
}

#[test]
fn transfer_claim_failed_when_not_claim_owner() {
	// transfer claim Error NotClaimOwner
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		let bounded_claim =
			BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();

		assert_eq!(
			Proofs::<Test>::get(&bounded_claim),
			Some((1, frame_system::Pallet::<Test>::block_number()))
		);

		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(2), 3, claim.clone()),
			Error::<Test>::NotClaimOwner
		);
	})
}

#[test]
fn transfer_claim_failed_when_claim_not_exist() {
	// transfer claim Error ClaimNotExist
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		let bounded_claim =
			BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();

		assert_eq!(
			Proofs::<Test>::get(&bounded_claim),
			Some((1, frame_system::Pallet::<Test>::block_number()))
		);

		let claim2 = vec![0, 2];

		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(2), 3, claim2.clone()),
			Error::<Test>::ClaimNotExist
		);
	})
}
