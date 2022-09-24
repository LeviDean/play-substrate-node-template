#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use sp_runtime::{
	offchain::storage::{MutateStorageError, StorageRetrievalError, StorageValueRef},
	offchain::{http, Duration},
	traits::Zero,
};

use serde::{Deserialize, Deserializer};

use frame_system::{
    offchain::{
        AppCrypto, CreateSignedTransaction, SendSignedTransaction, SubmitTransaction,
        Signer,
    },
};
use sp_core::crypto::KeyTypeId;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"ocwd");
pub mod crypto {
    use super::KEY_TYPE;
    use sp_core::sr25519::Signature as Sr25519Signature;
    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        traits::Verify,
        MultiSignature, MultiSigner,
    };
    app_crypto!(sr25519, KEY_TYPE);

    pub struct OcwAuthId;

    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for OcwAuthId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }

    impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
        for OcwAuthId
        {
            type RuntimeAppPublic = Public;
            type GenericSignature = sp_core::sr25519::Signature;
            type GenericPublic = sp_core::sr25519::Public;
        }
}


const ONCHAIN_TX_KEY: &[u8] = b"ocw::storage::tx";

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	// use frame_support::inherent::Vec;
	use sp_std::vec::Vec;
    use sp_std::vec;
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::offchain::storage::{StorageRetrievalError, StorageValueRef},
	};
	use frame_system::{pallet_prelude::*, offchain::SendTransactionTypes};
	use sp_io::offchain_index;

	#[derive(Deserialize, Encode, Decode)]
	struct GithubInfo {
		#[serde(deserialize_with = "de_string_to_bytes")]
		login: Vec<u8>,
		#[serde(deserialize_with = "de_string_to_bytes")]
		blog: Vec<u8>,
		public_repos: u32,
	}

	use core::{convert::TryInto, fmt};
    impl fmt::Debug for GithubInfo {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{{ login: {}, blog: {}, public_repos: {} }}",
                sp_std::str::from_utf8(&self.login).map_err(|_| fmt::Error)?,
                sp_std::str::from_utf8(&self.blog).map_err(|_| fmt::Error)?,
                &self.public_repos
                )
        }
    }

	#[derive(Debug, Deserialize, Encode, Decode, Default)]
	struct IndexingData(Vec<u8>, u64);
	

	pub fn de_string_to_bytes<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
	where
	D: Deserializer<'de>,
	{
		let s: &str = Deserialize::deserialize(de)?;
		Ok(s.as_bytes().to_vec())
	}


	#[pallet::config]
	pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> + SendTransactionTypes<Call<Self>>{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);


	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SomethingStored(u32, T::AccountId),
	}


	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
		StorageOverflow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}


		#[pallet::weight(0)]
        pub fn submit_data(origin: OriginFor<T>, payload: Vec<u8>) -> DispatchResultWithPostInfo {

            let _who = ensure_signed(origin)?;

            log::info!("in submit_data call: {:?}", payload);

            Ok(().into())
        }

		#[pallet::weight(0)]
		pub fn submit_data_unsigned(origin: OriginFor<T>, n: u64) -> DispatchResult {
			ensure_none(origin)?;

            
			// let key = Self::derive_key(frame_system::Module::<T>::block_number());
			// let key = b"test_key".to_vec();
			let key = b"testKey".to_vec();
			let data = IndexingData(b"submit_number_unsigned".to_vec(), n);
			offchain_index::set(&key, &data.encode());
			log::info!("in submit_data_unsigned, value: {:?}", n);

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: T::BlockNumber) {
			log::info!("Hello from offchain workers!: {:?}", block_number);

			// unsigned transaction + write to offchain storage
			let value: u64 = 42;

            let call = Call::submit_data_unsigned { n: value };

            _ = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
                .map_err(|_| {
                    log::error!("Failed in offchain_unsigned_tx");
                });

			// transaction
			// let payload: Vec<u8> = vec![1,2,3,4,5,6,7,8];
            // _ = Self::send_signed_tx(payload);
			

			// offchain http
			// if let Ok(info) = Self::fetch_github_info() {
            //     log::info!("Github Info: {:?}", info);
            // } else {
            //     log::info!("Error while fetch github info!");
            // }
			
			// offchain storage
			if block_number % 2u32.into() != Zero::zero() {
				// odd
				let key = Self::derive_key(block_number);
				let val_ref = StorageValueRef::persistent(&key);

				// get a local random value
				let random_slice = sp_io::offchain::random_seed();

				// get a local timestamp
				let timestamp_u64 = sp_io::offchain::timestamp().unix_millis();

				// combine to a tuple and print it
				let value = (random_slice, timestamp_u64);
				log::info!("in odd block, value to write: {:?}", value);

				struct StateError;

				// write or mutate tuple content to key
				// val_ref.set(&value);

				let res = val_ref.mutate(|val: Result<Option<([u8; 32], u64)>, StorageRetrievalError>| -> Result<_, StateError> {
					match val {
						Ok(Some(_)) => Ok(value),
						_ => Ok(value),
					}
				});

				match res {
					Ok(value) => {
						log::info!("in odd block, mutate successfully: {:?}", value);
					},
					Err(MutateStorageError::ValueFunctionFailed(_)) => (),
					Err(MutateStorageError::ConcurrentModification(_)) => (),
				};

			} else {
				// even
				let key = Self::derive_key(block_number - 1u32.into());
				// ? Why need mut
				let mut val_ref = StorageValueRef::persistent(&key);

				// get from db by key
				if let Ok(Some(value)) = val_ref.get::<([u8; 32], u64)>() {
					// print values
					log::info!("in even block, value read: {:?}", value);
					// delete that key
					val_ref.clear();
				}
			}

			log::info!("Bye-bye from offchain workers!: {:?}", block_number);
		}
	}

	impl<T: Config> Pallet<T> {

		// #[deny(clippy::clone_double_ref)]
		fn derive_key(block_number: T::BlockNumber) -> Vec<u8> {
			block_number.using_encoded(|encoded_bn| {
			  ONCHAIN_TX_KEY.clone().into_iter()
				.chain(b"/".into_iter())
				.chain(encoded_bn)
				.copied()
				.collect::<Vec<u8>>()
			})
		}

		fn fetch_github_info() -> Result<GithubInfo, http::Error> {
            // prepare for send request
            let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(8_000));
            let request =
                http::Request::get("https://api.github.com/orgs/substrate-developer-hub");
            let pending = request
                .add_header("User-Agent", "Substrate-Offchain-Worker")
                .deadline(deadline).send().map_err(|_| http::Error::IoError)?;
            let response = pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;
            if response.code != 200 {
                log::warn!("Unexpected status code: {}", response.code);
                return Err(http::Error::Unknown)
            }
            let body = response.body().collect::<Vec<u8>>();
            let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
                log::warn!("No UTF8 body");
                http::Error::Unknown
            })?;

            // parse the response str
            let gh_info: GithubInfo =
                serde_json::from_str(body_str).map_err(|_| http::Error::Unknown)?;

            Ok(gh_info)
        }

		fn send_signed_tx(payload: Vec<u8>) -> Result<(), &'static str> {
            let signer = Signer::<T, T::AuthorityId>::all_accounts();
            if !signer.can_sign() {
                return Err(
                    "No local accounts available. Consider adding one via `author_insertKey` RPC.",
                    )
            }

            let results = signer.send_signed_transaction(|_account| {

                Call::submit_data { payload: payload.clone() }
            });

            for (acc, res) in &results {
                match res {
                    Ok(()) => log::info!("[{:?}] Submitted data:{:?}", acc.id, payload),
                    Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
                }
            }

            Ok(())
        }


	}

	#[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            if let Call::submit_data_unsigned { n: _ } = call {
                //let provide = b"submit_xxx_unsigned".to_vec();
                ValidTransaction::with_tag_prefix("ExampleOffchainWorker")
                    .priority(10000)
                    .and_provides(1)
                    .longevity(3)
                    .propagate(true)
                    .build()
            } else {
                InvalidTransaction::Call.into()
            }
        }
    }
}
