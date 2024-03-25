// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

//! Runtime API definition required by ChainX RPC extensions.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments, clippy::unnecessary_mut_passed)]

use sp_std::{collections::btree_map::BTreeMap, prelude::*};

use codec::Codec;

pub use chainx_primitives::AssetId;
pub use xpallet_mining_asset::{
    AssetLedger, MinerLedger, MiningAssetInfo, MiningDividendInfo, MiningWeight,
};

sp_api::decl_runtime_apis! {
    /// The API to query mining asset info.
    pub trait XMiningAssetApi<AccountId, Balance, MiningWeight, BlockNumber>
    where
        AccountId: Codec,
        Balance: Codec,
        MiningWeight: Codec,
        BlockNumber: Codec,
    {
        /// Get overall information about all mining assets.
        fn mining_assets() -> Vec<MiningAssetInfo<AccountId, Balance, MiningWeight, BlockNumber>>;

        /// Get the asset mining dividends info given the asset miner AccountId.
        fn mining_dividend(who: AccountId) -> BTreeMap<AssetId, MiningDividendInfo<Balance>>;

        /// Get the mining ledger details given the asset miner AccountId.
        fn miner_ledger(who: AccountId) -> BTreeMap<AssetId, MinerLedger<MiningWeight, BlockNumber>>;
    }
}

/*
这段代码定义了ChainX区块链RPC(远程过程调用)扩展所需的运行时API(应用程序编程接口).
这个API允许外部客户端与ChainX区块链进行交互,查询有关挖矿资产的信息.以下是代码的主要组成部分和它们的功能:

1. **`#![cfg_attr(not(feature = "std"), no_std)]`**:
   - 这是一个条件编译属性,它指定当没有启用`std`特性时,应该编译为`no_std`环境.这通常适用于在WebAssembly (Wasm) 环境中运行的代码.

2. **`#![allow(clippy::too_many_arguments, clippy::unnecessary_mut_passed)]`**:
   - 这是一个编译警告抑制属性,它告诉Rust编译器忽略由`clippy`工具报告的特定警告.在这里,它抑制了因为函数参数过多或传递了不必要的可变引用而产生的警告.

3. **`use`语句**:
   - 这些语句引入了代码中所需的外部依赖项,包括`sp_std`和`codec`库中的类型和特质.

4. **`pub use`语句**:
   - 这些语句将`chainx_primitives`和`xpallet_mining_asset`模块中的类型重新导出,使得它们可以直接在当前模块的外部使用.

5. **`sp_api::decl_runtime_apis!`宏**:
   - 这个宏定义了一个运行时API接口,它包含了一组可以在ChainX区块链上查询挖矿资产信息的方法.

6. **`XMiningAssetApi` trait**:
   - 这是一个特质(trait),定义了`XMiningAssetApi`接口.它包含以下方法:
     - `mining_assets()`: 返回关于所有挖矿资产的总体信息.
     - `mining_dividend(who: AccountId)`: 根据资产矿工的账户ID,返回资产挖矿分红信息.
     - `miner_ledger(who: AccountId)`: 根据资产矿工的账户ID,返回矿工账本的详细信息.

这个API为ChainX区块链的用户提供了一个标准化的方式来查询挖矿相关的数据,这对于开发基于ChainX的去中心化应用(DApps)和工具非常重要.
通过这些API,用户可以获取到挖矿资产的当前状态,分红信息和矿工账本,从而更好地理解和参与到ChainX的挖矿生态系统中.
*/
