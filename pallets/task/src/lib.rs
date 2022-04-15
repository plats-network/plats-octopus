#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

/*
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
*/

use codec::{Decode, Encode};
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ReservableCurrency, Imbalance, OnUnbalanced},
	PalletId,
};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{AccountIdConversion, Saturating, CheckedMul, SaturatedConversion},
	Permill,ArithmeticError,
};
use sp_std::vec::Vec;

pub type CampaignIndex = u32;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub type PositiveImbalanceOf<T> = <<T as Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::PositiveImbalance;


pub type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;


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

		// Percentage of campaign deposit
		#[pallet::constant]
		type CampaignDeposit: Get<Permill>;

		type RejectOrigin: EnsureOrigin<Self::Origin>;

		type ApprovalOrigin: EnsureOrigin<Self::Origin>;

		type RewardOrigin: EnsureOrigin<Self::Origin>;

		type CampaignDuration: Get<Self::BlockNumber>;

		/// Handler for the unbalanced decrease when slashing for a approval campaign.
		type SlashDeposit: OnUnbalanced<NegativeImbalanceOf<Self>>;

		/// The task's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn campaign_count)]
	pub(crate) type CampaignCount<T> = StorageValue<_, CampaignIndex, ValueQuery>;

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

	#[pallet::storage]
	#[pallet::getter(fn approvals)]
	pub type ApprovalCampaigns<T> = StorageValue<_, Vec<CampaignIndex>, ValueQuery>;

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

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New campaign.
		NewCampaign {
			campaign_index: CampaignIndex,
		},
		RejectCampaign {
			campaign_index: CampaignIndex,
			value: BalanceOf<T>,
		},
		ApproveCampaign {
			campaign_index: CampaignIndex,
		},
		RemainingBudget{ remaining_budget: BalanceOf<T>,},

		Deposit {
			amount: BalanceOf<T>,
		},
		SlashDepositClient { campaign_index : CampaignIndex, slashed: BalanceOf<T>},

		Rewarded {
			campaign_index: CampaignIndex,
			award: BalanceOf<T>,
			account : Vec<T::AccountId>,

		}

	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		InsufficientBalance,
		CampaignNotExist,
		NotApprovalCampaign,
		NotEnoughBalanceForUsers
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		///Create a campaign
		/// Should be reserved token first
		/// Store on chain
		#[pallet::weight(10_000)]
		pub fn create_campaign(origin: OriginFor<T>, value: BalanceOf<T>) -> DispatchResult {
			let client = ensure_signed(origin)?;

			let bond = (T::CampaignDepositMinimum::get()).max(T::CampaignDeposit::get() * value);

			// Reserved balance for client
			let _ =
				T::Currency::reserve(&client, bond).map_err(|_| Error::<T>::InsufficientBalance);

			let count = Self::campaign_count();

			CampaignCount::<T>::put(count + 1);
			Campaigns::<T>::insert(count, Campaign { client, value, bond });
			Self::deposit_event(Event::NewCampaign { campaign_index: count });

			Ok(())
		}

		/// Reject creating campaign from client
		/// Only root can execute this extrinsic call
		/// In this case will unreseved bond amount not slash bond amount

		#[pallet::weight(10_000)]
		pub fn reject_campaign(
			origin: OriginFor<T>,
			campaign_index: CampaignIndex,
		) -> DispatchResult {
			T::RejectOrigin::ensure_origin(origin)?;

			let campaign =
				Campaigns::<T>::take(&campaign_index).ok_or(Error::<T>::CampaignNotExist)?;

			let value = campaign.value;
			let _ = T::Currency::unreserve(&campaign.client, value);

			Self::deposit_event(Event::RejectCampaign { campaign_index, value });

			Ok(())
		}
		/// Approve campaign
		/// Only root can execute this extrinsic call
		/// Will be deposit value amount into campaign account
		#[pallet::weight(10_000)]
		pub fn approve_campaign(
			origin: OriginFor<T>,
			campaign_index: CampaignIndex,
		) -> DispatchResult {
			T::ApprovalOrigin::ensure_origin(origin)?;

			ensure!(Campaigns::<T>::contains_key(campaign_index), Error::<T>::CampaignNotExist);
			ApprovalCampaigns::<T>::append(campaign_index);
			

			Self::deposit_campaign_account(campaign_index);
			Self::deposit_event(Event::ApproveCampaign { campaign_index });
			Ok(())
		}

		/// Reward for all users with specific campaigns
		/// Check deposit amount is enough balance to pay for all users
		#[pallet::weight(10_000)]
		pub fn reward(
			origin: OriginFor<T>,
			campaign_index: CampaignIndex,
			users: Vec<T::AccountId>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			T::RewardOrigin::ensure_origin(origin)?;

			//Ensure this campaign is registered
			ensure!(Campaigns::<T>::contains_key(campaign_index), Error::<T>::CampaignNotExist);

			let approval = ApprovalCampaigns::<T>::get();
			// Ensure this campaign is approved
			ensure!(approval.contains(&campaign_index), Error::<T>::NotApprovalCampaign);

			let campaign = Campaigns::<T>::get(&campaign_index).unwrap();
			let total_amount = amount.checked_mul(&users.len().saturated_into()).ok_or(ArithmeticError::Overflow)?;
			ensure!(total_amount < campaign.value, Error::<T>::NotEnoughBalanceForUsers);

			let mut budget_remain = Self::remain_balance();

			let mut imbalance = <PositiveImbalanceOf<T>>::zero();

			for index in approval.into_iter() {
				if let Some(p) = Self::campaigns(index) {
					if p.value <= budget_remain {
						budget_remain -= p.value;
						Campaigns::<T>::remove(index);

						let unreserve = T::Currency::unreserve(&p.client, p.bond);
						log::info!("Unreserved Balance:{:?}", unreserve);
						for user in users.iter(){
							imbalance.subsume(T::Currency::deposit_creating(&user, amount));
						}

						Self::deposit_event(Event::Rewarded {
							campaign_index: index,
							award: amount,
							account: users.clone(),
						});
					}
				}
			};

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}

	pub fn deposit_campaign_account(campaign_index: CampaignIndex){

		let campaign = Campaigns::<T>::get(campaign_index).unwrap();
		let value = campaign.value;
		let imbalance = T::Currency::slash(&campaign.client, value).0;
		T::SlashDeposit::on_unbalanced(imbalance);
		
		Self::deposit_event(Event::SlashDepositClient { campaign_index, slashed: value });


	}


	pub fn remain_balance() -> BalanceOf<T> {
		let account = Self::account_id();

		T::Currency::free_balance(&account).saturating_sub(T::Currency::minimum_balance())
	}
}


impl<T: Config> OnUnbalanced<NegativeImbalanceOf<T>> for Pallet<T> {
	fn on_nonzero_unbalanced(imbalance_amount: NegativeImbalanceOf<T>) {
		let amount = imbalance_amount.peek();

		// Must resolve into existing but better to be safe.
		let _ = T::Currency::resolve_creating(&Self::account_id(), imbalance_amount);

		Self::deposit_event(Event::Deposit { amount });
	}
}