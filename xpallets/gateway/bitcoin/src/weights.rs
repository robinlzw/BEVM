// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

//! Weights for xpallet_gateway_bitcoin
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-05-13, STEPS: 50, REPEAT: 20, LOW RANGE: [], HIGH RANGE: []
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("benchmarks"), DB CACHE: 1024

// Executed Command:
// ./target/release/chainx
// benchmark
// --chain=benchmarks
// --steps=50
// --repeat=20
// --pallet=xpallet_gateway_bitcoin
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./xpallets/gateway/bitcoin/src/weights.rs
// --template=./scripts/xpallet-weight-template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for xpallet_gateway_bitcoin.
pub trait WeightInfo {
    fn push_header() -> Weight;
    fn push_transaction() -> Weight;
    fn create_taproot_withdraw_tx() -> Weight;
    fn set_best_index() -> Weight;
    fn set_confirmed_index() -> Weight;
    fn remove_pending() -> Weight;
    fn remove_proposal() -> Weight;
    fn set_btc_withdrawal_fee() -> Weight;
    fn set_btc_deposit_limit() -> Weight;
    fn set_coming_bot() -> Weight;
}

/// Weights for xpallet_gateway_bitcoin using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn push_header() -> Weight {
        (116_466_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(10 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    fn push_transaction() -> Weight {
        (313_612_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(23 as Weight))
            .saturating_add(T::DbWeight::get().writes(10 as Weight))
    }
    fn create_taproot_withdraw_tx() -> Weight {
        (147_105_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(14 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn set_best_index() -> Weight {
        (3_180_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn set_confirmed_index() -> Weight {
        (3_334_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn remove_pending() -> Weight {
        (376_795_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(9 as Weight))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
    }
    fn remove_proposal() -> Weight {
        (60_645_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn set_btc_withdrawal_fee() -> Weight {
        (2_483_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn set_btc_deposit_limit() -> Weight {
        (2_575_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn set_coming_bot() -> Weight {
        (2_887_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn push_header() -> Weight {
        (116_466_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(10 as Weight))
            .saturating_add(RocksDbWeight::get().writes(5 as Weight))
    }
    fn push_transaction() -> Weight {
        (313_612_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(23 as Weight))
            .saturating_add(RocksDbWeight::get().writes(10 as Weight))
    }
    fn create_taproot_withdraw_tx() -> Weight {
        (147_105_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(14 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    fn set_best_index() -> Weight {
        (3_180_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn set_confirmed_index() -> Weight {
        (3_334_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn remove_pending() -> Weight {
        (376_795_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(9 as Weight))
            .saturating_add(RocksDbWeight::get().writes(6 as Weight))
    }
    fn remove_proposal() -> Weight {
        (60_645_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(5 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    fn set_btc_withdrawal_fee() -> Weight {
        (2_483_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn set_btc_deposit_limit() -> Weight {
        (2_575_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn set_coming_bot() -> Weight {
        (2_887_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
}

/*
这段代码是 Substrate 框架中 `xpallet_gateway_bitcoin` 模块的权重信息定义.
权重信息是 Substrate 系统中用于衡量操作复杂性和资源消耗的一个关键概念.
每个可能的操作(例如函数调用或交易)都有一个与之关联的权重,
这有助于在执行操作时计算所需的总权重,并确保系统的稳定性和公平性.

这里的权重信息是为了衡量 `xpallet_gateway_bitcoin` 模块中的各种操作所需的资源.
这些操作包括推送比特币区块头,推送交易,创建 Taproot 地址的提款交易,设置最佳区块索引,
设置已确认索引,移除挂起的交易,移除提款提案,设置比特币提款费用,设置比特币存款限额和设置即将到来的机器人账户.

`WeightInfo` trait 定义了一系列的函数,每个函数返回一个 `Weight` 类型,表示相应操作的权重.
`SubstrateWeight` 结构体实现了这个 trait,为每个操作提供了具体的权重值.
这些值是在特定的硬件和配置下通过基准测试得到的,以确保它们反映了实际操作的成本.

此外,为了向后兼容性和测试,还有一个为单元结构体 `()` 实现的 `WeightInfo` trait.
这个实现使用了 `RocksDbWeight`,它是 Substrate 默认的数据库权重配置,
而不是特定于某个 Substrate 节点配置的 `DbWeight`.

这些权重值是通过 Substrate 的基准测试 CLI 工具生成的,该工具可以模拟执行各种操作并测量它们的资源消耗.
这些权重值对于调整交易费用和确保网络的运行效率至关重要.
*/
