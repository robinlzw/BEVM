// Copyright 2023 BEVM Project Authors. Licensed under GPL-3.0.

pub use crate as xassets_bridge;
use pallet_evm::FixedGasWeightMapping;
pub use xassets_bridge::{Config, Error, Event as XAssetsBridgeEvent, MappingAssetId};

use frame_support::{parameter_types, traits::ConstU32, PalletId};
use sp_core::{H160, H256};
pub use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32, BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub struct Test
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		EVM: pallet_evm::{Pallet, Call, Storage, Config<T>, Event<T>},
		BtcLedger: xpallet_btc_ledger::{Pallet, Call, Storage, Config<T>, Event<T>},
		XAssetsBridge: xassets_bridge::{Pallet, Call, Storage, Config<T>, Event<T>},
	}
);

parameter_types! {
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(
			frame_support::weights::Weight::from_parts(1024, u64::MAX),
		);
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 44;
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
	type AccountId = AccountId32;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
	pub const MinimumPeriod: u64 = 1000;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

impl pallet_balances::Config for Test {
	type Balance = u128;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = ();
	type RuntimeFreezeReason = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type MaxHolds = ();
}

parameter_types! {
	// 0x1111111111111111111111111111111111111111
	pub EvmCaller: H160 = H160::from_slice(&[17u8;20][..]);
	pub const BtcLedgerPalletId: PalletId = PalletId(*b"bevmtrsy");
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type EvmCaller = EvmCaller;
	type NativeCurrency = Balances;
}

impl xpallet_btc_ledger::Config for Test {
	type Balance = u128;
	type RuntimeEvent = RuntimeEvent;
	type CouncilOrigin = frame_system::EnsureRoot<AccountId32>;
	type PalletId = BtcLedgerPalletId;
}

impl pallet_evm::Config for Test {
	type FeeCalculator = ();
	type GasWeightMapping = FixedGasWeightMapping<Self>;
	type CallOrigin = pallet_evm::EnsureAddressRoot<Self::AccountId>;
	type WithdrawOrigin = pallet_evm::EnsureAddressNever<Self::AccountId>;
	type AddressMapping = pallet_evm::HashedAddressMapping<BlakeTwo256>;
	type Currency = Balances;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = ();
	type PrecompilesValue = ();
	type ChainId = ();
	type BlockGasLimit = ();
	type OnChargeTransaction = ();
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
	type WeightInfo = ();
	type WeightPerGas = ();
	type RuntimeEvent = RuntimeEvent;
	type OnCreate = ();
	type GasLimitPovSizeRatio = ();
	type SuicideQuickClearLimit = ();
	type Timestamp = Timestamp;
}

pub const ALICE: [u8; 32] = [1u8; 32];
pub const BOB: [u8; 32] = [2u8; 32];

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(ALICE.into(), 1000), (BOB.into(), 1000)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	xassets_bridge::GenesisConfig::<Test> { admin_key: Some(ALICE.into()) }
		.assimilate_storage(&mut t)
		.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));

	ext
}

pub(crate) fn last_event() -> RuntimeEvent {
	frame_system::Pallet::<Test>::events().pop().expect("Event expected").event
}

pub(crate) fn expect_event<E: Into<RuntimeEvent>>(e: E) {
	assert_eq!(last_event(), e.into());
}
