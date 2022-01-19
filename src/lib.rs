#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, traits::Get};
use frame_support::inherent::Vec;
use frame_system::ensure_signed;
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Config: frame_system::Config {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	trait Store for Module<T: Config> as IdentityModule {
		pub Identity get(fn get_identity): map hasher(blake2_128_concat) Vec<u8> => Option<T::AccountId>;

		// ( identity, attribute_key ) => attribute_value
		pub Attribute get(fn get_attribute): map hasher(blake2_128_concat) (Vec<u8>, Vec<u8>) => Vec<u8>;
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Config>::AccountId {
		IdentityCreated(Vec<u8>, AccountId),

		// Identity, Attribute Key, Attribute Value
		AttributeAdded(Vec<u8>, Vec<u8>, Vec<u8>),

		// Identity, Attribute Key
		AttributeRemoved(Vec<u8>, Vec<u8>),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Config> {
		IdentityAlreadyClaimed,
		IdentityNotFound,
		NotAuthorized,
		AttributeNotFound,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		#[weight = 10_000 + T::DbWeight::get().reads_writes(1, 1)]
		pub fn create_identity(
			origin, 
			identity: Vec<u8>
		) -> dispatch::DispatchResult {

			let who = ensure_signed(origin)?;

			match <Identity<T>>::get(&identity) {
				// Return an error if signer is not identity owner
				None => {
					// Update storage.
					<Identity<T>>::insert(&identity, &who);
					// Emit an event.
					Self::deposit_event(RawEvent::IdentityCreated(identity, who));
					// Return a successful DispatchResult
					Ok(())
				},
				Some(_) => Err(Error::<T>::IdentityAlreadyClaimed)?
			}
			
		}

		// Allows identity owners to add attribute to their identity (key, value)
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn add_attribute(
			origin,
			identity: Vec<u8>,
			attribute_key: Vec<u8>,
			attribute_value: Vec<u8>
		) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Identity<T>>::get(&identity) {
				// Return an error if signer is not identity owner
				None => Err(Error::<T>::IdentityNotFound)?,
				Some(address) => {
					if address != who {
						return Err(Error::<T>::NotAuthorized)?
					} else{
						Attribute::insert((&identity, &attribute_key), &attribute_value);
						Self::deposit_event(RawEvent::AttributeAdded(identity, attribute_key, attribute_value));
						Ok(())
					}
				},
			}
		}

		// Allows identity owners to remove identity
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn remove_attribute(
			origin,
			identity: Vec<u8>,
			attribute_key: Vec<u8>,
		) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Identity<T>>::get(&identity) {
				// Return an error if signer is not identity owner
				None => Err(Error::<T>::IdentityNotFound)?,
				Some(address) => {
					if address != who {
						return Err(Error::<T>::NotAuthorized)?
					} else{
						Attribute::remove((&identity, &attribute_key));
						Self::deposit_event(RawEvent::AttributeRemoved(identity, attribute_key));
						Ok(())
					}
				},
			}
		}
	}
}