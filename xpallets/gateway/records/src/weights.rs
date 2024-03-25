// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

//! Weights for xpallet_gateway_records
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-05-13, STEPS: 50, REPEAT: 20, LOW RANGE: [], HIGH RANGE: []
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("benchmarks"), DB CACHE: 1024

// Executed Command:
// ./target/release/chainx
// benchmark
// --chain=benchmarks
// --steps=50
// --repeat=20
// --pallet=xpallet_gateway_records
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./xpallets/gateway/records/src/weights.rs
// --template=./scripts/xpallet-weight-template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for xpallet_gateway_records.
pub trait WeightInfo {
    fn root_deposit() -> Weight;
    fn root_withdraw() -> Weight;
    fn set_withdrawal_state() -> Weight;
    fn set_withdrawal_state_list(u: u32) -> Weight;
}

/// Weights for xpallet_gateway_records using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn root_deposit() -> Weight {
        (185_417_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(8 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    fn root_withdraw() -> Weight {
        (109_687_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    fn set_withdrawal_state() -> Weight {
        (121_624_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(8 as Weight))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
    }
    fn set_withdrawal_state_list(_u: u32) -> Weight {
        (113_045_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(8 as Weight))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn root_deposit() -> Weight {
        (185_417_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(8 as Weight))
            .saturating_add(RocksDbWeight::get().writes(5 as Weight))
    }
    fn root_withdraw() -> Weight {
        (109_687_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(5 as Weight))
            .saturating_add(RocksDbWeight::get().writes(5 as Weight))
    }
    fn set_withdrawal_state() -> Weight {
        (121_624_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(8 as Weight))
            .saturating_add(RocksDbWeight::get().writes(6 as Weight))
    }
    fn set_withdrawal_state_list(_u: u32) -> Weight {
        (113_045_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(8 as Weight))
            .saturating_add(RocksDbWeight::get().writes(6 as Weight))
    }
}

/*
这段代码是一个Substrate框架中的权重(Weight)信息定义文件,用于`xpallet_gateway_records`模块.这个文件是使用Substrate的基准测试工具自动生成的,用于确定执行特定操作所需的计算和存储资源.这些权重信息对于调整交易费用和优化网络性能至关重要.

文件中包含了以下关键部分:

1. 文件头部的注释:
   - 提供了版权声明和许可证信息.
   - 描述了文件是如何使用Substrate基准测试CLI工具自动生成的,包括生成日期,步骤数,重复次数,执行环境和配置参数.
   - 显示了执行的命令行指令,用于生成这个权重文件.

2. 权重信息接口`WeightInfo`:
   - 定义了`xpallet_gateway_records`模块需要的权重函数.这些函数包括:
     - `root_deposit()`:根账户执行存款操作的权重.
     - `root_withdraw()`:根账户执行提现操作的权重.
     - `set_withdrawal_state()`:设置提现状态的权重.
     - `set_withdrawal_state_list(u: u32)`:设置提现状态列表的权重,其中`u`是列表的大小.

3. 权重实现`SubstrateWeight<T>`:
   - 为使用Substrate节点和推荐硬件的`xpallet_gateway_records`模块提供了权重实现.
   - 实现了`WeightInfo`接口,为每个操作计算权重.权重是基于常量和数据库读写操作的权重组合而成的.

4. 兼容性和测试的权重信息实现:
   - 为了向后兼容和测试,为`()`(空元组)实现了`WeightInfo`接口.这允许在没有特定类型参数的情况下使用这些权重函数.

权重值是以"Weight"为单位的,这是一个可以表示计算和存储负担的抽象单位.这些权重值是在特定的硬件和网络条件下测量得到的,
以便在实际的区块链网络中使用.通过这些权重信息,网络可以对交易进行定价,并确保网络的运行效率和公平性.
*/
