// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

pub use crate as xassets_bridge;
pub use xassets_bridge::{AssetId, Config, Error, Event as XAssetsBridgeEvent};

use frame_support::traits::ConstU32;
use frame_support::{parameter_types, traits::GenesisBuild};
use frame_system as system;
use sp_core::{H160, H256};
pub use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    AccountId32,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Evm: pallet_evm::{Pallet, Call, Storage, Config, Event<T>},
        XAssetsRegistrar: xpallet_assets_registrar::{Pallet, Call, Config, Storage, Event<T>},
        XAssets: xpallet_assets::{Pallet, Call, Config<T>, Storage, Event<T>},
        XAssetsBridge: xassets_bridge::{Pallet, Call, Storage, Config<T>, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 44;
}

impl system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId32;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<5>;
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
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
}

parameter_types! {
    pub const AssetDeposit: u64 = 1;
    pub const ApprovalDeposit: u64 = 1;
    pub const StringLimit: u32 = 50;
    pub const MetadataDepositBase: u64 = 1;
    pub const MetadataDepositPerByte: u64 = 1;

    // 0x1111111111111111111111111111111111111111
    pub EvmCaller: H160 = H160::from_slice(&[17u8;20][..]);
    pub ClaimBond: u128 = 2;
}

parameter_types! {
    pub const ChainXAssetId: AssetId = 0;
}

impl xpallet_assets_registrar::Config for Test {
    type Event = Event;
    type NativeAssetId = ChainXAssetId;
    type RegistrarHandler = ();
    type WeightInfo = ();
}

impl xpallet_assets::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type TreasuryAccount = ();
    type OnCreatedAccount = frame_system::Provider<Test>;
    type OnAssetChanged = ();
    type WeightInfo = ();
}

impl pallet_evm::Config for Test {
    type FeeCalculator = ();
    type GasWeightMapping = ();
    type CallOrigin = pallet_evm::EnsureAddressRoot<Self::AccountId>;
    type WithdrawOrigin = pallet_evm::EnsureAddressNever<Self::AccountId>;
    type AddressMapping = pallet_evm::HashedAddressMapping<BlakeTwo256>;
    type Currency = Balances;
    type Runner = pallet_evm::runner::stack::Runner<Self>;
    type Event = Event;
    type PrecompilesType = ();
    type PrecompilesValue = ();
    type ChainId = ();
    type BlockGasLimit = ();
    type OnChargeTransaction = ();
    type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
    type FindAuthor = ();
    type WeightInfo = ();
}

impl xassets_bridge::Config for Test {
    type Event = Event;
    type EvmCaller = EvmCaller;
    type ClaimBond = ClaimBond;
}

pub const ALICE: [u8; 32] = [1u8; 32];
pub const BOB: [u8; 32] = [2u8; 32];

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(ALICE.into(), 1000), (BOB.into(), 1000)],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    xassets_bridge::GenesisConfig::<Test> {
        admin_key: Some(ALICE.into()),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));

    ext
}

pub(crate) fn last_event() -> Event {
    system::Pallet::<Test>::events()
        .pop()
        .expect("Event expected")
        .event
}

pub(crate) fn expect_event<E: Into<Event>>(e: E) {
    assert_eq!(last_event(), e.into());
}

/*
这段代码是一个用于测试的Substrate框架运行时环境配置,它设置了一个模拟的区块链网络,用于测试`xassets_bridge` pallet.
这个pallet是ChainX项目的一部分,负责在Substrate资产和以太坊ERC20代币之间建立桥接.以下是代码中定义的一些关键组件和它们的功能:

### 模块和配置

- `xassets_bridge`:桥接pallet,用于在Substrate资产和以太坊ERC20代币之间进行互操作.
- `AssetId`:用于唯一标识资产的类型.
- `Config`:包含了pallet配置的类型.
- `Error`:定义了可能发生的错误类型.
- `Event`:定义了pallet可能发出的事件类型.

### 运行时构造

使用`frame_support::construct_runtime!`宏来构建一个模拟的运行时环境,其中包括了系统(System),时间戳(Timestamp),余额(Balances),
以太坊虚拟机(Evm),资产注册(XAssetsRegistrar),资产处理(XAssets)和资产桥接(XAssetsBridge)等模块.

### 参数配置

- `BlockHashCount`:区块哈希计数.
- `SS58Prefix`:SS58地址前缀.
- `ExistentialDeposit`:创建新账户所需的最小存款.
- `MinimumPeriod`:时间戳模块的最小周期.
- `AssetDeposit`,`ApprovalDeposit`,`StringLimit`,`MetadataDepositBase`,`MetadataDepositPerByte`:与资产注册相关的存款和费用参数.
- `EvmCaller`:以太坊调用者的地址.
- `ClaimBond`:声明以太坊地址映射到Substrate账户所需的保证金.

### 配置实现

为Substrate框架的核心模块(如系统,时间戳,余额等)提供了测试配置.这些配置定义了区块链的参数,如账户ID类型,事件类型,余额类型等.

### 测试辅助函数

- `new_test_ext`:创建并返回一个新的测试外部状态,用于模拟区块链网络的状态.
- `last_event`:获取并返回最后一个发出的事件.
- `expect_event`:断言最后一个事件是否符合预期的事件类型.

整体来看,这段代码为`xassets_bridge` pallet提供了一个完整的测试环境,允许开发者在不依赖实际区块链网络的情况下进行测试.
*/