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
use frame_support::{traits::Currency, PalletId};
use sp_runtime::traits::AccountIdConversion;

#[frame_support::pallet]
pub mod pallet {

	pub use super::*;
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency : Currency<Self::AccountId>;

		#[pallet::constant]
		type DepositMinimum : Get<u128>;

		#[pallet::constant]
		type MaxTasks: Get<u32>;

		/// The task's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub(super) type TaskId<T> = StorageValue<_, u32, ValueQuery>;


	#[pallet::storage]
	#[pallet::getter(fn some_map)]
	pub(super) type Task<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		TaskCreated(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {

	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_task(origin: OriginFor<T>) -> DispatchResult {

			let who = ensure_signed(origin)?;

			Ok(())
		}

	}
}


impl<T: Config> Pallet<T> {

	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}

}