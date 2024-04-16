#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

pub mod weights;

pub struct Chat;
pub struct Satelite;

pub trait RewardCap {
	fn get(node_count: u32) -> u128;
}

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

	pub type ReputationOf<T, I> = BalanceOf<T, I>;

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
		/// Not enough votes in pool
		NotEnoughVotes,
		/// Too many votes
		TooManyVotes,
		/// The claim already exists.
		AlreadyClaimed,
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
		reputation: ReputationOf<T, I>,
		frozen_until: BlockNumberFor<T>,
		addr: NodeAddress,
		enc: Hash,
	}

	#[derive(Encode, Decode, TypeInfo, RuntimeDebug)]
	#[scale_info(skip_type_params(T, I))]
	pub struct FrozenReputation<T: Config<I>, I: 'static = ()> {
		reputation: ReputationOf<T, I>,
		frozen_until: BlockNumberFor<T>,
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
	#[pallet::getter(fn list_frozen_reputations)]
	pub type ReputationFreezes<T: Config<I>, I: 'static = ()> = StorageMap<
		Hasher = Blake2_128Concat,
		Key = NodeIdentity,
		Value = FrozenReputation<T, I>,
		QueryKind = OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		Joined { identity: Hash, addr: NodeAddress },
		AddrChanged { identity: Hash, addr: NodeAddress },
		Reclaimed { identity: Hash },
		ReputationTransfered { from: Hash, to: Hash, amount: ReputationOf<T, I> },
		ReputationBurned { source: Hash, target: Hash, amount: ReputationOf<T, I> },
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

		/// How much of the stake is node rewarded each period. (0 == 0%, 256 == 100%)
		#[pallet::constant]
		type ReputationIncomeFactor: Get<u8>;

		/// If reputatuion falls bellow this factor of stake, the stake is slashed.
		/// (0 == 0%, 256 == 100%)
		#[pallet::constant]
		type SlashAreaFactor: Get<u8>;

		/// How much of the stake is slashed when reputation falls into the slash area.
		/// (0 == 0%, 256 == 100%)
		#[pallet::constant]
		type SlashFactor: Get<u8>;

		/// Percentage of the total budget that will be distributed based of reputation.
		/// (total_budget * ReputationRewardFraction / 256) * reputation / total_reputation
		#[pallet::constant]
		type ReputationRewardFactor: Get<u8>;

		/// How often is reputation income distributed.
		#[pallet::constant]
		type ReputationIncomePeriodBlocks: Get<BlockNumberFor<Self>>;

		/// Function to calculate collective reward cap. Without this nodes have no motivation to
		/// allow new nodes to join since split of revenue between nodes motivates the nodes to not
		/// let anyone in, no matter the amount of users in the network. Cap should increase as more
		/// nodes join the network, if the total user revenue exceeds the cap, overflow should be
		/// sent to the treasury.
		type RevardCap: RewardCap;

		/// Weight information for extrinsics in this pallet.
		// type WeightInfo: WeightInfo;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The staking balance.
		type Currency: Currency<Self::AccountId>;

		// type StakeDurationMilis: pallet_timestamp::Config::Moment;
	}

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		pub fn budget_account_id() -> T::AccountId {
			T::BudgetPalletId::get().into_account_truncating()
		}

		pub fn calc_reputation_income(staked: BalanceOf<T, I>) -> ReputationOf<T, I> {
			let staked = staked.saturated_into::<u128>();
			let reputation = staked * T::ReputationIncomeFactor::get() as u128 / 256;
			reputation.saturated_into()
		}

		pub fn calc_initial_reputation(staked: BalanceOf<T, I>) -> ReputationOf<T, I> {
			let staked = staked.saturated_into::<u128>();
			let reputation = staked * T::SlashAreaFactor::get() as u128 / 256;
			reputation.saturated_into()
		}

		pub fn is_in_slash_area(staked: BalanceOf<T, I>, reputation: ReputationOf<T, I>) -> bool {
			let stake = staked.saturated_into::<u128>();
			let reputation = reputation.saturated_into::<u128>();
			let slash_area = stake * T::SlashAreaFactor::get() as u128 / 256;
			reputation < slash_area
		}

		//	pub fn calc_reward(reputation: ReputationOf<T, I>) -> (BalanceOf<T, I>, BalanceOf<T, I>)
		// { 		let cap = T::RevardCap::get(Stakes::<T, I>::iter().count() as u32);
		//		let total_budget = U256::from(
		//			T::Currency::total_balance(&Self::budget_account_id()).saturated_into::<u128>(),
		//		);
		//		let stake_count = Stakes::<T, I>::iter().count() as u128;

		//		let reputation_budget =
		//			total_budget * U256::from(T::ReputationRewardFraction::get()) / U256::from(256);
		//		let base_budget = total_budget - reputation_budget;

		//		let base_reward = base_budget / stake_count;
		//		let reputation_reward = reputation_budget *
		//			U256::from(reputation.saturated_into::<u128>()) /
		//			U256::from(ReputationVolume::<T, I>::get().saturated_into::<u128>());
		//		let reward = u128::try_from(base_reward + reputation_reward).unwrap();
		//		let capped_reward = reward.min(cap);

		//		(capped_reward.saturated_into(), (reward - capped_reward).saturated_into())
		//	}
	}

	#[pallet::hooks]
	impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {
		fn on_initialize(n: BlockNumberFor<T>) -> Weight {
			if n % T::ReputationIncomePeriodBlocks::get() == BlockNumberFor::<T>::default() {
				for (identity, mut stake) in Stakes::<T, I>::iter() {
					if Self::is_in_slash_area(stake.staked, stake.reputation) {
						let new_stake = stake.staked.saturated_into::<u128>() *
							T::SlashFactor::get() as u128 /
							256;
						stake.staked = new_stake.saturated_into();
					}

					let income = Self::calc_reputation_income(stake.staked);
					stake.reputation += income;
					Stakes::<T, I>::insert(identity, &stake);
				}
			}

			if n % T::StakePeriodBlocks::get() == BlockNumberFor::<T>::default() {
				let mut total_reputation = 0;
				let stake_count = Stakes::<T, I>::iter_values()
					.inspect(|stake| total_reputation += stake.reputation.saturated_into::<u128>())
					.count() as u128;
				let cap = T::RevardCap::get(stake_count as u32);
				let mut full =
					T::Currency::total_balance(&Self::budget_account_id()).saturated_into::<u128>();
				let budget = full.min(cap);
				let reward_budget = U256::from(budget) *
					U256::from(T::ReputationRewardFactor::get()) /
					U256::from(256);
				let base_budget = budget - reward_budget.saturated_into::<u128>();
				let base_reward = base_budget / stake_count;

				let budget_account = Self::budget_account_id();
				let treasury = Self::account_id();

				for stake in Stakes::<T, I>::iter_values() {
					let reputation_reward = U256::from(reward_budget) *
						U256::from(stake.reputation.saturated_into::<u128>()) /
						U256::from(total_reputation);
					let reward = base_reward + u128::try_from(reputation_reward).unwrap();
					T::Currency::transfer(
						&budget_account,
						&stake.owner,
						reward.saturated_into(),
						ExistenceRequirement::AllowDeath,
					)
					.unwrap();
					full -= reward;
				}

				T::Currency::transfer(
					&budget_account,
					&treasury,
					full.saturated_into(),
					ExistenceRequirement::AllowDeath,
				)
				.unwrap();
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

			// transfer stake money from the caller to the treasury account of this pallet
			T::Currency::transfer(&owner, &treasury, staked, ExistenceRequirement::AllowDeath)?;

			let stake = Stake::<T, I> {
				staked,
				owner,
				reputation: Self::calc_initial_reputation(staked),
				frozen_until: frame_system::Pallet::<T>::block_number() +
					T::StakePeriodBlocks::get(),
				addr,
				enc,
			};

			// prevent caller from joining again
			ensure!(!Stakes::<T, I>::contains_key(identity), Error::<T, I>::AlreadyJoined);
			Stakes::<T, I>::insert(identity, stake);

			Self::deposit_event(Event::Joined { addr, identity });

			Ok(())
		}

		#[pallet::weight(1000000000000)]
		#[pallet::call_index(1)]
		pub fn transfer_reputation(
			origin: OriginFor<T>,
			from: NodeIdentity,
			to: NodeIdentity,
			amount: ReputationOf<T, I>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let mut from_stake = Stakes::<T, I>::get(from).ok_or(Error::<T, I>::NoSuchStake)?;
			ensure!(from_stake.owner == sender, Error::<T, I>::NotOwner);
			ensure!(from_stake.reputation >= amount, Error::<T, I>::NotEnoughReputation);

			let mut to_stake = Stakes::<T, I>::get(to).ok_or(Error::<T, I>::NoSuchStake)?;
			to_stake.reputation += amount;
			from_stake.reputation -= amount;

			Stakes::<T, I>::insert(from, &from_stake);
			Stakes::<T, I>::insert(to, &to_stake);

			Self::deposit_event(Event::ReputationTransfered { from, to, amount });

			Ok(())
		}

		#[pallet::weight(1000000000000)]
		#[pallet::call_index(2)]
		pub fn burn_reputation(
			origin: OriginFor<T>,
			source: NodeIdentity,
			target: NodeIdentity,
			amount: ReputationOf<T, I>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let mut source_stake = Stakes::<T, I>::get(source).ok_or(Error::<T, I>::NoSuchStake)?;
			ensure!(source_stake.owner == sender, Error::<T, I>::NotOwner);
			ensure!(source_stake.reputation >= amount, Error::<T, I>::NotEnoughReputation);

			let mut target_stake = Stakes::<T, I>::get(target).ok_or(Error::<T, I>::NoSuchStake)?;
			target_stake.reputation -= amount;
			source_stake.reputation -= amount;

			ReputationFreezes::<T, I>::mutate(source, |frozen| {
				let frosen = frozen.get_or_insert(FrozenReputation::<T, I> {
					reputation: ReputationOf::<T, I>::default(),
					frozen_until: BlockNumberFor::<T>::default(),
				});

				frosen.reputation += amount;
				frosen.frozen_until =
					frame_system::Pallet::<T>::block_number() + T::StakePeriodBlocks::get();
			});

			Stakes::<T, I>::insert(source, &source_stake);
			Stakes::<T, I>::insert(target, &target_stake);

			Self::deposit_event(Event::ReputationBurned { source, target, amount });

			Ok(())
		}

		#[pallet::weight(1000000000000)]
		#[pallet::call_index(3)]
		pub fn change_addr(
			origin: OriginFor<T>,
			identity: NodeIdentity,
			addr: NodeAddress,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let mut stake = Stakes::<T, I>::get(identity).ok_or(Error::<T, I>::NoSuchStake)?;
			ensure!(stake.owner == sender, Error::<T, I>::NotOwner);

			stake.addr = addr;
			Stakes::<T, I>::insert(identity, &stake);
			Self::deposit_event(Event::AddrChanged { identity, addr });

			Ok(())
		}

		#[pallet::weight(1000000000000)]
		#[pallet::call_index(4)]
		pub fn reclaim(origin: OriginFor<T>, identity: NodeIdentity) -> DispatchResult {
			let receiver = ensure_signed(origin)?;
			let stake = Stakes::<T, I>::get(identity).ok_or(Error::<T, I>::NoSuchStake)?;
			ensure!(stake.owner == receiver, Error::<T, I>::NotOwner);
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
				&receiver,
				stake.staked,
				ExistenceRequirement::AllowDeath,
			)?;

			Stakes::<T, I>::remove(identity);

			Self::deposit_event(Event::Reclaimed { identity });

			Ok(())
		}

		#[pallet::weight(1000000000000)]
		#[pallet::call_index(5)]
		pub fn reclaim_reputation(origin: OriginFor<T>, identity: NodeIdentity) -> DispatchResult {
			let receiver = ensure_signed(origin)?;
			let stake = Stakes::<T, I>::get(identity).ok_or(Error::<T, I>::NoSuchStake)?;
			ensure!(stake.owner == receiver, Error::<T, I>::NotOwner);
			ensure!(
				stake.frozen_until <= frame_system::Pallet::<T>::block_number(),
				Error::<T, I>::StakeIsLocked
			);

			let frozen =
				ReputationFreezes::<T, I>::get(identity).ok_or(Error::<T, I>::NoSuchStake)?;
			ensure!(
				frozen.frozen_until <= frame_system::Pallet::<T>::block_number(),
				Error::<T, I>::StakeIsLocked
			);
			T::Currency::transfer(
				&Self::account_id(),
				&receiver,
				frozen.reputation,
				ExistenceRequirement::AllowDeath,
			)?;

			Ok(())
		}
	}
}
