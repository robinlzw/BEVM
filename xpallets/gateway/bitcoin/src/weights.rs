// Copyright 2023 BEVM Project Authors. Licensed under GPL-3.0.

//! Weights for xpallet_gateway_bitcoin
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-05-13, STEPS: 50, REPEAT: 20, LOW RANGE: [], HIGH RANGE: []
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("benchmarks"), DB CACHE: 1024

// Executed Command:
// ./target/release/chainx
// benchmark
// --chain=benchmarks
// --steps=50
// --repeat=20
// --pallet=xpallet_gateway_bitcoin
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./xpallets/gateway/bitcoin/src/weights.rs
// --template=./scripts/xpallet-weight-template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for xpallet_gateway_bitcoin.
pub trait WeightInfo {
	fn push_header() -> Weight;
	fn push_transaction() -> Weight;
	fn create_taproot_withdraw_tx() -> Weight;
	fn set_best_index() -> Weight;
	fn set_confirmed_index() -> Weight;
	fn remove_pending() -> Weight;
	fn remove_proposal() -> Weight;
	fn set_btc_withdrawal_fee() -> Weight;
	fn set_btc_deposit_limit() -> Weight;
	fn set_coming_bot() -> Weight;
}

/// Weights for xpallet_gateway_bitcoin using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn push_header() -> Weight {
		todo!()
	}
	fn push_transaction() -> Weight {
		todo!()
	}
	fn create_taproot_withdraw_tx() -> Weight {
		todo!()
	}
	fn set_best_index() -> Weight {
		todo!()
	}
	fn set_confirmed_index() -> Weight {
		todo!()
	}
	fn remove_pending() -> Weight {
		todo!()
	}
	fn remove_proposal() -> Weight {
		todo!()
	}
	fn set_btc_withdrawal_fee() -> Weight {
		todo!()
	}
	fn set_btc_deposit_limit() -> Weight {
		todo!()
	}
	fn set_coming_bot() -> Weight {
		todo!()
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn push_header() -> Weight {
		todo!()
	}
	fn push_transaction() -> Weight {
		todo!()
	}
	fn create_taproot_withdraw_tx() -> Weight {
		todo!()
	}
	fn set_best_index() -> Weight {
		todo!()
	}
	fn set_confirmed_index() -> Weight {
		todo!()
	}
	fn remove_pending() -> Weight {
		todo!()
	}
	fn remove_proposal() -> Weight {
		todo!()
	}
	fn set_btc_withdrawal_fee() -> Weight {
		todo!()
	}
	fn set_btc_deposit_limit() -> Weight {
		todo!()
	}
	fn set_coming_bot() -> Weight {
		todo!()
	}
}
