
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_campaign_should_work() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(Task::create_campaign(Origin::signed(BOB),0,  1000));

		let campaign = Task::campaigns(0).unwrap();
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
		let sys_account_1 = Task::account_id(0);
		assert_eq!(Balances::free_balance(sys_account_1), 1000);

		//Create campaign 2
		assert_ok!(Task::create_campaign(Origin::signed(BOB),1, 5000));
		//Check client balance should be reserve bond amount when deposi for campaign 2
		//reserve amount = 1000.max(5000*permil(2)) = 1000
		//(T::CampaignDepositMinimum::get()).max(T::CampaignDeposit::get() * value);
		assert_eq!(Balances::reserved_balance(BOB), 2000);
	});
}

#[test]
fn payment_should_be_working() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(Task::create_campaign(Origin::signed(BOB), 0, 5000));
		System::set_block_number(10);
		assert_ok!(Task::payment(Origin::root(), 0, vec![USER1, USER2], 1000u32.into()));
		//Check unreserved balance for BOB
		assert_eq!(Balances::reserved_balance(BOB), 0);
		//Check balance of user 1
		assert_eq!(Task::balance_of(USER1).1, 1000);
		//Check balance of user 2
		assert_eq!(Task::balance_of(USER2).1, 1000);

		// check campaign payout
		assert_eq!(Task::campaign_payout(0), vec![USER1, USER2]);
	});
}

#[test]
fn claim_should_be_working() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(Task::create_campaign(Origin::signed(BOB), 0, 5000));
		System::set_block_number(10);
		assert_ok!(Task::payment(Origin::root(), 0, vec![USER1, USER2], 1000u32.into()));
		//Check balance of user 1
		assert_eq!(Task::balance_of(USER1).1, 1000);
		//Check balance of user 2
		assert_eq!(Task::balance_of(USER2).1, 1000);

		// check campaign payout
		assert_eq!(Task::campaign_payout(0), vec![USER1, USER2]);

		System::set_block_number(25);
		//Before user 1 claim
		assert_eq!(Balances::free_balance(USER1), 0);
		// User 1 claim
		assert_ok!(Task::claim(Origin::root(), 0, 1000, USER1));

		//after user 1 claim
		assert_eq!(Balances::free_balance(USER1), 1000);

		//Remaining balance in system
		assert_eq!(Balances::free_balance(Task::account_id(0)), 4000);

		// User 2 claim
		assert_ok!(Task::claim(Origin::root(), 0, 1000, USER2));

		//Remaining balance in system
		assert_eq!(Balances::free_balance(Task::account_id(0)), 3000);

		// Balance storage should be zero
		assert_eq!(Task::balance_of(USER1).1, 0);
		assert_eq!(Task::balance_of(USER2).1, 0);
		// Index payout should be empty
		assert_eq!(Task::index_payout().is_empty(), true);
	});
}
