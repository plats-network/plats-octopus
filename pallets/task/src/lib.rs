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

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use frame_support::{traits::{Currency, ReservableCurrency}, PalletId};
use sp_runtime::traits::AccountIdConversion;
use scale_info::TypeInfo;
use codec::{Encode, Decode};
use sp_std::vec::Vec;

pub type CampaignIndex = u32;

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
		type Currency : Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		#[pallet::constant]
		type DepositMinimum : Get<BalanceOf<Self>>;

		type RejectOrigin : EnsureOrigin<Self::Origin>;

		type ApprovalOrigin: EnsureOrigin<Self::Origin>;

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
	pub type ApprovalCampaigns<T> =
		StorageValue<_, Vec<CampaignIndex>, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig;

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self
		}
	}

	#[pallet::genesis_build]
	impl<T:Config> GenesisBuild<T> for GenesisConfig{
		fn build(&self){

			//get campaign account
			let account_id = <Pallet<T>>::account_id();
			// get existential balance
			let min = T::Currency::minimum_balance();

			if T::Currency::free_balance(&account_id)< min {

				// give minimum balance for campaign account
				let _ = T::Currency::make_free_balance_be(&account_id, min);
			}
		} 
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New campaign.
		NewCampaign { campaign_index: CampaignIndex },
		RejectCampaign { campaign_index: CampaignIndex, value:BalanceOf<T> },
		ApproveCampaign { campaign_index: CampaignIndex},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		InsufficientBalance,
		CampaignNotExist,

	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		///Create a campaign
		/// Should be reserved token first
		/// Store on chain 
		#[pallet::weight(10_000)]
		pub fn create_campaign(origin: OriginFor<T>, 
			#[pallet::compact] value: BalanceOf<T> ) -> DispatchResult {

			let client = ensure_signed(origin)?;

			let bond = (T::DepositMinimum::get()).max(value);
			
			// Reserved balance for client
			let _ = T::Currency::reserve(&client, bond).map_err(|_| Error::<T>::InsufficientBalance);

			let count = Self::campaign_count();

			CampaignCount::<T>::put(count+1);
			Campaigns::<T>::insert(count, Campaign {client, value, bond});
			Self::deposit_event(Event::NewCampaign{campaign_index: count});

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn reject_campaign(origin: OriginFor<T>, campaign_index: CampaignIndex)-> DispatchResult {
			
			T::RejectOrigin::ensure_origin(origin)?;

			let campaign = Campaigns::<T>::take(&campaign_index).ok_or(Error::<T>::CampaignNotExist)?;

			let value = campaign.value;
			let _ = T::Currency::unreserve(&campaign.client, value);

			Self::deposit_event(Event::RejectCampaign{campaign_index, value });

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn approve_campaign(
			origin: OriginFor<T>,
			#[pallet::compact] campaign_index: CampaignIndex,
		) -> DispatchResult {
			T::ApprovalOrigin::ensure_origin(origin)?;

			ensure!(Campaigns::<T>::contains_key(campaign_index), Error::<T>::CampaignNotExist);
			ApprovalCampaigns::<T>::append(campaign_index);
			Self::deposit_event(Event::ApproveCampaign{campaign_index});
			Ok(())
		}




	}
}


impl<T: Config> Pallet<T> {

	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}

}