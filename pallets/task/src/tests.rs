use super::*;
use crate::{mock::*, Error};

use frame_support::{assert_noop, assert_ok};

fn id_account(index: &Vec<u8>) -> AccountId {
	<Test as Config>::PalletId::get().into_sub_account(index)
}

#[test]
fn create_campaign_should_work() {
	new_test_ext().execute_with(|| {
		let campaign_id = "0".as_bytes().to_vec();
		// Dispatch a signed extrinsic.
		assert_ok!(Task::create_campaign(Origin::signed(BOB), campaign_id.clone(), 1000));

		let campaign = Task::campaigns(&campaign_id).unwrap();
		//Check client
		assert_eq!(campaign.client, BOB);
		//Check value
		assert_eq!(campaign.value, 1000);
		//Check bond amount when create campaign
		assert_eq!(campaign.bond, 1000);

		//Check client balance should be reserve bond amount
		//reserve amount = 1000.max(1000*permil(2)) = 1000
		assert_eq!(Balances::reserved_balance(BOB), 1000);
		//Check system account when clien deposit
		let sys_account_1 = Task::account_id(&campaign_id);
		assert_eq!(Balances::free_balance(sys_account_1), 1000);
	});
}

#[test]
fn payment_should_be_working() {
	new_test_ext().execute_with(|| {
		let campaign_id_0 = "0".as_bytes().to_vec();
		// Dispatch a signed extrinsic.
		assert_ok!(Task::create_campaign(Origin::signed(BOB), campaign_id_0.clone(), 5000));
		System::set_block_number(10);
		assert_ok!(Task::payment(
			Origin::root(),
			campaign_id_0.clone(),
			vec![USER1, USER2],
			1000u32.into()
		));
		//Check unreserved balance for BOB
		assert_eq!(Balances::reserved_balance(BOB), 0);
		//Check balance of user 1
		assert_eq!(Task::balance_of(USER1).1, 1000);
		//Check balance of user 2
		assert_eq!(Task::balance_of(USER2).1, 1000);

		// check campaign payout
		assert_eq!(Task::campaign_payout(campaign_id_0), vec![USER1, USER2]);
	});
}

#[test]
fn claim_should_be_working() {
	new_test_ext().execute_with(|| {
		let campaign_id_0 = "0".as_bytes().to_vec();

		// Dispatch a signed extrinsic.
		assert_ok!(Task::create_campaign(Origin::signed(BOB), campaign_id_0.clone(), 5000));
		System::set_block_number(10);
		assert_ok!(Task::payment(
			Origin::root(),
			campaign_id_0.clone(),
			vec![USER1, USER2],
			1000u32.into()
		));
		//Check balance of user 1
		assert_eq!(Task::balance_of(USER1).1, 1000);
		//Check balance of user 2
		assert_eq!(Task::balance_of(USER2).1, 1000);

		// check campaign payout
		assert_eq!(Task::campaign_payout(campaign_id_0.clone()), vec![USER1, USER2]);

		System::set_block_number(25);
		//Before user 1 claim
		assert_eq!(Balances::free_balance(USER1), 0);
		// User 1 claim
		assert_ok!(Task::claim(Origin::root(), campaign_id_0.clone(), 1000, USER1));

		// //after user 1 claim
		assert_eq!(Balances::free_balance(USER1), 1000);

		//Remaining balance in system
		assert_eq!(Balances::free_balance(Task::account_id(&campaign_id_0)), 4000);

		// User 2 claim
		assert_ok!(Task::claim(Origin::root(), campaign_id_0.clone(), 1000, USER2));

		//Remaining balance in system
		assert_eq!(Balances::free_balance(Task::account_id(&campaign_id_0)), 3000);

		// Balance storage should be zero
		assert_eq!(Task::balance_of(USER1).1, 0);
		assert_eq!(Task::balance_of(USER2).1, 0);
		// Index payout should be empty
		assert_eq!(Task::index_payout().is_empty(), true);
	});
}

#[test]
fn multiple_campaigns_should_work() {
	new_test_ext().execute_with(|| {
		let campaign_id_0 = "0".as_bytes().to_vec();
		let campaign_id_1 = "1".as_bytes().to_vec();
		// Dispatch a signed extrinsic.
		assert_ok!(Task::create_campaign(Origin::signed(BOB), campaign_id_0.clone(), 1000));

		let campaign = Task::campaigns(&campaign_id_0).unwrap();
		//Check client
		assert_eq!(campaign.client, BOB);
		//Check value
		assert_eq!(campaign.value, 1000);
		//Check bond amount when create campaign
		assert_eq!(campaign.bond, 1000);

		//Check client balance should be reserve bond amount
		//reserve amount = 1000.max(1000*permil(2)) = 1000
		assert_eq!(Balances::reserved_balance(BOB), 1000);
		//Check system account when clien deposit
		let sys_account_1 = id_account(&campaign_id_0);
		assert_eq!(Balances::free_balance(sys_account_1.clone()), 1000);

		//Create campaign 2
		assert_ok!(Task::create_campaign(Origin::signed(ALICE), campaign_id_1.clone(), 5000));
		let campaign2 = Task::campaigns(&campaign_id_1).unwrap();
		//Check client balance should be reserve bond amount when deposi for campaign 2
		//reserve amount = 1000.max(5000*permil(2)) = 1000
		//(T::CampaignDepositMinimum::get()).max(T::CampaignDeposit::get() * value);

		assert_eq!(Balances::reserved_balance(ALICE), 1000);

		//Check client
		assert_eq!(campaign2.client, ALICE);
		//Check value
		assert_eq!(campaign2.value, 5000);
		//Check bond amount when create campaign
		assert_eq!(campaign2.bond, 1000);

		let sys_account_2 = id_account(&campaign_id_1);
		assert!(sys_account_1 != sys_account_2);

		assert_eq!(Balances::free_balance(sys_account_2), 5000);

		System::set_block_number(10);
		assert_ok!(Task::payment(
			Origin::root(),
			campaign_id_1.clone(),
			vec![USER1, USER2],
			2500u32.into()
		));
		assert_ok!(Task::payment(
			Origin::root(),
			campaign_id_0.clone(),
			vec![USER1, USER2],
			500u32.into()
		));

		System::set_block_number(25);
		//Before user 1 claim
		assert_eq!(Balances::free_balance(USER1), 0);
		// User 1 claim for campaign 1
		assert_ok!(Task::claim(Origin::root(), campaign_id_0.clone(), 500, USER1));
		//after user 1 claim
		assert_eq!(Balances::free_balance(USER1), 500);

		assert_eq!(Balances::free_balance(id_account(&campaign_id_0)), 500);

		// User 2 claim for campaign 1
		assert_ok!(Task::claim(Origin::root(), campaign_id_0, 499, USER2));
		// User 1 claim for campaign 2
		assert_ok!(Task::claim(Origin::root(), campaign_id_1, 1000, USER1));
		// //after user 1 claim for campaign 2
		//500 + 1000
		assert_eq!(Balances::free_balance(USER1), 1500);
	});
}
