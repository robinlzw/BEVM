// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstU32, ConstU64},
    PalletId,
};
use frame_system::EnsureRoot;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    AccountId32,
};

/// The AccountId alias in this test module.
pub(crate) type AccountId = AccountId32;
pub(crate) type BlockNumber = u64;
pub(crate) type Balance = u128;
pub(crate) use crate as btc_ledger;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        BtcLedger: btc_ledger::{Pallet, Call, Storage, Config<T>, Event<T>}
    }
);

parameter_types! {
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(1024);
    pub const BtcLedgerPalletId: PalletId = PalletId(*b"pcx/trsy");
}
impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = BlockWeights;
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Call = Call;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
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
    type Event = Event;
    type CouncilOrigin = EnsureRoot<AccountId>;
    type PalletId = BtcLedgerPalletId;
}

pub const ALICE: [u8; 32] = [1u8; 32];
pub const BOB: [u8; 32] = [2u8; 32];
pub const CHARLIE: [u8; 32] = [3u8; 32];

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    btc_ledger::GenesisConfig::<Test> {
        balances: vec![(ALICE.into(), 10), (BOB.into(), 20)],
    }
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

/*

这段代码是用于设置和运行 Substrate 框架中的 `btc_ledger` 模块的测试环境.
它定义了测试用的运行时环境,参数类型,账户 ID 别名,块号和余额类型,并提供了构建和执行测试所需的所有必要组件.

### 核心组件

1. **AccountId**: 用于测试的账户 ID 类型,这里使用了 `AccountId32`.

2. **BlockNumber**: 用于测试的块号类型,这里使用了 `u64`.

3. **Balance**: 用于测试的余额类型,这里使用了 `u128`.

4. **UncheckedExtrinsic** 和 **Block**: 分别用于模拟未检查的交易和区块.

5. **construct_runtime**: 宏用于构建测试运行时环境,包括 `System` 和 `BtcLedger` 两个 Pallet.

### 参数类型

- **BlockWeights**: 定义了区块的权重限制.

- **BtcLedgerPalletId**: 定义了 `btc_ledger` Pallet 的 ID.

### 系统配置

- **frame_system::Config**: 实现了 `frame_system` 配置 trait,定义了测试环境的基本参数,如调用过滤器,区块权重,账户 ID 等.

### `btc_ledger` 配置

- **crate::Config**: 实现了 `btc_ledger` 配置 trait,指定了余额类型,事件类型,理事会起源和 Pallet ID.

### 测试账户和余额

- **ALICE, BOB, CHARLIE**: 定义了三个测试账户的公钥.

- **new_test_ext**: 函数用于创建测试外部环境,初始化存储,设置账户余额,并执行一些初始操作,如增加提供者和设置块号.

整体而言,这段代码为 `btc_ledger` 模块提供了一个完整的测试环境,允许开发者在其中执行测试用例,验证模块的功能和性能.
*/
