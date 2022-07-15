use crate as pallet_task;
use frame_support::{parameter_types, PalletId};
use frame_system as system;
use frame_system::EnsureRoot;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32, Permill,
};
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub type Balance = u128;
pub type AccountId = AccountId32;
// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Config<T>,  Storage, Event<T>},
		Task: pallet_task::{Pallet, Call,  Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
	pub const MaxReserves: u32 = 50;
}
impl pallet_balances::Config for Test {
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = frame_system::Pallet<Test>;
	type MaxLocks = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
}

parameter_types! {
	pub const CampaignDepositMinimum: Balance = 1000;
	pub const CampaignDeposit : Permill = Permill::from_percent(2);
	pub const ClaimDuration : u64 = 10;
	pub const PayoutDuration: u64 = 20;
	pub const TaskPalletId: PalletId = PalletId(*b"plt/task");
}

impl pallet_task::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type CampaignDepositMinimum = CampaignDepositMinimum;
	type CampaignDeposit = CampaignDeposit;
	// type RewardOrigin = EnsureRoot<AccountId>;
	type ClaimDuration = ClaimDuration;
	type PayoutDuration = PayoutDuration;
	type PalletId = TaskPalletId;
}

pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
pub const BOB: AccountId32 = AccountId32::new([2u8; 32]);

pub const USER1: AccountId32 = AccountId32::new([3u8; 32]);
pub const USER2: AccountId32 = AccountId32::new([4u8; 32]);
pub const USER3: AccountId32 = AccountId32::new([5u8; 32]);

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	pallet_balances::GenesisConfig::<Test> { balances: vec![(ALICE, 100000), (BOB, 100000)] }
		.assimilate_storage(&mut t)
		.unwrap();
	t.into()
}
