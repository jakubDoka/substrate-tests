#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::sp_runtime::SaturatedConversion;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use codec::{Codec, MaxEncodedLen};
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::{traits::AtLeast32BitUnsigned, FixedPointOperand},
		traits::{Currency, ExistenceRequirement},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{AccountIdConversion, StaticLookup};

	/// Source type to be used in Lookup::lookup
	type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

	/// Type used to convert an integer into a Balance
	#[cfg(feature = "std")]
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	pub type Ed = [u8; 32];
	pub type CryptoHash = [u8; 32];

	// pub type Balance = u128;
	pub const STAKE_AMOUNT: u128 = 1_000_000;
	pub const INIT_VOTE_POOL: u32 = 3;
	pub static STAKE_DURATION_MILIS: u64 = 1000 * 60 * 60 * 24 * 30;
	pub const BASE_SLASH: u128 = 2;
	pub const SLASH_FACTOR: u32 = 1;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[derive(Clone, Copy, PartialEq, Debug, Encode, Decode, TypeInfo)]
	pub enum NodeAddress {
		Ip4([u8; 4 + 2]),
		Ip6([u8; 16 + 2]),
	}

	#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Encode, Decode, TypeInfo)]
	pub struct NodeIdentity {
		pub sign: CryptoHash,
		pub enc: CryptoHash,
	}

	impl MaxEncodedLen for NodeIdentity {
		fn max_encoded_len() -> usize {
			14 + 2 + 16 + 2
		}
	}

	#[derive(Encode, Decode, TypeInfo, RuntimeDebug)]
	#[scale_info(skip_type_params(T))]
	pub struct Stake<T: Config> {
		owner: T::AccountId,
		amount: BalanceOf<T>,
		created_at: <T as pallet_timestamp::Config>::Moment,
		votes: Votes,
		id: Ed,
		addr: NodeAddress,
	}

	#[derive(Encode, Decode, TypeInfo, Debug)]
	struct Votes {
		pool: u32,
		rating: u32,
	}

	impl Default for Votes {
		fn default() -> Self {
			Self { pool: INIT_VOTE_POOL, rating: 0 }
		}
	}

	#[pallet::storage]
	#[pallet::unbounded]
	pub type Stakes<T: Config> = StorageMap<
		Hasher = Blake2_128Concat,
		Key = NodeIdentity,
		Value = Stake<T>,
		QueryKind = OptionQuery,
	>;

	#[derive(Encode, Decode, TypeInfo, Debug, PartialEq, Clone)]
	pub struct NodeData {
		pub sign: CryptoHash,
		pub enc: CryptoHash,
		pub id: Ed,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event {
		Joined { identity: Ed, addr: NodeAddress },
		AddrChanged { identity: Ed, addr: NodeAddress },
		Reclaimed { identity: Ed },
	}

	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_balances::Config + pallet_timestamp::Config
	{
		/// The treasury's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Weight information for extrinsics in this pallet.
		// type WeightInfo: WeightInfo;

		/// The overarching event type.
		type RuntimeEvent: From<Event> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The staking balance.
		type Currency: Currency<Self::AccountId>;

		/// Balance used to make transfers.
		type Balance: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Codec
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TypeInfo
			+ FixedPointOperand;
	}

	impl<T: Config> Pallet<T> {
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// #[pallet::weight(Weight::default())]
		#[pallet::weight(100000)]
		#[pallet::call_index(0)]
		pub fn join(
			origin: OriginFor<T>,
			node_data: NodeData,
			addr: NodeAddress,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let amount = STAKE_AMOUNT;
			let amount: BalanceOf<T> = amount.saturated_into::<BalanceOf<T>>();
			let treasury = Self::account_id();

			// transfer stake money from the caller to the treasury account of this pallet
			T::Currency::transfer(&sender, &treasury, amount, ExistenceRequirement::AllowDeath)?;

			let stake = Stake::<T> {
				amount,
				owner: sender,
				id: node_data.id,
				votes: Votes::default(),
				created_at: pallet_timestamp::Pallet::<T>::get(),
				addr,
			};

			let id = NodeIdentity { sign: node_data.sign, enc: node_data.enc };

			Stakes::<T>::insert(id, stake);

			Self::deposit_event(Event::Joined { addr, identity: node_data.id });

			Ok(())
		}

		// #[pallet::weight(Weight::default())]
		// #[pallet::call_index(1)]
		// pub fn vote(
		// 	origin: OriginFor<T>,
		// 	identity: NodeIdentity,
		// 	target: NodeIdentity,
		// 	rating: i32,
		// ) -> DispatchResult {
		// 	todo!()
		// }
		//
		// #[pallet::weight(Weight::default())]
		// #[pallet::call_index(2)]
		// pub fn list(
		// 	origin: OriginFor<T>,
		// 	// ) -> DispatchResult<Vec<(NodeData, NodeAddress)>> {
		// ) -> DispatchResult {
		// 	todo!()
		// }
		//
		// #[pallet::weight(Weight::default())]
		// #[pallet::call_index(3)]
		// pub fn change_addr(
		// 	origin: OriginFor<T>,
		// 	identity: NodeIdentity,
		// 	addr: NodeAddress,
		// ) -> DispatchResult {
		// 	todo!()
		// }
		//
		// #[pallet::weight(Weight::default())]
		// #[pallet::call_index(4)]
		// pub fn reclaim(origin: OriginFor<T>, identity: NodeIdentity) -> DispatchResult {
		// 	todo!()
		// }
	}
}
