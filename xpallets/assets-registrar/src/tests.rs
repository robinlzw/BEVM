// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use frame_support::{assert_noop, assert_ok, parameter_types, sp_io, traits::GenesisBuild};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

use chainx_primitives::AssetId;
use xp_protocol::X_BTC;

use crate::{self as xpallet_assets_registrar, AssetInfo, Chain, Config, Error};

/// The AccountId alias in this test module.
pub(crate) type BlockNumber = u64;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        XAssetsRegistrar: xpallet_assets_registrar::{Pallet, Call, Config, Storage, Event<T>},
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
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const ChainXAssetId: AssetId = 0;
}

impl Config for Test {
    type Event = Event;
    type NativeAssetId = ChainXAssetId;
    type RegistrarHandler = ();
    type WeightInfo = ();
}

pub struct ExtBuilder;
impl Default for ExtBuilder {
    fn default() -> Self {
        Self
    }
}

pub(crate) fn btc() -> (AssetId, AssetInfo) {
    (
        xp_protocol::X_BTC,
        AssetInfo::new::<Test>(
            b"X-BTC".to_vec(),
            b"X-BTC".to_vec(),
            Chain::Bitcoin,
            8,
            b"ChainX's cross-chain Bitcoin".to_vec(),
        )
        .unwrap(),
    )
}

impl ExtBuilder {
    pub fn build(self, assets: Vec<(AssetId, AssetInfo, bool, bool)>) -> sp_io::TestExternalities {
        let mut storage = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        GenesisBuild::<Test>::assimilate_storage(
            &xpallet_assets_registrar::GenesisConfig { assets },
            &mut storage,
        )
        .unwrap();

        sp_io::TestExternalities::new(storage)
    }

    pub fn build_with(self) -> sp_io::TestExternalities {
        let btc_assets = btc();
        let assets = vec![(btc_assets.0, btc_assets.1, true, true)];
        self.build(assets)
    }

    pub fn build_and_execute(self, test: impl FnOnce()) {
        let btc_assets = btc();
        let assets = vec![(btc_assets.0, btc_assets.1, true, true)];
        let mut ext = self.build(assets);
        ext.execute_with(|| System::set_block_number(1));
        ext.execute_with(test);
    }
}

pub type Err = Error<Test>;

#[test]
fn test_register() {
    ExtBuilder::default().build_and_execute(|| {
        let abc_id = 100;
        let abc_assets = (
            abc_id,
            AssetInfo::new::<Test>(
                b"ABC".to_vec(),
                b"ABC".to_vec(),
                Chain::Bitcoin,
                8,
                b"abc".to_vec(),
            )
            .unwrap(),
        );
        assert_ok!(XAssetsRegistrar::register(
            Origin::root(),
            abc_assets.0,
            abc_assets.1.clone(),
            false,
            false
        ));
        assert_noop!(
            XAssetsRegistrar::register(Origin::root(), abc_assets.0, abc_assets.1, false, false),
            Err::AssetAlreadyExists
        );

        assert_noop!(
            XAssetsRegistrar::get_asset_info(&abc_id),
            Err::AssetIsInvalid
        );

        assert_ok!(XAssetsRegistrar::recover(Origin::root(), abc_id, true));
        assert!(XAssetsRegistrar::get_asset_info(&abc_id).is_ok());

        assert_noop!(
            XAssetsRegistrar::deregister(Origin::root(), 10000),
            Err::AssetIsInvalid
        );
        assert_noop!(
            XAssetsRegistrar::recover(Origin::root(), X_BTC, true),
            Err::AssetAlreadyValid
        );

        assert_ok!(XAssetsRegistrar::deregister(Origin::root(), X_BTC));
        assert_noop!(
            XAssetsRegistrar::get_asset_info(&X_BTC),
            Err::AssetIsInvalid
        );
    })
}

/*
这段代码是一个用于测试`xpallet_assets_registrar`模块的Rust单元测试.`xpallet_assets_registrar`是一个Substrate框架的运行时模块,
用于注册和管理跨链资产.测试的主要目的是验证资产注册,注销,恢复等操作的正确性和边界条件.

以下是代码中的关键部分和它们的功能:

1. **测试环境构建**:使用`frame_support`库中的`construct_runtime!`宏构建了一个名为`Test`的运行时环境,
其中包含了`System`和`XAssetsRegistrar`两个Pallet.这个环境模拟了一个真实的Substrate区块链,用于测试.

2. **参数配置**:通过`parameter_types!`宏定义了一些参数类型,例如`BlockHashCount`和`SS58Prefix`.
同时,实现了`frame_system::Config` trait,为测试环境提供了必要的配置.

3. **模拟外部环境**:`ExtBuilder`结构体用于构建测试的外部环境.它可以创建一个包含特定资产的存储状态,
并构建一个`sp_io::TestExternalities`对象,这个对象模拟了区块链的状态.

4. **测试用例**:`test_register`函数是一个测试用例,它执行了一系列的注册,注销和恢复操作,并验证了操作的结果.
测试用例使用了`assert_ok!`和`assert_noop!`宏来检查调用是否成功或失败,以及预期的错误是否被返回.

5. **模拟资产**:`btc`函数创建了一个模拟的比特币资产(`X-BTC`),并返回了资产ID和资产信息.这个资产信息用于在测试中注册新的资产.

6. **执行测试**:`ExtBuilder`的`build_and_execute`方法用于构建测试环境并执行一个给定的测试闭包.这允许在测试环境中执行代码并观察其效果.

整体而言,这段代码展示了如何使用Substrate框架的工具和宏来构建和测试区块链运行时模块.通过模拟区块链环境和执行测试用例,开发者可以确保他们的代码在上线前是健壮和可靠的.
*/
