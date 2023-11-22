// Copyright 2023 BEVM Project Authors. Licensed under GPL-3.0.

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::RuntimeDebug;

/// The network type of BEVM.
#[derive(PartialEq, Eq, Clone, Copy, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum NetworkType {
	/// Main network type
	Mainnet,
	/// Test network type
	#[default]
	Testnet,
}

impl NetworkType {
	/// Return the ss58 address format identifier of the network type.
	pub fn ss58_addr_format_id(&self) -> Ss58AddressFormatId {
		match self {
			NetworkType::Mainnet => MAINNET_ADDRESS_FORMAT_ID,
			NetworkType::Testnet => TESTNET_ADDRESS_FORMAT_ID,
		}
	}
}

/// Ss58AddressFormat identifier
pub type Ss58AddressFormatId = u8;
/// BEVM main network ss58 address format identifier
pub const MAINNET_ADDRESS_FORMAT_ID: Ss58AddressFormatId = 44; // 44 is Ss58AddressFormat::BEVMAccount
/// BEVM test network ss58 address format identifier
pub const TESTNET_ADDRESS_FORMAT_ID: Ss58AddressFormatId = 42; // 42 is Ss58AddressFormat::SubstrateAccount
