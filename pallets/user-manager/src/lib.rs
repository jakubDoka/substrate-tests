#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

// pub mod weights;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use codec::{Decode, Encode};
	use frame_support::{pallet_prelude::*, Blake2_128Concat};
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;

	pub const USER_NAME_CAP: usize = 32;
	pub type RawUserName = [u8; USER_NAME_CAP];
	pub type CryptoHash = [u8; 32];

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::error]
	pub enum Error<T> {
		/// The caller does not have an identity on the store.
		NoIdentity,
	}

	#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Encode, Decode, TypeInfo)]
	pub struct Profile {
		pub sign: CryptoHash,
		pub enc: CryptoHash,
	}

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn list_username_to_owner)]
	pub type UsernameToOwner<T: Config> = StorageMap<
		Hasher = Blake2_128Concat,
		Key = RawUserName,
		Value = T::AccountId,
		QueryKind = OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn list_owner_to_username)]
	pub type OwnerToUsername<T: Config> = StorageMap<
		Hasher = Blake2_128Concat,
		Key = T::AccountId,
		Value = RawUserName,
		QueryKind = OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn list_identity_to_username)]
	pub type IdentityToUsername<T: Config> = StorageMap<
		Hasher = Blake2_128Concat,
		Key = CryptoHash,
		Value = RawUserName,
		QueryKind = OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn list_identities)]
	pub type Identities<T: Config> = StorageMap<
		Hasher = Blake2_128Concat,
		Key = T::AccountId,
		Value = Profile,
		QueryKind = OptionQuery,
	>;

	#[pallet::config]
	pub trait Config: frame_system::Config {}

	impl<T: Config> Pallet<T> {
		pub fn pick_name(name: RawUserName, caller: T::AccountId) -> Result<(), Error<T>> {
			OwnerToUsername::<T>::insert(caller.clone(), name);
			UsernameToOwner::<T>::insert(name, caller.clone());
			IdentityToUsername::<T>::insert(
				Identities::<T>::get(caller).ok_or(Error::<T>::NoIdentity)?.sign,
				name,
			);
			Ok(())
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1000000000000)]
		#[pallet::call_index(0)]
		pub fn register(origin: OriginFor<T>, data: Profile) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			Identities::<T>::insert(caller, data);
			Ok(())
		}

		#[pallet::weight(1000000000000)]
		#[pallet::call_index(1)]
		pub fn register_with_name(
			origin: OriginFor<T>,
			data: Profile,
			name: RawUserName,
		) -> DispatchResult {
			Self::register(origin.clone(), data)?;
			let caller = ensure_signed(origin)?;
			Self::pick_name(name, caller)?;
			Ok(())
		}

		#[pallet::weight(1000000000000)]
		#[pallet::call_index(2)]
		pub fn give_up_name(origin: OriginFor<T>, name: RawUserName) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			UsernameToOwner::<T>::take(name);
			OwnerToUsername::<T>::take(caller.clone());
			IdentityToUsername::<T>::take(
				Identities::<T>::get(caller).ok_or(Error::<T>::NoIdentity)?.sign,
			);
			Ok(())
		}

		#[pallet::weight(1000000000000)]
		#[pallet::call_index(3)]
		pub fn transfer_name(
			origin: OriginFor<T>,
			name: RawUserName,
			target: T::AccountId,
		) -> DispatchResult {
			Self::give_up_name(origin.clone(), name.clone())?;
			OwnerToUsername::<T>::insert(target.clone(), name.clone());
			UsernameToOwner::<T>::insert(name, target.clone());
			IdentityToUsername::<T>::insert(
				Identities::<T>::get(target).ok_or(Error::<T>::NoIdentity)?.sign,
				name,
			);
			Ok(())
		}
	}
}
