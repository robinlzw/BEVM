// Copyright 2023 BEVM Project Authors. Licensed under GPL-3.0.

#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::Codec;

sp_api::decl_runtime_apis! {
	pub trait BtcLedgerApi<AccountId, Balance>
	where
		AccountId: Codec,
		Balance: Codec,
	{
		fn get_balance(who: AccountId) -> Balance;
		fn get_total() -> Balance;
	}
}
