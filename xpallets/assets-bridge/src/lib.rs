// Copyright 2023 BEVM Project Authors. Licensed under GPL-3.0.

//! # Assets Bridge
//!
//! ## Overview
//!
//! Bridge between pallet-assets and Erc20 tokens

#![cfg_attr(not(feature = "std"), no_std)]

pub mod abi;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
pub use abi::*;
pub mod recover;
pub use recover::*;

use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement, IsType, ReservableCurrency, WithdrawReasons},
	transactional,
};
use parity_scale_codec::Encode;
use sp_core::{ecdsa, H160, U256};
use sp_io::{crypto::secp256k1_ecdsa_recover, hashing::keccak_256};
use sp_runtime::traits::{StaticLookup, UniqueSaturatedInto, Zero};
use sp_std::vec::Vec;

use pallet_evm::{AddressMapping, CallInfo, ExitReason, Runner};

pub type EcdsaSignature = ecdsa::Signature;
pub type AddressMappingOf<T> = <T as pallet_evm::Config>::AddressMapping;

#[derive(Copy, Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, scale_info::TypeInfo)]
pub enum MappingAssetId {
	NativeCurrency,
	Bitcoin,
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The assets-bridge's inner evm caller.
		#[pallet::constant]
		type EvmCaller: Get<H160>;

		/// The bevm native currency.
		type NativeCurrency: ReservableCurrency<Self::AccountId>;
	}

	/// The Erc20 Contract Addresses for NativeCurrency
	///
	/// Erc20: Option<H160>
	#[pallet::storage]
	#[pallet::getter(fn erc20)]
	pub type Erc20<T: Config> = StorageValue<_, H160>;

	/// The pallet admin key.
	#[pallet::storage]
	#[pallet::getter(fn admin_key)]
	pub(super) type Admin<T: Config> = StorageValue<_, T::AccountId>;

	/// The pause flag
	#[pallet::storage]
	#[pallet::getter(fn is_paused)]
	pub(super) type Pause<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// The `AccountId` of the admin key.
		pub admin_key: Option<T::AccountId>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { admin_key: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			if let Some(key) = &self.admin_key {
				<Admin<T>>::put(key.clone());
			}
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub fn deposit_event)]
	pub enum Event<T: Config> {
		/// (asset_id, account_id, evm_address, amount, erc20_contract)
		DepositExecuted(MappingAssetId, T::AccountId, H160, u128, Option<H160>),
		/// (asset_id, account_id, evm_address, amount, erc20_contract)
		WithdrawExecuted(MappingAssetId, T::AccountId, H160, u128, Option<H160>),
		/// (account_id)
		SetAdmin(T::AccountId),
		/// (erc20_contract)
		Register(H160),
		/// the status of assets bridge
		IsPaused(bool),
	}

	/// Error for evm accounts module.
	#[pallet::error]
	pub enum Error<T> {
		/// Erc20 contract address has been registered
		Registered,
		/// Erc20 contract address has not been registered
		Unregistered,
		/// Failed Erc20 contract call
		ExecutedFailed,
		/// Require admin authority
		RequireAdmin,
		/// Ban deposit and withdraw when in emergency
		InEmergency,
		/// Zero balance
		ZeroBalance,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		DispatchError: From<<<T as pallet_evm::Config>::Runner as pallet_evm::Runner<T>>::Error>,
	{
		/// Register the erc20 contract of native currency.
		/// Note: for admin
		///
		/// - `erc20`: The erc20 contract address
		/// - `overwrite`: Whether to overwrite the existing contract address (use with caution)
		#[pallet::weight({0})]
		#[pallet::call_index(0)]
		pub fn register(
			origin: OriginFor<T>,
			erc20_address: H160,
			overwrite: bool,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(Some(who) == Self::admin_key(), Error::<T>::RequireAdmin);

			ensure!(overwrite || Self::erc20().is_some(), Error::<T>::Registered);

			Erc20::<T>::mutate(|erc20| *erc20 = Some(erc20_address));

			Self::deposit_event(Event::Register(erc20_address));

			Ok(Pays::No.into())
		}

		/// Pause/Unpause assets bridge deposit and withdraw
		/// Note: for admin
		#[pallet::weight({0})]
		#[pallet::call_index(1)]
		pub fn set_pause(origin: OriginFor<T>, pause_value: bool) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(Some(who) == Self::admin_key(), Error::<T>::RequireAdmin);

			Pause::<T>::mutate(|pause| *pause = pause_value);

			Self::deposit_event(Event::IsPaused(pause_value));

			Ok(Pays::No.into())
		}

		/// Set this pallet admin key
		/// Note: for super admin
		#[pallet::weight({0})]
		#[pallet::call_index(2)]
		pub fn set_admin(
			origin: OriginFor<T>,
			new_admin: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			let new_admin = T::Lookup::lookup(new_admin)?;

			let _ = Self::set_admin_inner(new_admin.clone());

			Self::deposit_event(Event::SetAdmin(new_admin));

			Ok(Pays::No.into())
		}

		/// Deposit native currency from wasm to evm
		/// Note: for user who hold native currency
		#[pallet::weight({0})]
		#[pallet::call_index(3)]
		#[transactional]
		pub fn deposit_native_to_evm(
			origin: OriginFor<T>,
			amount: u128,
			eth_address: H160,
		) -> DispatchResultWithPostInfo {
			let asset_id = MappingAssetId::NativeCurrency;

			let who = ensure_signed(origin)?;
			ensure!(!Self::is_in_emergency(), Error::<T>::InEmergency);
			ensure!(!amount.is_zero(), Error::<T>::ZeroBalance);

			let evm_caller = T::EvmCaller::get();
			let proxy = T::AddressMapping::into_account_id(evm_caller);

			// 1. transfer native currency to proxy account
			T::NativeCurrency::transfer(
				&who,
				&proxy,
				amount.unique_saturated_into(),
				ExistenceRequirement::AllowDeath,
			)?;

			// 2. mint native currency to eth_address in evm
			let native_contract = Self::erc20().ok_or(Error::<T>::Unregistered)?;

			let inputs = mint_into_encode(eth_address, amount.unique_saturated_into());

			Self::call_evm(native_contract, inputs)?;

			Self::deposit_event(Event::DepositExecuted(
				asset_id,
				who,
				eth_address,
				amount,
				Some(native_contract),
			));

			Ok(Pays::No.into())
		}

		/// Deposit BTC(btc ledger module) from substrate account to evm address
		/// Note: for user who hold BTC
		#[pallet::weight({0})]
		#[pallet::call_index(4)]
		#[transactional]
		pub fn deposit_btc_to_evm(
			origin: OriginFor<T>,
			amount: u128,
			eth_address: H160,
		) -> DispatchResultWithPostInfo {
			let asset_id = MappingAssetId::Bitcoin;

			let who = ensure_signed(origin)?;
			ensure!(!Self::is_in_emergency(), Error::<T>::InEmergency);
			ensure!(amount > 0, Error::<T>::ZeroBalance);

			let mapping_account = AddressMappingOf::<T>::into_account_id(eth_address);

			<T as pallet_evm::Config>::Currency::transfer(
				&who,
				&mapping_account,
				amount.unique_saturated_into(),
				ExistenceRequirement::AllowDeath,
			)?;

			Self::deposit_event(Event::DepositExecuted(asset_id, who, eth_address, amount, None));

			Ok(Pays::No.into())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn set_admin_inner(new_admin: T::AccountId) -> Weight {
		Admin::<T>::mutate(|admin| *admin = Some(new_admin));
		T::DbWeight::get().reads_writes(1,1)
	}

	pub fn apply_direct_deposit_btc(evm_account: H160, amount: u128) -> DispatchResult {
		ensure!(!Self::is_in_emergency(), Error::<T>::InEmergency);
		ensure!(amount > 0, Error::<T>::ZeroBalance);

		let mapping_account = AddressMappingOf::<T>::into_account_id(evm_account);
		<T as pallet_evm::Config>::Currency::deposit_creating(
			&mapping_account,
			amount.unique_saturated_into(),
		);

		Ok(())
	}

	pub fn withdraw_native_from_evm(
		from: H160,
		dest: T::AccountId,
		amount: u128,
	) -> DispatchResult {
		let asset_id = MappingAssetId::NativeCurrency;

		if Self::is_in_emergency() {
			return Err(DispatchError::Other("in emergency"))
		};

		let evm_caller = T::EvmCaller::get();
		let proxy = T::AddressMapping::into_account_id(evm_caller);

		// 1. transfer native currency from proxy to dest account
		T::NativeCurrency::transfer(
			&proxy,
			&dest,
			amount.unique_saturated_into(),
			ExistenceRequirement::AllowDeath,
		)?;

		// 2. burn native currency(erc20) in evm
		let native_contract = Self::erc20().ok_or(Error::<T>::Unregistered)?;
		let inputs = burn_from_encode(from, amount);
		Self::call_evm(native_contract, inputs)?;

		Self::deposit_event(Event::WithdrawExecuted(
			asset_id,
			dest,
			from,
			amount.unique_saturated_into(),
			Some(native_contract),
		));

		Ok(())
	}

	pub fn withdraw_btc_from_evm(from: H160, amount: u128) -> DispatchResult {
		let asset_id = MappingAssetId::Bitcoin;

		if Self::is_in_emergency() {
			return Err(DispatchError::Other("in emergency"))
		};

		let mapping_account = AddressMappingOf::<T>::into_account_id(from);

		// burn btc in wasm
		<T as pallet_evm::Config>::Currency::withdraw(
			&mapping_account,
			amount.unique_saturated_into(),
			WithdrawReasons::all(),
			ExistenceRequirement::AllowDeath,
		)?;

		Self::deposit_event(Event::WithdrawExecuted(
			asset_id,
			mapping_account,
			from,
			amount.unique_saturated_into(),
			None,
		));

		Ok(())
	}

	fn call_evm(erc20: H160, inputs: Vec<u8>) -> DispatchResult {
		match T::Runner::call(
			T::EvmCaller::get(),
			erc20,
			inputs,
			U256::default(),
			3_000_000,
			None,
			None,
			None,
			Vec::new(),
			false,
			false,
			None,
			None,
			T::config(),
		) {
			Ok(CallInfo { exit_reason: ExitReason::Succeed(_), .. }) => Ok(()),
			_ => Err(Error::<T>::ExecutedFailed.into()),
		}
	}

	fn is_in_emergency() -> bool {
		Self::is_paused()
	}
}
