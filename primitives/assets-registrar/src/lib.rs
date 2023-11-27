// Copyright 2023 BEVM Project Authors. Licensed under GPL-3.0.

//! The asset registrar primitives.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use sp_runtime::{DispatchError, DispatchResult, RuntimeDebug};
use sp_std::slice::Iter;

use bevm_primitives::AssetId;

const CHAINS: [Chain; 1] = [Chain::Bitcoin];

/// The blockchain types.
#[derive(
	PartialEq,
	Eq,
	Ord,
	PartialOrd,
	Clone,
	Copy,
	Default,
	Encode,
	Decode,
	RuntimeDebug,
	MaxEncodedLen,
	TypeInfo,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Chain {
	/// Bitcoin
	#[default]
	Bitcoin,

	/// Placeholder
	NotSupportedChain,
}

impl Chain {
	/// Returns an iterator of all `Chain`.
	pub fn iter() -> Iter<'static, Chain> {
		CHAINS.iter()
	}
}

/// Trait for doing some stuff on the registration/deregistration of a foreign asset.
pub trait RegistrarHandler {
	/// Called when a new asset is added or a deregistered asset is recovered.
	fn on_register(_asset_id: &AssetId, _has_mining_rights: bool) -> DispatchResult {
		Ok(())
	}

	/// Called when an asset is deregistered.
	fn on_deregister(_asset_id: &AssetId) -> DispatchResult {
		Ok(())
	}
}

#[impl_trait_for_tuples::impl_for_tuples(30)]
impl RegistrarHandler for Tuple {
	fn on_register(asset_id: &AssetId, has_mining_rights: bool) -> DispatchResult {
		for_tuples!( #( Tuple::on_register(asset_id, has_mining_rights)?; )* );
		Ok(())
	}

	fn on_deregister(asset_id: &AssetId) -> DispatchResult {
		for_tuples!( #( Tuple::on_deregister(asset_id)?; )* );
		Ok(())
	}
}

#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
/// BTC withdraw limit parameters
pub struct WithdrawalLimit<Balance> {
	/// Minimal balance
	pub minimal_withdrawal: Balance,
	/// Fee balance
	pub fee: Balance,
}

/// Bitcoin config
pub trait ChainT<Balance: Default> {
	/// ASSET should be the native Asset for this chain.
	/// BTC is Default 0
	fn asset_id() -> AssetId;
	/// Chain type
	fn chain() -> Chain;
	/// Check address
	fn check_addr(_addr: &[u8], _ext: &[u8]) -> DispatchResult {
		Ok(())
	}
	/// Withdraw limit
	fn withdrawal_limit(_asset_id: &AssetId) -> Result<WithdrawalLimit<Balance>, DispatchError> {
		Ok(WithdrawalLimit::default())
	}
}
