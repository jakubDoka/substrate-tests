#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	pub type Ed = [u8; 32];
	pub type CryptoHash = [u8; 32];

	pub type Balance = u128;
	pub const STAKE_AMOUNT: Balance = 1_000_000;
	pub const INIT_VOTE_POOL: u32 = 3;
	pub static STAKE_DURATION_MILIS: u64 = 1000 * 60 * 60 * 24 * 30;
	pub const BASE_SLASH: Balance = 2;
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

	#[derive(Encode, Decode, TypeInfo, Debug, PartialEq, Clone)]
	pub struct NodeData {
		pub sign: CryptoHash,
		pub enc: CryptoHash,
		pub id: Ed,
	}

	#[derive(Encode, Decode, TypeInfo, Debug)]
	struct Stake<T: Config> {
		owner: T::AccountId,
		amount: Balance,
		created_at: u64,
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
	pub(super) type Stakes<T: Config> =
		StorageMap<Hasher = Blake2_128Concat, Key = u32, Value = u32, QueryKind = ValueQuery>;

	// #[pallet::storage]
	// pub(super) type StakeList<T: Config> =
	// 	StorageValue<_, Vec<NodeIdentity>>;

	pub struct NodeStaker<T: Config> {
		stakes: Stakes<T>,
		// stake_list: StakeList<T>,
	}

	// pub struct NodeStaker<T: Config> {
	// 	stakes: StorageMap<_, Blake2_128Concat, NodeIdentity, Stake<T>>,
	// 	stake_list: Vec<NodeIdentity>,
	// }

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event {
		Joined { identity: Ed, addr: NodeAddress },
		AddrChanged { identity: Ed, addr: NodeAddress },
		Reclaimed { identity: Ed },
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(Weight::default())]
		#[pallet::call_index(0)]
		pub fn join(
			origin: OriginFor<T>,
			node_data: NodeData,
			addr: NodeAddress,
		) -> DispatchResult {
			todo!()
		}

		#[pallet::weight(Weight::default())]
		#[pallet::call_index(1)]
		pub fn vote(
			origin: OriginFor<T>,
			identity: NodeIdentity,
			target: NodeIdentity,
			rating: i32,
		) -> DispatchResult {
			todo!()
		}

		#[pallet::weight(Weight::default())]
		#[pallet::call_index(2)]
		pub fn list(
			origin: OriginFor<T>,
			// ) -> DispatchResult<Vec<(NodeData, NodeAddress)>> {
		) -> DispatchResult {
			todo!()
		}

		#[pallet::weight(Weight::default())]
		#[pallet::call_index(3)]
		pub fn change_addr(
			origin: OriginFor<T>,
			identity: NodeIdentity,
			addr: NodeAddress,
		) -> DispatchResult {
			todo!()
		}

		#[pallet::weight(Weight::default())]
		#[pallet::call_index(4)]
		pub fn reclaim(origin: OriginFor<T>, identity: NodeIdentity) -> DispatchResult {
			todo!()
		}
	}
}
