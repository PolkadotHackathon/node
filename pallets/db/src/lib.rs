#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

type HashedID = [u32; 16];

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	// trait BoundedVecContains {
	// 	type Item;
	//
	// 	fn contains(&self, item: &Self::Item) -> bool;
	// }
	//
	// impl<T: PartialEq, S> BoundedVecContains for BoundedVec<T, S> {
	// 	type Item = T;
	//
	// 	fn contains(&self, item: &Self::Item) -> bool {
	// 		self.iter().any(|i| i == item)
	// 	}
	// }

	use frame_support::traits::Currency;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The maximum number of users that can be stored in the pallet.
		#[pallet::constant]
		type MaxUserCount: Get<u32>;

		/// The maximum number of user data items that can be stored in the pallet.
		#[pallet::constant]
		type MaxUserData: Get<u32>;

		#[pallet::constant]
		type GlobalKey: Get<u32>;

		#[pallet::constant]
		type InitialBalance: Get<u32>;

		type Currency: Currency<Self::AccountId>;
	}

	// TODO: Make contain real good stuff
	#[derive(Clone, Encode, Decode, PartialEq, Copy, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub struct UserClick {
		pub dom_id: HashedID,
		pub timestamp: u64,
	}

	/// A list of user clicks and their timestamps.
	// #[allow(type_alias_bounds)]
	// pub type UserClicks<T: Config> = BoundedVec<UserClick, T::MaxUserData>;

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct UserClicks<T: Config> {
		/// Clicks (CYCLCIC BUFFER)
		pub clicks: BoundedVec<UserClick, T::MaxUserData>,
		/// Position of the next click to be added
		pub pos: u16,
	}

	impl<T: Config> UserClicks<T> {
		pub fn new() -> Self {
			Self { clicks: BoundedVec::<UserClick, T::MaxUserData>::new(), pos: 0 }
		}

		pub fn push(&mut self, click: UserClick) {
			// self.clicks[self.pos as usize] = click;
			// self.clicks.force_insert(self.pos as usize, click).unwrap();
			if self.clicks.try_push(click).is_err() {
				*self.clicks.get_mut(self.pos as usize).unwrap() = click;
			}
			self.pos = (self.pos + 1) % T::MaxUserData::get() as u16;
		}
	}

	/// /// A mapping from accounts to user
	/// #[pallet::storage]
	/// pub(super) type UserMap<T: Config> = StorageMap<_, Twox64Concat, T::AccountId,
	/// UserClicks<T>>;

	/// A mapping from user accounts to their user clicks.
	#[allow(type_alias_bounds)]
	pub type WebsiteUsers<T: Config> =
		BoundedBTreeMap<T::AccountId, UserClicks<T>, T::MaxUserCount>;

	/// A mapping from websites to their users.
	#[pallet::storage]
	pub(super) type WebsiteMap<T: Config> = StorageMap<_, Twox64Concat, u128, WebsiteUsers<T>>;

	#[pallet::error]
	pub enum Error<T> {
		WebsiteAlreadyRegistered,
		WebsiteNotRegistered,
		WebsiteIncorrectlyRegistered,
		UserAlreadyRegistered,
		UserNotRegistered,
		UserDataOverflow,
		InsufficientBalance,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		UserAdded(T::AccountId),
		UserRemoved(T::AccountId),
		UserUpdated(T::AccountId),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight((Weight::from_parts(0, 0), Pays::No))]
		pub fn register_website(origin: OriginFor<T>, website_id: u128) -> DispatchResult {
			// let _sender = ensure_signed(origin)?;

			ensure!(
				!WebsiteMap::<T>::contains_key(&website_id),
				Error::<T>::WebsiteAlreadyRegistered
			);

			WebsiteMap::<T>::insert(website_id, WebsiteUsers::<T>::new());

			Ok(())
		}

		#[pallet::weight((Weight::from_parts(0, 0), Pays::No))]
		pub fn update_click(
			origin: OriginFor<T>,
			website_id: u128,
			user_id: T::AccountId,
			dom_id: HashedID,
			timestamp: u64,
		) -> DispatchResult {
			// Money spent on this transaction

			// let sender = ensure_signed(origin.clone())?;

			// TODO:
			// 0. Check if website is registered
			// 1. Register user if not already registered
			// 2. Add click to user data
			// 3. Shift clicks left if overflow

			// INFO: 0.
			ensure!(WebsiteMap::<T>::contains_key(&website_id), Error::<T>::WebsiteNotRegistered);

			// INFO: 1.
			let website_users = WebsiteMap::<T>::get(&website_id);
			ensure!(website_users.is_some(), Error::<T>::WebsiteIncorrectlyRegistered);

			if let Some(mut website_users) = website_users {
				if !website_users.contains_key(&user_id) {
					let result = website_users.try_insert(user_id.clone(), UserClicks::<T>::new());
					// Register users on website
					ensure!(result.is_ok(), Error::<T>::UserAlreadyRegistered);
					Self::deposit_event(Event::UserAdded(user_id.clone()));
				}

				// Sanity check
				ensure!(website_users.contains_key(&user_id), Error::<T>::UserNotRegistered);

				// Add click to user data
				let user_data = website_users.get_mut(&user_id).unwrap();
				user_data.push(UserClick { dom_id, timestamp });
				// ensure!(result.is_ok(), Error::<T>::UserDataOverflow);

				// Insert user in user map
				WebsiteMap::<T>::insert(website_id, website_users);
			}

			Self::deposit_event(Event::UserUpdated(user_id));
			Ok(())
		}
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			// if let Call::register_website { ref website_id } = call {
			// 	if let Ok(hash) = frame_system::Pallet::<T>::r
			// 	{
			// 		return Ok(ValidTransaction {
			// 			priority: 100,
			// 			requires: Vec::new(),
			// 			provides: vec![hash.as_ref().to_vec()],
			// 			longevity: TransactionLongevity::max_value(),
			// 			propagate: true,
			// 		});
			// 	}
			// }
			//
			// Err(InvalidTransaction::Call.into())

			Ok(ValidTransaction {
				priority: 100,
				requires: Vec::new(),
				provides: vec![T::GlobalKey::get().to_be_bytes().to_vec()],
				longevity: TransactionLongevity::max_value(),
				propagate: true,
			})
		}
	}
}
