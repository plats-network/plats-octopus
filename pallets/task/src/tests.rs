use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_campaign_should_work() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(Task::create_campaign(Origin::signed(BOB), 1000));
	});
}
