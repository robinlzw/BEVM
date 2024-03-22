// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

//! Weights for xpallet_assets
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-05-13, STEPS: 50, REPEAT: 20, LOW RANGE: [], HIGH RANGE: []
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("benchmarks"), DB CACHE: 1024

// Executed Command:
// ./target/release/chainx
// benchmark
// --chain=benchmarks
// --steps=50
// --repeat=20
// --pallet=xpallet_assets
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./xpallets/assets/src/weights.rs
// --template=./scripts/xpallet-weight-template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for xpallet_assets.
pub trait WeightInfo {
    fn transfer() -> Weight;
    fn force_transfer() -> Weight;
    fn set_balance(n: u32) -> Weight;
    fn set_asset_limit() -> Weight;
}

/// Weights for xpallet_assets using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn transfer() -> Weight {
        (160_552_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(8 as Weight))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
    }
    fn force_transfer() -> Weight {
        (158_525_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(8 as Weight))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
    }
    fn set_balance(_n: u32) -> Weight {
        (170_937_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    fn set_asset_limit() -> Weight {
        (10_093_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn transfer() -> Weight {
        (160_552_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(8 as Weight))
            .saturating_add(RocksDbWeight::get().writes(6 as Weight))
    }
    fn force_transfer() -> Weight {
        (158_525_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(8 as Weight))
            .saturating_add(RocksDbWeight::get().writes(6 as Weight))
    }
    fn set_balance(_n: u32) -> Weight {
        (170_937_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
    }
    fn set_asset_limit() -> Weight {
        (10_093_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
}

/*
这段代码是Substrate框架中用于资产模块(`xpallet_assets`)的权重配置.
权重(Weight)是Substrate中用于衡量交易和区块生产操作复杂性的一个概念.
每个操作都有一个与之关联的权重,这有助于区块链节点估计执行操作所需的计算资源,并据此调整交易费用.

### 权重配置文件的生成

这个权重配置文件是通过Substrate的基准测试工具自动生成的.基准测试工具运行了一系列的操作,
并测量了它们的执行时间,以及对数据库的读写次数.这些数据被用来估计每个操作的权重.
生成这个文件的命令在注释的开始部分给出,包括了使用的参数和条件.

### 权重函数

`WeightInfo` trait 定义了几种与资产转移相关的操作的权重函数,包括:

- `transfer()`: 普通转账操作的权重.
- `force_transfer()`: 强制转账操作的权重.
- `set_balance()`: 设置账户余额的权重,这里的 `n` 参数表示受影响的账户数量.
- `set_asset_limit()`: 设置资产限制的权重.

### 权重实现

`SubstrateWeight<T>` 结构体实现了 `WeightInfo` trait,为每种操作提供了具体的权重值.
这些值是基于推荐的硬件和Substrate节点的典型性能.
每个权重值都是通过将固定的操作数(如读取和写入数据库的次数)与数据库操作的权重相加来计算的.

### 兼容性和测试

为了向后兼容和测试,还为 `()`(空元组)实现了 `WeightInfo` trait.这个实现使用了 `RocksDbWeight`,
它是Substrate中用于衡量对RocksDB数据库操作的权重的一个常量.

### 总结

这个文件为资产模块中的操作提供了权重配置,这些配置对于区块链的交易费用和区块生产策略至关重要.
通过这些权重,Substrate节点可以有效地管理和优化区块链的性能.
*/
