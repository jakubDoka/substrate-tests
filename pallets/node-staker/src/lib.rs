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
	use primitive_types::U256;
	use scale_info::prelude::vec::Vec;
	use sp_runtime::{
		traits::{AccountIdConversion, StaticLookup},
		SaturatedConversion,
	};

	/// Source type to be used in Lookup::lookup
	pub type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

	/// Type used to convert an integer into a Balance
	// #[cfg(feature = "std")]
	pub type BalanceOf<T, I> =
		<<T as Config<I>>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	pub type Hash = [u8; 32];
	pub type NodeIdentity = Hash;

	pub const STAKE_AMOUNT: u128 = 1_000_000;
	pub const INIT_VOTE_POOL: u32 = 3;
	pub static STAKE_DURATION_MILIS: u64 = 1000 * 60 * 60 * 24 * 30;
	pub const BASE_SLASH: u128 = 2;
	pub const SLASH_FACTOR: u32 = 1;

	#[pallet::pallet]
	pub struct Pallet<T, I = ()>(_);

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// The claim already exists.
		AlreadyClaimed,
		/// The user was recently slashed and has temporary protection.
		RecentlySlashed,
		/// User already voted in this vote.
		AlreadyVoted,
		/// Already joined/created stake.
		AlreadyJoined,
		/// The claim does not exist, so it cannot be revoked.
		NoSuchClaim,
		/// The stake struct does not exist.
		NoSuchStake,
		/// The claim is owned by another account, so caller can't revoke it.
		NotClaimOwner,
		/// Caller is not the owner of the stake.
		NotOwner,
		/// Caller is trying to vote for his own stake.
		CannotVoteForSelf,
		/// Stake is locked and cannot be claimed.
		StakeIsLocked,
		/// Not enough reputation to transfer.
		NotEnoughReputation,
	}

	#[derive(Clone, Copy, PartialEq, Debug, Encode, Decode, TypeInfo)]
	pub enum NodeAddress {
		Ip4([u8; 4 + 2]),
		Ip6([u8; 16 + 2]),
	}

	#[derive(Encode, Decode, TypeInfo, RuntimeDebug)]
	#[scale_info(skip_type_params(T, I))]
	pub struct Stake<T: Config<I>, I: 'static = ()> {
		owner: T::AccountId,
		staked: BalanceOf<T, I>,
		frozen_until: BlockNumberFor<T>,
		protected_until: BlockNumberFor<T>,
		enc: Hash,
	}

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn list_stakes)]
	pub type Stakes<T: Config<I>, I: 'static = ()> = StorageMap<
		Hasher = Blake2_128Concat,
		Key = NodeIdentity,
		Value = Stake<T, I>,
		QueryKind = OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_votes)]
	pub type Votes<T: Config<I>, I: 'static = ()> = StorageMap<
		Hasher = Blake2_128Concat,
		Key = NodeIdentity,
		Value = Vec<NodeIdentity>,
		QueryKind = OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_addresses)]
	pub type Addresses<T: Config<I>, I: 'static = ()> = StorageMap<
		Hasher = Blake2_128Concat,
		Key = NodeIdentity,
		Value = NodeAddress,
		QueryKind = OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		Joined { identity: NodeIdentity, addr: NodeAddress },
		AddrChanged { identity: NodeIdentity, addr: NodeAddress },
		Reclaimed { identity: NodeIdentity },
		Voted { source: NodeIdentity, target: NodeIdentity },
	}

	#[pallet::config]
	pub trait Config<I: 'static = ()>:
		frame_system::Config + pallet_balances::Config + pallet_timestamp::Config
	{
		/// The treasury's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The pallet id of the user manager to take funds from.
		#[pallet::constant]
		type BudgetPalletId: Get<PalletId>;

		/// Amount of time for which funds are frozen.
		#[pallet::constant]
		type StakePeriodBlocks: Get<BlockNumberFor<Self>>;

		#[pallet::constant]
		type ProtectionPeriodBlocks: Get<BlockNumberFor<Self>>;

		/// How much of the stake is slashed from the minimal stake.
		/// (0 == 0%, 256 == 100%)
		#[pallet::constant]
		type SlashFactor: Get<u8>;

		/// The minimum amount required to stake.
		#[pallet::constant]
		type MandatoryStake: Get<BalanceOf<Self, I>>;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The staking balance.
		type Currency: Currency<Self::AccountId>;
	}

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		pub fn budget_account_id() -> T::AccountId {
			T::BudgetPalletId::get().into_account_truncating()
		}

		pub fn ensure_stake_owner(
			identity: NodeIdentity,
			origin: OriginFor<T>,
		) -> Result<(T::AccountId, Stake<T, I>), DispatchError> {
			let owner = ensure_signed(origin)?;
			let stake = Stakes::<T, I>::get(identity).ok_or(Error::<T, I>::NoSuchStake)?;
			ensure!(stake.owner == owner, Error::<T, I>::NotOwner);
			Ok((owner, stake))
		}

		pub fn slash_amount() -> BalanceOf<T, I> {
			let base = T::MandatoryStake::get();
			let factor = T::SlashFactor::get();
			let slash = base.saturated_into::<u128>() * factor as u128 / 256;
			slash.saturated_into()
		}
	}

	#[pallet::hooks]
	impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {
		fn on_initialize(n: BlockNumberFor<T>) -> Weight {
			if n % T::StakePeriodBlocks::get() == BlockNumberFor::<T>::default() {
				let total_stake = Stakes::<T, I>::iter_values()
					.map(|s| s.staked.saturated_into::<u128>())
					.sum::<u128>();
				let budget =
					T::Currency::total_balance(&Self::budget_account_id()).saturated_into::<u128>();

				let budget_account = Self::budget_account_id();

				for stake in Stakes::<T, I>::iter_values() {
					let reward = U256::from(budget) *
						U256::from(stake.staked.saturated_into::<u128>()) /
						U256::from(total_stake);
					T::Currency::transfer(
						&budget_account,
						&stake.owner,
						u128::try_from(reward).unwrap().saturated_into(),
						ExistenceRequirement::AllowDeath,
					)
					.unwrap();
				}
			}

			Weight::default()
		}
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::weight(1000000000000)]
		#[pallet::call_index(0)]
		pub fn join(
			origin: OriginFor<T>,
			identity: NodeIdentity,
			enc: Hash,
			addr: NodeAddress,
			staked: BalanceOf<T, I>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let treasury = Self::account_id();
			let staked = staked.max(T::MandatoryStake::get());

			// transfer stake money from the caller to the treasury account of this pallet
			T::Currency::transfer(&owner, &treasury, staked, ExistenceRequirement::AllowDeath)?;

			let stake = Stake::<T, I> {
				staked,
				owner,
				frozen_until: frame_system::Pallet::<T>::block_number() +
					T::StakePeriodBlocks::get(),
				protected_until: frame_system::Pallet::<T>::block_number() +
					T::ProtectionPeriodBlocks::get(),
				enc,
			};

			// prevent caller from joining again
			ensure!(!Stakes::<T, I>::contains_key(identity), Error::<T, I>::AlreadyJoined);
			Stakes::<T, I>::insert(identity, stake);
			Addresses::<T, I>::insert(identity, addr);

			Self::deposit_event(Event::Joined { addr, identity });

			Ok(())
		}

		#[pallet::weight(1000000000000)]
		#[pallet::call_index(1)]
		pub fn vote(
			origin: OriginFor<T>,
			source: NodeIdentity,
			target: NodeIdentity,
		) -> DispatchResult {
			_ = Self::ensure_stake_owner(source, origin)?;

			ensure!(source != target, Error::<T, I>::CannotVoteForSelf);

			Votes::<T, I>::try_mutate(target, |votes_slot| {
				let votes = votes_slot.get_or_insert_with(Default::default);
				ensure!(!votes.contains(&source), Error::<T, I>::AlreadyClaimed);

				if votes.len() == Stakes::<T, I>::iter().count() / 2 {
					votes_slot.take();
					let to_burn = Self::slash_amount();
					Stakes::<T, I>::try_mutate(target, |stake_slot| {
						let stake = stake_slot.as_mut().ok_or(Error::<T, I>::NoSuchStake)?;
						ensure!(
							stake.protected_until <= frame_system::Pallet::<T>::block_number(),
							Error::<T, I>::RecentlySlashed
						);
						stake.staked -= to_burn;
						if stake.staked < T::MandatoryStake::get() {
							stake_slot.take();
							Self::deposit_event(Event::Reclaimed { identity: target });
						}
						DispatchResult::Ok(())
					})?;
				} else {
					votes.push(source);
				}

				DispatchResult::Ok(())
			})?;

			Self::deposit_event(Event::Voted { source, target });

			Ok(())
		}

		#[pallet::weight(1000000000000)]
		#[pallet::call_index(3)]
		pub fn change_addr(
			origin: OriginFor<T>,
			identity: NodeIdentity,
			addr: NodeAddress,
		) -> DispatchResult {
			Self::ensure_stake_owner(identity, origin)?;
			Addresses::<T, I>::insert(identity, addr);
			Self::deposit_event(Event::AddrChanged { identity, addr });
			Ok(())
		}

		#[pallet::weight(1000000000000)]
		#[pallet::call_index(4)]
		pub fn reclaim(origin: OriginFor<T>, identity: NodeIdentity) -> DispatchResult {
			let (owner, stake) = Self::ensure_stake_owner(identity, origin)?;
			ensure!(
				stake.frozen_until <= frame_system::Pallet::<T>::block_number(),
				Error::<T, I>::StakeIsLocked
			);

			Stakes::<T, I>::remove(identity);

			// let balance = T::Currency::free_balance(&receiver);
			// print!("current balance: {balance:?}");

			let treasury = Self::account_id();
			T::Currency::transfer(
				&treasury,
				&owner,
				stake.staked,
				ExistenceRequirement::AllowDeath,
			)?;

			Stakes::<T, I>::remove(identity);
			Addresses::<T, I>::remove(identity);

			Self::deposit_event(Event::Reclaimed { identity });

			Ok(())
		}
	}
}
