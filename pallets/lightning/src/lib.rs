#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

pub mod weights;

pub struct Chat;
pub struct Satelite;

// pub mod benchmarking;

// target/release/node-template \
// benchmark \
// pallet \
// --chain \
// dev \
// --pallet \
// pallet_node_staker \
// --extrinsic \
// "*" \
// --steps=50 \
// --repeat=20 \
// --wasm-execution=compiled \
// --output \
// pallets/node-staker/src/weights.rs \
// --template \
// ./benchmarking/frame-weight-template.hbs

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::{
		traits::{AccountIdConversion, IdentifyAccount, StaticLookup, Verify},
		SaturatedConversion,
	};

	/// Source type to be used in Lookup::lookup
	pub type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

	/// Type used to convert an integer into a Balance
	// #[cfg(feature = "std")]
	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct ChannelId<T: Config> {
		pub from: T::AccountId,
		pub to: T::AccountId,
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct Channel<T: Config> {
		nonce: T::Nonce,
		frozen: BalanceOf<T>,
		frozen_until: BlockNumberFor<T>,
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct ClaimContext<T: Config> {
		nonce: T::Nonce,
		amount: BalanceOf<T>,
		from: T::AccountId,
		to: T::AccountId,
	}

	#[pallet::storage]
	#[pallet::getter(fn get_channels)]
	pub type Channels<T: Config> = StorageMap<_, Blake2_128Concat, ChannelId<T>, Channel<T>>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::error]
	pub enum Error<T> {
		StillFrozen,
		NotFound,
		InvalidClaim,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ChannelCreated { from: T::AccountId, to: T::AccountId },
		ChannelClaimed { from: T::AccountId, to: T::AccountId },
	}

	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_balances::Config + pallet_timestamp::Config
	{
		/// Pallet to freee funds in.
		type PalletId: Get<PalletId>;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The staking balance.
		type Currency: Currency<Self::AccountId>;

		type Public: Clone
			+ PartialEq
			+ IdentifyAccount<AccountId = Self::AccountId>
			+ core::fmt::Debug
			+ codec::Codec
			+ Ord
			+ scale_info::TypeInfo;

		/// A matching `Signature` type.
		type Signature: Verify<Signer = Self::Public>
			+ Clone
			+ PartialEq
			+ core::fmt::Debug
			+ codec::Codec
			+ scale_info::TypeInfo;
	}

	impl<T: Config> Pallet<T> {
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1000000000000)]
		#[pallet::call_index(0)]
		pub fn create_channel(
			origin: OriginFor<T>,
			to: T::AccountId,
			amount: BalanceOf<T>,
			frozen_for: BlockNumberFor<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let channel_id = ChannelId::<T> { from: who.clone(), to: to.clone() };

			Channels::<T>::try_mutate(channel_id, |channel| -> DispatchResult {
				let channel = channel.get_or_insert_with(|| Channel::<T> {
					nonce: Default::default(),
					frozen: Default::default(),
					frozen_until: Default::default(),
				});

				ensure!(
					channel.frozen_until < <frame_system::Pallet<T>>::block_number(),
					Error::<T>::StillFrozen
				);

				T::Currency::transfer(
					&Self::account_id(),
					&who,
					channel.frozen,
					ExistenceRequirement::AllowDeath,
				)?;

				T::Currency::transfer(
					&who,
					&Self::account_id(),
					amount,
					ExistenceRequirement::AllowDeath,
				)?;

				channel.nonce = (channel.nonce.saturated_into::<u64>() + 1).saturated_into();
				channel.frozen = amount;
				channel.frozen_until = <frame_system::Pallet<T>>::block_number() + frozen_for;

				Self::deposit_event(Event::ChannelCreated { from: who.clone(), to: to.clone() });

				Ok(())
			})
		}

		#[pallet::weight(1000000000000)]
		#[pallet::call_index(1)]
		pub fn claim(
			origin: OriginFor<T>,
			from: T::AccountId,
			amount: BalanceOf<T>,
			signature: T::Signature,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let channel_id = ChannelId::<T> { from: from.clone(), to: who.clone() };

			Channels::<T>::try_mutate(channel_id, |channel| -> DispatchResult {
				let channel = channel.as_mut().ok_or(Error::<T>::NotFound)?;

				let context = ClaimContext::<T> {
					nonce: channel.nonce,
					amount,
					from: from.clone(),
					to: who.clone(),
				};

				ensure!(
					signature.verify(context.encode().as_slice(), &from),
					Error::<T>::InvalidClaim,
				);

				T::Currency::transfer(
					&Self::account_id(),
					&who,
					amount,
					ExistenceRequirement::AllowDeath,
				)?;

				T::Currency::transfer(
					&Self::account_id(),
					&from,
					channel.frozen - amount,
					ExistenceRequirement::AllowDeath,
				)?;

				channel.frozen = Default::default();
				channel.frozen_until = Default::default();

				Self::deposit_event(Event::ChannelClaimed { from: from.clone(), to: who.clone() });

				Ok(())
			})
		}
	}
}
