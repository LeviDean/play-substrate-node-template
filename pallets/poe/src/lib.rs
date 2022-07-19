#![cfg_attr(not(feature = "std"), no_std)]

/// A module for proof of existence
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    #[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

    #[pallet::pallet]
    #[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn proofs)]
	pub type Proofs<T: Config> = StorageMap<
        _, 
        Blake2_128Concat,
        Vec<u8>,
        (T::AccountId, T::BlockNumber),
    >;

    #[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClaimCreated(T::AccountId, Vec<u8>),
        ClaimRevoked(T::AccountId, Vec<u8>),
        ClaimTransfered(T::AccountId, T::AccountId, Vec<u8>)
	}

    #[pallet::error]
	pub enum Error<T> {
        ProofAlreadyClaimed,
        NoSuchProof,
        NotProofOwner,
	}

    #[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        pub fn creat_claim(origin: OriginFor<T>, claim: Vec<u8>) ->DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            
            ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ProofAlreadyClaimed);
            Proofs::<T>::insert(
                &claim,
                (sender.clone(), frame_system::Pallet::<T>::block_number())
            );

            Self::deposit_event(Event::ClaimCreated(sender, claim));

            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn revoke_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let (owner, _) = Proofs::<T>::get(&claim).ok_or(Error::<T>::NoSuchProof)?;
            ensure!(owner == sender, Error::<T>::NotProofOwner);

            Proofs::<T>::remove(&claim);

            Self::deposit_event(Event::ClaimRevoked(sender, claim));
            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn transfer_claim(origin: OriginFor<T>, receiver: T::AccountId, claim: Vec<u8>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let (owner, _) = Proofs::<T>::get(&claim).ok_or(Error::<T>::NoSuchProof)?;
            ensure!(owner == sender, Error::<T>::NotProofOwner);

            Proofs::<T>::insert(
                &claim,
                (receiver.clone(), frame_system::Pallet::<T>::block_number())
            );

            Self::deposit_event(Event::ClaimTransfered(sender, receiver, claim));
            Ok(().into())
        }
    }
}