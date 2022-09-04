#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

use frame_support::inherent::Vec;
use frame_support::pallet_prelude::*;
use frame_support::sp_runtime::traits::{AccountIdConversion, Saturating};
use frame_support::sp_runtime::SaturatedConversion;
use frame_support::traits::{Currency, ExistenceRequirement::KeepAlive, Get, ReservableCurrency};
use frame_support::PalletId;
use frame_system::pallet_prelude::*;

pub type Id = u32;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	#[derive(Debug, Decode, Encode, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Room<T: Config> {
		creator: T::AccountId,
		id_room: Id,
		start: T::BlockNumber,
		length: T::BlockNumber,
		delay: T::BlockNumber,
		total_value: BalanceOf<T>,
		is_started: bool,
		is_ended: bool,
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		// type RoomTime: Time;

		/// The Room's pallet id
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The currency trait.
		type Currency: ReservableCurrency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(crate) type RoomId<T> = StorageValue<_, Id, ValueQuery>;

	// #[pallet::storage]
	// pub(crate) type NumberUser<T> = StorageValue<_, u32, ValueQuery>;

	// Room index and information of room
	#[pallet::storage]
	#[pallet::getter(fn get_room)]
	pub(super) type Rooms<T: Config> = StorageMap<_, Blake2_128Concat, Id, Room<T>, OptionQuery>;

	// Users who have joined in room (id_room, balance deposit)
	#[pallet::storage]
	#[pallet::getter(fn get_user_in_room)]
	pub(super) type UserInRoom<T: Config> =
		StorageMap<_, Blake2_128Concat, Id, Vec<T::AccountId>, OptionQuery>;

	// Value which users have deposited
	#[pallet::storage]
	#[pallet::getter(fn get_deposit)]
	pub(super) type UserDeposit<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, OptionQuery>;

	// Result data of user in room
	#[pallet::storage]
	#[pallet::getter(fn get_user_result)]
	pub(super) type UserResult<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u32, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Create new room (id_room, who)
		CreateRoom { id_room: u32, creator: T::AccountId },
		/// Who join in room (id_room, who, amount deposited)
		JoinRoom { id_room: u32, user: T::AccountId, deposit_amount: u32 },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Room doesn't exist
		RoomNotExist,
		/// Maximum participants
		RoomOverload,
		/// Room started already
		RoomAlreadyStarted,
		/// Room ended already
		RoomAlreadyEnded,
		/// User has already joined
		UserAlreadyJoined,
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub results: Vec<(T::AccountId, u32)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> GenesisConfig<T> {
			GenesisConfig { results: Vec::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for (account, result) in self.results.iter() {
				UserResult::<T>::insert(account, result);
			}
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: T::BlockNumber) -> u64 {
			let id_room = Rooms::<T>::iter_keys().collect::<Vec<u32>>();
			for id in id_room {
				let mut room = Rooms::<T>::get(id).unwrap();
				let payout_block =
					room.start.saturating_add(room.length).saturating_add(room.delay);
				let vec_users = UserInRoom::<T>::get(id).unwrap();
				if payout_block <= n && room.is_started && !room.is_ended {
					room.is_ended = true;
					for user in vec_users {
						let result = UserResult::<T>::get(user.clone()).unwrap();
						let deposit = UserDeposit::<T>::get(user.clone()).unwrap();
						// Rewards depend on result of users: factor(BEAT token, bonus token)
						let reward: (u128, u128) = match result {
							x if x > 95 && x <= 100 => (100, 100),
							x if x > 85 && x <= 95 => (90, 90),
							x if x > 75 && x <= 85 => (80, 80),
							x if x > 65 && x <= 75 => (70, 70),
							x if x > 55 && x <= 65 => (60, 60),
							x if x > 45 && x <= 55 => (50, 50),
							_ => (0, 0),
						};
						let reward_beat = deposit.saturated_into::<u128>() * reward.0 / 100;
						let res = T::Currency::transfer(
							&Self::account_id(),
							&user,
							reward_beat.saturated_into(),
							KeepAlive,
						);
						debug_assert!(res.is_ok());
						UserDeposit::<T>::remove(user.clone());
					}
					<Rooms<T>>::insert(id, room);
				}
			}
			0
		}
	}
	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create_new_room(
			origin: OriginFor<T>,
			length: T::BlockNumber,
			delay: T::BlockNumber,
			deposit: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Update room id
			let mut current_id_room = <RoomId<T>>::get();
			current_id_room += 1;
			<RoomId<T>>::put(current_id_room);

			// let start = T::RoomTime::now().saturated_into::<u64>();
			let start = frame_system::Pallet::<T>::block_number();

			// let end = start + num_days * 86_400;
			// let mut current_number = <NumberUser<T>>::get();
			// current_number += 1;
			// <NumberUser<T>>::put(current_number);

			// let mut users_joined = UserInRoom::<T>::take(&who).unwrap_or_default();
			// users_joined.push(who.clone());

			let room = Room::<T> {
				creator: who.clone(),
				id_room: current_id_room,
				start,
				length,
				delay,
				total_value: deposit.saturated_into(),
				is_started: false,
				is_ended: false,
			};
			let deposit = deposit;
			let mut vec_users = Vec::new();
			vec_users.push(who.clone());

			T::Currency::transfer(&who, &Self::account_id(), deposit.saturated_into(), KeepAlive)?;

			<UserInRoom<T>>::insert(current_id_room, vec_users);
			<UserDeposit<T>>::insert(who.clone(), deposit);
			<Rooms<T>>::insert(current_id_room, room);

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn join_room(
			origin: OriginFor<T>,
			id_room: u32,
			deposit: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Check if the room exists or not
			ensure!(id_room <= <RoomId<T>>::get(), Error::<T>::RoomNotExist);

			// Check if the user has not already join any room
			ensure!(!UserDeposit::<T>::contains_key(&who), Error::<T>::UserAlreadyJoined);

			let mut room = Rooms::<T>::get(id_room).unwrap();

			// Check if the room has started/ ended yet
			ensure!(!room.is_started, Error::<T>::RoomAlreadyStarted);
			ensure!(!room.is_ended, Error::<T>::RoomAlreadyEnded);

			room.total_value += deposit.saturated_into();

			// Update infomation
			let mut vec_users = UserInRoom::<T>::get(id_room).unwrap();
			vec_users.push(who.clone());

			<UserInRoom<T>>::insert(id_room, vec_users.clone());
			<UserDeposit<T>>::insert(who.clone(), deposit);

			T::Currency::transfer(&who, &Self::account_id(), deposit.saturated_into(), KeepAlive)?;

			// If the number of users is 2, room will start
			if (vec_users.len() as i32) == 2 {
				room.is_started = true;
				let start = frame_system::Pallet::<T>::block_number();
				room.start = start;
			}
			<Rooms<T>>::insert(id_room, room);

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// The account ID of the room pot.
	///
	/// This actually does computation. If you need to keep using it, then make sure you cache the
	/// value and only call this once.
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}
}
