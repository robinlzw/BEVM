// Copyright 2023 BEVM Project Authors. Licensed under GPL-3.0.

//! Minimal Pallet that stores the numeric Ethereum-style chain id in the runtime.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet;

pub use pallet::*;

#[pallet]
pub mod pallet {
	use core::marker::PhantomData;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// The Ethereum Chain Id Pallet
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {}

	impl<T: Config> Get<u64> for Pallet<T> {
		fn get() -> u64 {
			Self::chain_id()
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn chain_id)]
	pub type ChainId<T> = StorageValue<_, u64, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub chain_id: u64,
		#[serde(skip)]
		pub _marker: PhantomData<T>,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { chain_id: 1501u64, _marker: PhantomData }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			ChainId::<T>::put(self.chain_id);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight({0})]
		#[pallet::call_index(0)]
		pub fn set_chain_id(
			origin: OriginFor<T>,
			#[pallet::compact] new_chain_id: u64,
		) -> DispatchResult {
			ensure_root(origin)?;

			ChainId::<T>::mutate(|chain_id| *chain_id = new_chain_id);

			Ok(())
		}
	}
}
