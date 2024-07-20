#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
	pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }
    
    #[derive(Clone, Encode, Decode, PartialEq, Copy, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct UserData {
        pub data: u32,
    }

    #[pallet::storage]
    pub(super) type UserMap<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, UserData>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        UserAdded(T::AccountId),
        UserRemoved(T::AccountId),
        UserUpdated(T::AccountId),
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {

        #[pallet::weight(0)]
        pub fn update_click(origin: OriginFor<T>, button: Vec<u8>, timestamp: u32) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let data = UserData { data: timestamp };

            UserMap::<T>::insert(&sender, data);
            Self::deposit_event(Event::UserUpdated(sender));
            Ok(())
        }
    }
}