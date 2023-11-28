// Copyright 2023 BEVM Project Authors. Licensed under GPL-3.0.

//! Weights for xpallet_mining_staking
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-05-13, STEPS: 50, REPEAT: 20, LOW RANGE: [], HIGH RANGE: []
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("benchmarks"), DB CACHE: 1024

// Executed Command:
// ./target/release/chainx
// benchmark
// --chain=benchmarks
// --steps=50
// --repeat=20
// --pallet=xpallet_mining_staking
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./xpallets/mining/staking/src/weights.rs
// --template=./scripts/xpallet-weight-template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for xpallet_mining_staking.
pub trait WeightInfo {
	fn register() -> Weight;
	fn bond() -> Weight;
	fn unbond() -> Weight;
	fn unlock_unbonded_withdrawal() -> Weight;
	fn rebond() -> Weight;
	fn claim() -> Weight;
	fn chill() -> Weight;
	fn validate() -> Weight;
	fn set_validator_count() -> Weight;
	fn set_minimum_validator_count() -> Weight;
	fn set_bonding_duration() -> Weight;
	fn set_validator_bonding_duration() -> Weight;
	fn set_minimum_penalty() -> Weight;
	fn set_sessions_per_era() -> Weight;
}

/// Weights for xpallet_mining_staking using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn register() -> Weight {
		todo!()
	}
	fn bond() -> Weight {
		todo!()
	}
	fn unbond() -> Weight {
		todo!()
	}
	fn unlock_unbonded_withdrawal() -> Weight {
		todo!()
	}
	fn rebond() -> Weight {
		todo!()
	}
	fn claim() -> Weight {
		todo!()
	}
	fn chill() -> Weight {
		todo!()
	}
	fn validate() -> Weight {
		todo!()
	}
	fn set_validator_count() -> Weight {
		todo!()
	}
	fn set_minimum_validator_count() -> Weight {
		todo!()
	}
	fn set_bonding_duration() -> Weight {
		todo!()
	}
	fn set_validator_bonding_duration() -> Weight {
		todo!()
	}
	fn set_minimum_penalty() -> Weight {
		todo!()
	}
	fn set_sessions_per_era() -> Weight {
		todo!()
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn register() -> Weight {
		todo!()
	}
	fn bond() -> Weight {
		todo!()
	}
	fn unbond() -> Weight {
		todo!()
	}
	fn unlock_unbonded_withdrawal() -> Weight {
		todo!()
	}
	fn rebond() -> Weight {
		todo!()
	}
	fn claim() -> Weight {
		todo!()
	}
	fn chill() -> Weight {
		todo!()
	}
	fn validate() -> Weight {
		todo!()
	}
	fn set_validator_count() -> Weight {
		todo!()
	}
	fn set_minimum_validator_count() -> Weight {
		todo!()
	}
	fn set_bonding_duration() -> Weight {
		todo!()
	}
	fn set_validator_bonding_duration() -> Weight {
		todo!()
	}
	fn set_minimum_penalty() -> Weight {
		todo!()
	}
	fn set_sessions_per_era() -> Weight {
		todo!()
	}
}
