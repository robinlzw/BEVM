// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use std::collections::BTreeMap;

use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

use frame_support::{parameter_types, sp_io, traits::GenesisBuild};

use chainx_primitives::AssetId;
pub use xp_protocol::X_BTC;

use crate::{self as xpallet_assets, AssetInfo, AssetRestrictions, Chain, Config, Error};

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
        XAssetsRegistrar: xpallet_assets_registrar::{Pallet, Call, Config, Storage, Event<T>},
        XAssets: xpallet_assets::{Pallet, Call, Config<T>, Storage, Event<T>},
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
    type Event = Event;
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
    pub const ExistentialDeposit: u64 = 1;
    pub const MaxReserves: u32 = 50;
}
impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type ReserveIdentifier = [u8; 8];
    type MaxReserves = MaxReserves;
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

impl Config for Test {
    type Event = Event;
    type Currency = Balances;
    type TreasuryAccount = ();
    type OnCreatedAccount = frame_system::Provider<Test>;
    type OnAssetChanged = ();
    type WeightInfo = ();
}

pub struct ExtBuilder;

impl Default for ExtBuilder {
    fn default() -> Self {
        Self
    }
}

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

impl ExtBuilder {
    pub fn build(
        self,
        assets: Vec<(AssetId, AssetInfo, AssetRestrictions, bool, bool)>,
        endowed: BTreeMap<AssetId, Vec<(AccountId, Balance)>>,
    ) -> sp_io::TestExternalities {
        let _ = env_logger::try_init();
        let mut storage = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

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
    pub fn build_default(self) -> sp_io::TestExternalities {
        let btc_assets = btc();
        let assets = vec![(btc_assets.0, btc_assets.1, btc_assets.2, true, true)];
        let mut endowed = BTreeMap::new();
        let endowed_info = vec![(ALICE, 100), (BOB, 200), (CHARLIE, 300), (DAVE, 400)];
        endowed.insert(btc_assets.0, endowed_info);

        self.build(assets, endowed)
    }
    pub fn build_and_execute(self, test: impl FnOnce()) {
        let mut ext = self.build_default();
        ext.execute_with(|| System::set_block_number(1));
        ext.execute_with(test);
    }

    pub fn build_no_endowed_and_execute(self, test: impl FnOnce()) {
        let btc_assets = btc();
        let assets = vec![(btc_assets.0, btc_assets.1, btc_assets.2, true, true)];
        let mut ext = self.build(assets, Default::default());
        ext.execute_with(|| System::set_block_number(1));
        ext.execute_with(test);
    }
}

pub type XAssetsErr = Error<Test>;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARLIE: AccountId = 3;
pub const DAVE: AccountId = 4;

/*
这段代码是一个用于区块链测试的Rust程序,它定义了一个名为`Test`的测试环境,其中包括了一些基本的区块链参数和配置.
这个测试环境是为了模拟一个区块链网络,并允许开发者在这个环境中测试他们的智能合约和区块链逻辑.下面是代码中一些关键部分的中文解释:

1. **测试环境配置(Test Environment Configuration)**:定义了`AccountId`,`BlockNumber`和`Balance`等类型别名,
用于模拟区块链中的账户ID,区块编号和余额.

2. **运行时构建(Runtime Construction)**:使用`frame_support`库构建了一个模拟的区块链运行时环境,
包括系统(`System`),余额(`Balances`),资产注册(`XAssetsRegistrar`)和资产处理(`XAssets`)等模块.

3. **参数类型定义(Parameter Types Definition)**:设置了一些区块链的参数类型,
例如区块哈希计数(`BlockHashCount`),SS58前缀(`SS58Prefix`)和存在的存款(`ExistentialDeposit`)等.

4. **模块配置(Module Configuration)**:为`frame_system`和`pallet_balances`等模块提供了具体的配置,包括事件类型,余额类型,账户存储等.

5. **资产ID定义(Asset ID Definition)**:定义了一个常量`ChainXAssetId`,用于表示特定的资产ID.

6. **测试构建器(Test Builder)**:创建了一个`ExtBuilder`结构体,它可以帮助构建测试环境,包括初始化资产,设置账户余额等.
`build`函数用于根据提供的资产信息和账户余额构建测试外部状态.`build_default`函数提供了一个默认的构建方法,
它创建了一个比特币资产(`X-BTC`)并分配了一些余额给几个预定义的账户(如ALICE,BOB等).
`build_and_execute`和`build_no_endowed_and_execute`函数用于构建测试环境并执行一个测试函数.

7. **错误类型定义(Error Type Definition)**:定义了一个`XAssetsErr`类型,它是`Error`类型针对`Test`环境的别名.

8. **预定义的账户(Predefined Accounts)**:定义了一些预定义的账户常量,如ALICE,BOB,CHARLIE和DAVE,这些账户将在测试中使用.

总的来说,这段代码为区块链资产处理和交易提供了一个模拟测试环境,允许开发者在不依赖实际区块链网络的情况下进行测试.
*/