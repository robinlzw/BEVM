// Copyright 2023 BEVM Project Authors. Licensed under GPL-3.0.

use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU32, ConstU64},
	PalletId,
};
use frame_system::EnsureRoot;
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32, BuildStorage,
};

/// The AccountId alias in this test module.
pub(crate) type AccountId = AccountId32;
pub(crate) type Balance = u128;
pub(crate) use crate as btc_ledger;

type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub struct Test
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		BtcLedger: btc_ledger::{Pallet, Call, Storage, Config<T>, Event<T>}
	}
);

parameter_types! {
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(
			frame_support::weights::Weight::from_parts(1024, u64::MAX),
		);
	pub const BtcLedgerPalletId: PalletId = PalletId(*b"bevmtrsy");
}
impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = BlockWeights;
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type Nonce = u64;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

impl crate::Config for Test {
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type CouncilOrigin = EnsureRoot<AccountId>;
	type PalletId = BtcLedgerPalletId;
}

pub const ALICE: [u8; 32] = [1u8; 32];
pub const BOB: [u8; 32] = [2u8; 32];
pub const CHARLIE: [u8; 32] = [3u8; 32];

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

	btc_ledger::GenesisConfig::<Test> { balances: vec![(ALICE.into(), 10), (BOB.into(), 20)] }
		.assimilate_storage(&mut t)
		.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::inc_providers(&ALICE.into());
		System::inc_providers(&BOB.into());
		System::set_block_number(1)
	});

	ext
}
