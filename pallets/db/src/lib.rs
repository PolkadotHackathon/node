#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	trait BoundedVecContains {
		type Item;

		fn contains(&self, item: &Self::Item) -> bool;
	}

	impl<T: PartialEq, S> BoundedVecContains for BoundedVec<T, S> {
		type Item = T;

		fn contains(&self, item: &Self::Item) -> bool {
			self.iter().any(|i| i == item)
		}
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	// TODO: Make contain real good stuff
	#[derive(Clone, Encode, Decode, PartialEq, Copy, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub struct UserClick {
		pub dom_id: u64,
		pub timestamp: u64,
	}

	#[allow(type_alias_bounds)]
	pub type UserClicks = [UserClick; 1000];

	#[allow(type_alias_bounds)]
	pub type WebsiteUsers<T: Config> = [T::AccountId; 5];
	
	#[pallet::storage]
	pub(super) type UserMap<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, UserClicks>;

	#[pallet::storage]
	pub(super) type WebsiteMap<T: Config> = StorageMap<_, Twox64Concat, u64, WebsiteUsers<T>>;

	#[pallet::error]
	pub enum Error<T> {
		WebsiteAlreadyRegistered,
		WebsiteNotRegistered,
		WebsiteIncorrectlyRegistered,
		UserAlreadyRegistered,
		UserNotRegistered,
		UserDataOverflow,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		WebsiteRegistered(u64),
		UserAdded(T::AccountId),
		UserRemoved(T::AccountId),
		UserUpdated(T::AccountId),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(0)]
		pub fn register_website(origin: OriginFor<T>, website_id: u64) -> DispatchResult {
			let _sender = ensure_signed(origin)?;

			ensure!(!WebsiteMap::<T>::contains_key(&website_id), Error::<T>::WebsiteAlreadyRegistered);

			WebsiteMap::<T>::insert(&website_id, WebsiteUsers::<T>::new());

			Self::deposit_event(Event::WebsiteRegistered(website_id));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn update_click(
			origin: OriginFor<T>,
			website_id: u64,
			user_id: T::AccountId,
			dom_id: u64,
			timestamp: u64,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// UserMap::<T>::insert(&sender, data);

			// TODO:
			// 0. Check if website is registered
			// 1. Register user if not already registered
			// 2. Add click to user data
			// 3. Shift clicks left if overflow

			// INFO: 0.
			ensure!(WebsiteMap::<T>::contains_key(&dom_id), Error::<T>::WebsiteNotRegistered);

			// INFO: 1.
			let website_users = WebsiteMap::<T>::get(&website_id);
			ensure!(website_users.is_some(), Error::<T>::WebsiteIncorrectlyRegistered);

			if let Some(mut website_users) = website_users {
				if !website_users.contains(&user_id) {
					let result = website_users.try_push(user_id.clone());
					// Register users on website
					ensure!(result.is_ok(), Error::<T>::UserAlreadyRegistered);

					// Register user in user map
					UserMap::<T>::insert(&user_id, UserClicks::<T>::new());

					Self::deposit_event(Event::UserAdded(user_id.clone()));
				}

				// Add click to user data
				let user_data = UserMap::<T>::get(user_id.clone());
				ensure!(user_data.is_some(), Error::<T>::UserNotRegistered);

				if let Some(mut user_data) = user_data {
					let result = user_data.try_push(UserClick { dom_id, timestamp });
					ensure!(result.is_ok(), Error::<T>::UserDataOverflow);

					// TODO: Shift clicks left if overflow

					UserMap::<T>::insert(&user_id.clone(), user_data);
				}
			} else {
				unreachable!();
			}

			Self::deposit_event(Event::UserUpdated(sender));
			Ok(())
		}
	}
}
