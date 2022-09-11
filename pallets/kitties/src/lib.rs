#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::dispatch::{fmt::Debug, Codec};
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{Currency, Randomness, ReservableCurrency};
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;
	use sp_runtime::traits::{AtLeast32BitUnsigned, Bounded, One};

	// type KittyIndex = u32;

	#[pallet::type_value]
	pub fn GetDefaultValue<T: Config>() -> T::KittyIndex {
		0_u8.into()
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
	pub struct Kitty(pub [u8; 16]);

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		// ! Inject KittyIndex from the Runtime
		type KittyIndex: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Codec
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ Debug
			+ MaxEncodedLen
			+ TypeInfo
			+ Bounded;
		
		#[pallet::constant]	
		type MaxKittiesOwned: Get<u32>;

		// ! Need reserve tokens
		#[pallet::constant]
		type KittyReserve: Get<BalanceOf<Self>>;
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKitty<T: Config> = StorageValue<_, T::KittyIndex, ValueQuery, GetDefaultValue<T>>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Kitty>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn owner_kitties)]
	// ! key:owner value:[KittyIndex]
	pub type OwnerKitties<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<T::KittyIndex, T::MaxKittiesOwned>,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated(T::AccountId, T::KittyIndex, Kitty),
		KittyBred(T::AccountId, T::KittyIndex, Kitty),
		KittyTransferred(T::AccountId, T::AccountId, T::KittyIndex),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidKittyId,
		SameKittyId,
		NotOwner,
		KittiesIndexError,
		TooManyKitties,
		// ---------
		TokenNotEnough,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let kitty_id = Self::get_next_id().map_err(|_| Error::<T>::InvalidKittyId)?;

			// ! reserve tokens
			T::Currency::reserve(&who, T::KittyReserve::get())
				.map_err(|_| Error::<T>::TokenNotEnough)?;

			let dna = Self::random_value(&who);
			let kitty = Kitty(dna);

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &who);
			NextKitty::<T>::set(kitty_id + One::one());

			OwnerKitties::<T>::try_mutate(&who, |kitties| kitties.try_push(kitty_id.clone()))
				.map_err(|_| Error::<T>::TooManyKitties)?;

			Self::deposit_event(Event::KittyCreated(who, kitty_id, kitty));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn breed(
			origin: OriginFor<T>,
			kitty_id_1: T::KittyIndex,
			kitty_id_2: T::KittyIndex,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			T::Currency::reserve(&who, T::KittyReserve::get())
				.map_err(|_| Error::<T>::TokenNotEnough)?;

			// check kitty id
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameKittyId);
			let kitty_1 = Self::get_kitty(kitty_id_1).map_err(|_| Error::<T>::InvalidKittyId)?;
			let kitty_2 = Self::get_kitty(kitty_id_2).map_err(|_| Error::<T>::InvalidKittyId)?;

			// get next id
			let kitty_id = Self::get_next_id().map_err(|_| Error::<T>::InvalidKittyId)?;

			// selector for breeding
			let select = Self::random_value(&who);

			let mut data = [0u8; 16];
			for i in 0..kitty_1.0.len() {
				// 0 choose kitty_1, and 1 choose kitty_2
				data[i] = (kitty_1.0[i] & select[i]) | (kitty_2.0[i] & !select[i]);
			}

			let new_kitty = Kitty(data);

			Kitties::<T>::insert(kitty_id, &new_kitty);
			KittyOwner::<T>::insert(kitty_id, &who);
			NextKitty::<T>::set(kitty_id + One::one());

			OwnerKitties::<T>::try_mutate(&who, |kitties| kitties.try_push(kitty_id.clone()))
				.map_err(|_| Error::<T>::TooManyKitties)?;

			Self::deposit_event(Event::KittyBred(who, kitty_id, new_kitty));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn transfer(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
			new_owner: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let _target_kitty =
				Self::get_kitty(kitty_id).map_err(|_| Error::<T>::InvalidKittyId)?;

			ensure!(Self::kitty_owner(kitty_id) == Some(who.clone()), Error::<T>::NotOwner);

			OwnerKitties::<T>::try_mutate(&who, |owned| {
				if let Some(index) = owned.iter().position(|kitty| kitty == &kitty_id) {
					owned.swap_remove(index);
					return Ok(());
				}
				Err(())
			})
			.map_err(|_| <Error<T>>::NotOwner)?;

			// unreserve old
			T::Currency::unreserve(&who, T::KittyReserve::get());

			// reserve new
			T::Currency::reserve(&new_owner, T::KittyReserve::get())
				.map_err(|_| Error::<T>::TokenNotEnough)?;

			KittyOwner::<T>::insert(kitty_id, new_owner.clone());

			OwnerKitties::<T>::try_mutate(&new_owner, |kitties| kitties.try_push(kitty_id.clone()))
				.map_err(|_| Error::<T>::TooManyKitties)?;

			Self::deposit_event(Event::KittyTransferred(who, new_owner, kitty_id));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		// get a random 256
		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);

			payload.using_encoded(blake2_128) // ! [u8; 16] ?
		}

		fn get_next_id() -> Result<T::KittyIndex, DispatchError> {
			let kitty_id = Self::next_kitty_id();
			if kitty_id == T::KittyIndex::max_value() {
				return Err(Error::<T>::KittiesIndexError.into());
			}
			Ok(kitty_id)
		}

		// get kitty via id
		fn get_kitty(kitty_id: T::KittyIndex) -> Result<Kitty, ()> {
			match Self::kitties(kitty_id) {
				Some(kitty) => Ok(kitty),
				None => Err(()),
			}
		}
	}
}
