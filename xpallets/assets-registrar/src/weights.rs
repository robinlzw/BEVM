// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

//! Weights for xpallet_assets_registrar
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-05-13, STEPS: 50, REPEAT: 20, LOW RANGE: [], HIGH RANGE: []
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("benchmarks"), DB CACHE: 1024

// Executed Command:
// ./target/release/chainx
// benchmark
// --chain=benchmarks
// --steps=50
// --repeat=20
// --pallet=xpallet_assets_registrar
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./xpallets/assets-registrar/src/weights.rs
// --template=./scripts/xpallet-weight-template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for xpallet_assets_registrar.
pub trait WeightInfo {
    fn register() -> Weight;
    fn deregister() -> Weight;
    fn recover() -> Weight;
    fn update_asset_info() -> Weight;
}

/// Weights for xpallet_assets_registrar using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn register() -> Weight {
        (54_607_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
    }
    fn deregister() -> Weight {
        (35_301_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn recover() -> Weight {
        (45_382_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    fn update_asset_info() -> Weight {
        (13_710_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn register() -> Weight {
        (54_607_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(6 as Weight))
    }
    fn deregister() -> Weight {
        (35_301_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    fn recover() -> Weight {
        (45_382_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
    }
    fn update_asset_info() -> Weight {
        (13_710_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
}

/*
这段代码是为 `xpallet_assets_registrar` 模块定义的权重信息,它指定了该模块中各种操作的权重(即消耗的计算和存储资源).
权重信息是Substrate框架中的一个重要组成部分,用于确定交易费用和区块生产者的奖励.
这些权重是通过使用Substrate基准测试工具自动生成的,反映了在特定硬件和配置下执行操作所需的资源.

### 权重信息

- **`WeightInfo` trait**: 定义了 `xpallet_assets_registrar` 需要的权重函数.
这些函数包括 `register`(注册资产),`deregister`(注销资产),`recover`(恢复已注销的资产)和 `update_asset_info`(更新资产信息).

- **`SubstrateWeight` struct**: 实现了 `WeightInfo` trait,为特定类型的 `frame_system::Config` 提供了权重值.
这些权重值是基于Substrate节点和推荐硬件的平均资源消耗.

### 权重计算

每个操作的权重是通过以下方式计算的:

- **基础权重**: 一个固定的权重值,反映了执行操作所需的基本计算资源.
- **数据库权重**: 通过 `T::DbWeight::get()` 获取的权重,表示操作对数据库的读取和写入操作的资源消耗.这些值依赖于具体的配置和系统状态.

### 兼容性和测试

- **空实现**: 为了向后兼容和测试,`WeightInfo` trait 还有一个空实现,它直接使用 `RocksDbWeight` 而不是依赖于类型 `T`.
这允许在没有特定 `frame_system::Config` 上下文的情况下使用权重.

### 基准测试

这些权重值是通过执行基准测试生成的,基准测试是在特定的硬件和配置下进行的.这些测试提供了操作的平均资源消耗,这些数据对于设置交易费用和确保网络的稳定运行至关重要.

### 总结

权重信息是Substrate框架中用于管理交易费用和资源分配的关键组成部分.通过为每个操作定义明确的权重,可以确保区块链网络的公平性和可持续性.
这些权重值还可以帮助节点操作者和用户理解交易成本,并据此做出决策.
*/
