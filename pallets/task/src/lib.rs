#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/*
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
*/

use codec::{Decode, Encode};
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement, ReservableCurrency, WithdrawReasons},
	PalletId,
};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{AccountIdConversion, CheckedMul, SaturatedConversion, Saturating, Zero},
	ArithmeticError, Permill,
};
use sp_std::vec::Vec;

pub type CampaignIndex = Vec<u8>;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub struct Campaign<AccountId, Balance> {
	/// The account creating campaign it.
	client: AccountId,
	/// The (total) amount that should be paid if the campaign is accepted.
	value: Balance,
	/// The amount held on deposit (reserved) for making this campaign.
	bond: Balance,
}

#[frame_support::pallet]
pub mod pallet {

	pub use super::*;
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		// minimum balance that campaign should be deposit
		#[pallet::constant]
		type CampaignDepositMinimum: Get<BalanceOf<Self>>;

		// Percentage of campaign bond for check account
		#[pallet::constant]
		type CampaignDeposit: Get<Permill>;

		type RewardOrigin: EnsureOrigin<Self::Origin>;

		// Duration that user can claim their token reward
		type ClaimDuration: Get<Self::BlockNumber>;

		/// After PayoutDuration -> system pay token automatically for user
		/// when the user has not withdrawn all the money in specific campaign
		type PayoutDuration: Get<Self::BlockNumber>;
		/// The task's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);


	/// Campaign that have been made.
	#[pallet::storage]
	#[pallet::getter(fn campaigns)]
	pub type Campaigns<T: Config> = StorageMap<
		_,
		Twox64Concat,
		CampaignIndex,
		Campaign<T::AccountId, BalanceOf<T>>,
		OptionQuery,
	>;

	/// Store balance of user that system pay when user finish campaign
	#[pallet::storage]
	#[pallet::getter(fn balance_of)]
	pub type BalanceUser<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, (T::BlockNumber, BalanceOf<T>), ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New campaign.
		NewCampaign {
			campaign_index: CampaignIndex,
		},
		DepositClient {
			campaign_index: CampaignIndex,
			deposit_amount: BalanceOf<T>,
		},

		Payment {
			campaign_index: CampaignIndex,
			account: Vec<T::AccountId>,
		},
		Claim {
			user: T::AccountId,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		InsufficientBalance,
		CampaignNotExist,
		NotEnoughBalanceForUsers,
		InvalidClaim,
		UserNotReward,
		RemainingBalanceTooLow,
	}
	#[pallet::genesis_config]
	pub struct GenesisConfig;

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			//get campaign account
			let account_id = <Pallet<T>>::account_id();
			// get existential balance
			let min = T::Currency::minimum_balance();

			if T::Currency::free_balance(&account_id) < min {
				// give minimum balance for campaign account
				let _ = T::Currency::make_free_balance_be(&account_id, min);
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		///Create a campaign
		/// Should be reserved token first
		/// Store on chain
		#[pallet::weight(10_000)]
		pub fn create_campaign(
			origin: OriginFor<T>,
			campaign_index: CampaignIndex,
			#[pallet::compact] value: BalanceOf<T>,
		) -> DispatchResult {
			let client = ensure_signed(origin)?;

			let bond = (T::CampaignDepositMinimum::get()).max(T::CampaignDeposit::get() * value);
			// Reserved balance for client
			let _ =
				T::Currency::reserve(&client, bond).map_err(|_| Error::<T>::InsufficientBalance);
			Campaigns::<T>::insert(
				&campaign_index,
				Campaign { client: client.clone(), value, bond },
			);

			Self::deposit_campaign_account(&client, campaign_index.clone())?;

			Self::deposit_event(Event::NewCampaign { campaign_index });

			Ok(())
		}

		/// Reward for all users with specific campaigns
		/// Check deposit amount is enough balance to pay for all users
		#[pallet::weight(10_000)]
		pub fn payment(
			origin: OriginFor<T>,
			campaign_index: CampaignIndex,
			users: Vec<T::AccountId>,
			#[pallet::compact] amount: BalanceOf<T>,
		) -> DispatchResult {
			T::RewardOrigin::ensure_origin(origin)?;

			//Ensure this campaign is registered
			ensure!(Campaigns::<T>::contains_key(&campaign_index), Error::<T>::CampaignNotExist);

			let campaign = Campaigns::<T>::get(&campaign_index).unwrap();
			let total_amount = amount
				.checked_mul(&users.len().saturated_into())
				.ok_or(ArithmeticError::Overflow)?;
			ensure!(total_amount <= campaign.value, Error::<T>::NotEnoughBalanceForUsers);
			//let budget_remain = Self::remain_balance();
			if let Some(p) = Self::campaigns(&campaign_index) {
				let _ = T::Currency::unreserve(&p.client, p.bond);
				for user in users.iter() {
					let now = <frame_system::Pallet<T>>::block_number();
					<BalanceUser<T>>::mutate(&user, |val| {
						val.1 = val.1.saturating_add(amount);
						val.0 = now;
					});
				}

				Self::deposit_event(Event::Payment { campaign_index, account: users });
			}

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn claim(
			origin: OriginFor<T>,
			#[pallet::compact] amount: BalanceOf<T>,
			user: T::AccountId,
		) -> DispatchResult {
			T::RewardOrigin::ensure_origin(origin)?;
			let _ = Self::make_transfer(&user, amount)?;
			Self::deposit_event(Event::Claim {  user });
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	///Get campaign account
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}

	pub fn deposit_campaign_account(
		sender: &T::AccountId,
		campaign_index: CampaignIndex,
	) -> Result<(), DispatchError> {
		let campaign = Campaigns::<T>::get(&campaign_index).unwrap();
		let value = campaign.value;

		let imbalance = T::Currency::withdraw(
			&sender,
			value,
			WithdrawReasons::TRANSFER,
			ExistenceRequirement::KeepAlive,
		)?;

		T::Currency::resolve_creating(&Self::account_id(), imbalance);
		//Deposit into campaign account

		Self::deposit_event(Event::DepositClient { campaign_index, deposit_amount: value });
		Ok(())
	}

	/// Remaining balance of campaign account
	pub fn remain_balance() -> BalanceOf<T> {
		let account = Self::account_id();

		T::Currency::free_balance(&account).saturating_sub(T::Currency::minimum_balance())
	}

	fn make_transfer(
		to: &T::AccountId,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		//ensure!(CampaignPayout::<T>::get(&index).contains(to), Error::<T>::UserNotReward);
		let campaign_account = Self::account_id();
		let (when, balance_user) = Self::balance_of(&to);
		ensure!(balance_user >= amount.clone(), Error::<T>::RemainingBalanceTooLow);
		let now = <frame_system::Pallet<T>>::block_number();
		let duration = T::ClaimDuration::get();

		ensure!(now >= when.saturating_add(duration), Error::<T>::InvalidClaim);
		if balance_user == amount {
			<BalanceUser<T>>::insert(to, (now, BalanceOf::<T>::zero()));

		} else {
			log::info!("balance_user > amount");
			<BalanceUser<T>>::mutate(to, |val| {
				//val.unwrap().1 -= amount
				val.1 = val.1.saturating_sub(amount);
			});

		}

		let _ =
			T::Currency::transfer(&campaign_account, to, amount, ExistenceRequirement::KeepAlive)?;

		Ok(())
	}
}
