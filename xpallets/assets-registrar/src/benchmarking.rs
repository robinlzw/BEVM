// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;

use chainx_primitives::AssetId;

use crate::{AssetInfo, AssetInfoOf, AssetOnline, Call, Chain, Config, Pallet};

const ASSET_ID: AssetId = 8888;

fn b_asset_info_test_data<T: Config>() -> AssetInfo {
    AssetInfo::new::<T>(
        b"token".to_vec(),
        b"token_name".to_vec(),
        Chain::Bitcoin,
        8,
        b"token_desc".to_vec(),
    )
    .unwrap()
}

benchmarks! {
    register {
        let asset_info = b_asset_info_test_data::<T>();
    }: _(RawOrigin::Root, ASSET_ID, asset_info.clone(), true, true)
    verify {
        assert_eq!(AssetInfoOf::<T>::get(ASSET_ID), Some(asset_info));
    }

    deregister {
        let asset_info = b_asset_info_test_data::<T>();
        Pallet::<T>::register(RawOrigin::Root.into(), ASSET_ID, asset_info, true, true)?;
    }: _(RawOrigin::Root, ASSET_ID)
    verify {
        assert!(!AssetOnline::<T>::get(ASSET_ID));
    }

    recover {
        let asset_info = b_asset_info_test_data::<T>();
        Pallet::<T>::register(RawOrigin::Root.into(), ASSET_ID, asset_info, true, true)?;
        Pallet::<T>::deregister(RawOrigin::Root.into(), ASSET_ID)?;
    }: _(RawOrigin::Root, ASSET_ID, true)
    verify {
        assert!(AssetOnline::<T>::get(ASSET_ID));
    }

    update_asset_info {
        let asset_info = b_asset_info_test_data::<T>();
        Pallet::<T>::register(RawOrigin::Root.into(), ASSET_ID, asset_info.clone(), true, true)?;
    }: _(
        RawOrigin::Root,
        ASSET_ID,
        Some(b"new_token".to_vec()),
        Some(b"new_token_name".to_vec()),
        Some(b"new_desc".to_vec())
    )
    verify {
        let mut new_asset_info = asset_info;
        new_asset_info.set_token(b"new_token".to_vec());
        new_asset_info.set_token_name(b"new_token_name".to_vec());
        new_asset_info.set_desc(b"new_desc".to_vec());
        assert_eq!(AssetInfoOf::<T>::get(ASSET_ID).unwrap(), new_asset_info);
    }
}

impl_benchmark_test_suite!(
    Pallet,
    crate::tests::ExtBuilder::default().build_with(),
    crate::tests::Test,
);

/*
这段代码是一个Rust语言编写的基准测试套件,用于评估区块链系统中资产管理功能的性能.
这些测试是用`frame_benchmarking`库实现的,它是Substrate框架的一部分,专门用于开发区块链应用.

代码中定义了几个基准测试,每个测试都对应一个特定的资产管理操作:

1. **register**:测试注册新资产的性能.它首先创建一个`AssetInfo`实例作为测试数据,
然后使用`RawOrigin::Root`(代表系统根账户)调用`register`函数来注册资产.测试验证注册后的资产信息是否正确存储.

2. **deregister**:测试注销已注册资产的性能.它首先注册一个资产,然后调用`deregister`函数来注销该资产.测试验证资产是否已从在线状态中移除.

3. **recover**:测试恢复已注销资产的性能.它首先注册一个资产,然后注销它,最后调用`recover`函数来恢复该资产.测试验证资产是否成功恢复到在线状态.

4. **update_asset_info**:测试更新资产信息的性能.它首先注册一个资产,然后调用`update_asset_info`函数来更新该资产的名称,描述等信息.测试验证更新后的资产信息是否与预期相符.

每个基准测试都有一个`verify`阶段,用于验证测试的结果是否符合预期.例如,`register`测试验证注册的资产信息是否正确存储在状态中.

最后,`impl_benchmark_test_suite!`宏用于实现一个基准测试套件,它指定了测试的配置环境和要测试的`Pallet`(子模块).
在这个例子中,使用`ExtBuilder`来构建测试环境,`Test`是测试的配置类型.
*/
