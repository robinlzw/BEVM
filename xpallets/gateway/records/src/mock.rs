// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use frame_support::{parameter_types, sp_io, traits::GenesisBuild};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

use chainx_primitives::AssetId;
use xpallet_assets::AssetRestrictions;
use xpallet_assets_registrar::AssetInfo;

pub use xp_protocol::{X_BTC, X_ETH};

use crate::{self as xpallet_gateway_records, *};

/// The AccountId alias in this test module.
pub(crate) type AccountId = u64;
pub(crate) type BlockNumber = u64;
pub(crate) type Balance = u128;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        XAssetsRegistrar: xpallet_assets_registrar::{Pallet, Call, Storage, Event<T>, Config},
        XAssets: xpallet_assets::{Pallet, Call, Storage, Event<T>, Config<T>},
        XGatewayRecords: xpallet_gateway_records::{Pallet, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 0;
    pub const MaxReserves: u32 = 50;
}
impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type Balance = Balance;
    type DustRemoval = ();
    type Event = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type ReserveIdentifier = [u8; 8];
    type MaxReserves = MaxReserves;
}

// assets
parameter_types! {
    pub const ChainXAssetId: AssetId = 0;
}

impl xpallet_assets_registrar::Config for Test {
    type Event = ();
    type NativeAssetId = ChainXAssetId;
    type RegistrarHandler = ();
    type WeightInfo = ();
}

impl xpallet_assets::Config for Test {
    type Event = ();
    type Currency = Balances;
    type TreasuryAccount = ();
    type OnCreatedAccount = frame_system::Provider<Test>;
    type OnAssetChanged = ();
    type WeightInfo = ();
}

impl Config for Test {
    type Event = ();
    type WeightInfo = ();
}

pub type XRecordsErr = Error<Test>;

pub(crate) fn btc() -> (AssetId, AssetInfo, AssetRestrictions) {
    (
        X_BTC,
        AssetInfo::new::<Test>(
            b"X-BTC".to_vec(),
            b"X-BTC".to_vec(),
            Chain::Bitcoin,
            8,
            b"ChainX's cross-chain Bitcoin".to_vec(),
        )
        .unwrap(),
        AssetRestrictions::DESTROY_USABLE,
    )
}
pub(crate) fn eth() -> (AssetId, AssetInfo, AssetRestrictions) {
    (
        X_ETH,
        AssetInfo::new::<Test>(
            b"X-ETH".to_vec(),
            b"X-ETH".to_vec(),
            Chain::Ethereum,
            17,
            b"ChainX's cross-chain Ethereum".to_vec(),
        )
        .unwrap(),
        AssetRestrictions::DESTROY_USABLE,
    )
}

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARLIE: AccountId = 3;
pub const DAVE: AccountId = 4;

pub struct ExtBuilder;
impl Default for ExtBuilder {
    fn default() -> Self {
        Self
    }
}
impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let mut storage = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        let btc_assets = btc();
        let eth_assets = eth();
        let assets = vec![
            (btc_assets.0, btc_assets.1, btc_assets.2, true, true),
            (eth_assets.0, eth_assets.1, eth_assets.2, true, true),
        ];
        let mut endowed = BTreeMap::new();
        let endowed_info = vec![(ALICE, 100), (BOB, 200), (CHARLIE, 300), (DAVE, 400)];
        endowed.insert(btc_assets.0, endowed_info.clone());
        endowed.insert(eth_assets.0, endowed_info);

        let mut init_assets = vec![];
        let mut assets_restrictions = vec![];
        for (a, b, c, d, e) in assets {
            init_assets.push((a, b, d, e));
            assets_restrictions.push((a, c))
        }

        GenesisBuild::<Test>::assimilate_storage(
            &xpallet_assets_registrar::GenesisConfig {
                assets: init_assets,
            },
            &mut storage,
        )
        .unwrap();

        let _ = xpallet_assets::GenesisConfig::<Test> {
            assets_restrictions,
            endowed,
        }
        .assimilate_storage(&mut storage);

        sp_io::TestExternalities::new(storage)
    }
    pub fn build_and_execute(self, test: impl FnOnce()) {
        let mut ext = self.build();
        ext.execute_with(|| System::set_block_number(1));
        ext.execute_with(test);
    }
}

/*
这段代码是 ChainX 项目中用于测试的 Substrate 运行时环境的构建和配置.它定义了一个名为 `Test` 的测试运行时,
其中包含了多个 pallets,包括 `System`,`Balances`,`XAssetsRegistrar`,`XAssets` 和 `XGatewayRecords`.
此外,它还定义了一些测试用的常量,类型别名和辅助函数.

### 主要组件

- **Test 运行时**: 通过 `construct_runtime!` 宏构建的测试运行时环境,包含了 ChainX 项目中使用的 pallets.

- **参数类型**: 使用 `parameter_types!` 宏定义了一系列的参数类型,如账户 ID,区块编号,余额等.

- **配置 trait**: 实现了 `frame_system::Config`,`pallet_balances::Config`,`xpallet_assets_registrar::Config`,
`xpallet_assets::Config` 等 trait,为测试运行时提供了必要的配置.

- **事件**: 定义了测试运行时中使用的事件类型.

- **资产配置**: 定义了比特币(`X_BTC`)和以太坊(`X_ETH`)资产的信息和限制.

- **测试账户**: 定义了测试账户 `ALICE`,`BOB`,`CHARLIE` 和 `DAVE` 的账户 ID.

### 辅助函数和结构

- **ExtBuilder**: 一个结构体,用于构建和执行测试.它提供了 `build` 方法来创建测试外部环境,以及 `build_and_execute` 方法来执行测试代码.

### 总结

这段代码为 ChainX 项目的 Substrate 运行时提供了一个完整的测试环境,允许开发者在其中执行测试用例,验证 pallets 的功能和性能.
通过模拟运行时环境,开发者可以在不部署到实际区块链的情况下进行开发和测试.
*/
