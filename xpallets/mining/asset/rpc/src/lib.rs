// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

//! RPC interface for the transaction payment module.
#![allow(clippy::type_complexity)]
use std::collections::btree_map::BTreeMap;
use std::fmt::Display;
use std::str::FromStr;
use std::sync::Arc;

use codec::Codec;
use jsonrpc_derive::rpc;

use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

use xp_rpc::{runtime_error_into_rpc_err, Result, RpcBalance, RpcMiningWeight};

use xpallet_mining_asset_rpc_runtime_api::{
    AssetId, AssetLedger, MinerLedger, MiningAssetInfo, MiningDividendInfo,
    XMiningAssetApi as XMiningAssetRuntimeApi,
};

/// XMiningAsset RPC methods.
#[rpc]
pub trait XMiningAssetApi<BlockHash, AccountId, Balance, MiningWeight, BlockNumber>
where
    Balance: Display + FromStr,
    MiningWeight: Display + FromStr,
{
    /// Get overall information about all mining assets.
    #[rpc(name = "xminingasset_getMiningAssets")]
    fn mining_assets(
        &self,
        at: Option<BlockHash>,
    ) -> Result<
        Vec<
            MiningAssetInfo<
                AccountId,
                RpcBalance<Balance>,
                RpcMiningWeight<MiningWeight>,
                BlockNumber,
            >,
        >,
    >;

    /// Get the asset mining dividends info given the asset miner AccountId.
    #[rpc(name = "xminingasset_getDividendByAccount")]
    fn mining_dividend(
        &self,
        who: AccountId,
        at: Option<BlockHash>,
    ) -> Result<BTreeMap<AssetId, MiningDividendInfo<RpcBalance<Balance>>>>;

    /// Get the mining ledger details given the asset miner AccountId.
    #[rpc(name = "xminingasset_getMinerLedgerByAccount")]
    fn miner_ledger(
        &self,
        who: AccountId,
        at: Option<BlockHash>,
    ) -> Result<BTreeMap<AssetId, MinerLedger<RpcMiningWeight<MiningWeight>, BlockNumber>>>;
}

/// A struct that implements the [`XMiningAssetApi`].
pub struct XMiningAsset<C, B> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<B>,
}

impl<C, B> XMiningAsset<C, B> {
    /// Create new `XMiningAsset` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, AccountId, Balance, MiningWeight, BlockNumber>
    XMiningAssetApi<<Block as BlockT>::Hash, AccountId, Balance, MiningWeight, BlockNumber>
    for XMiningAsset<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: XMiningAssetRuntimeApi<Block, AccountId, Balance, MiningWeight, BlockNumber>,
    AccountId: Codec,
    Balance: Codec + Display + FromStr,
    MiningWeight: Codec + Display + FromStr,
    BlockNumber: Codec,
{
    fn mining_assets(
        &self,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<
        Vec<
            MiningAssetInfo<
                AccountId,
                RpcBalance<Balance>,
                RpcMiningWeight<MiningWeight>,
                BlockNumber,
            >,
        >,
    > {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        api.mining_assets(&at)
            .map(|mining_assets| {
                mining_assets
                    .into_iter()
                    .map(|mining_asset| MiningAssetInfo {
                        asset_id: mining_asset.asset_id,
                        mining_power: mining_asset.mining_power,
                        reward_pot: mining_asset.reward_pot,
                        reward_pot_balance: mining_asset.reward_pot_balance.into(),
                        ledger_info: AssetLedger {
                            last_total_mining_weight: mining_asset
                                .ledger_info
                                .last_total_mining_weight
                                .into(),
                            last_total_mining_weight_update: mining_asset
                                .ledger_info
                                .last_total_mining_weight_update,
                        },
                    })
                    .collect::<Vec<_>>()
            })
            .map_err(runtime_error_into_rpc_err)
    }

    fn mining_dividend(
        &self,
        who: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<BTreeMap<AssetId, MiningDividendInfo<RpcBalance<Balance>>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        api.mining_dividend(&at, who)
            .map(|mining_dividend| {
                mining_dividend
                    .into_iter()
                    .map(|(id, info)| {
                        (
                            id,
                            MiningDividendInfo {
                                own: info.own.into(),
                                other: info.other.into(),
                                insufficient_stake: info.insufficient_stake.into(),
                            },
                        )
                    })
                    .collect()
            })
            .map_err(runtime_error_into_rpc_err)
    }

    fn miner_ledger(
        &self,
        who: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<BTreeMap<AssetId, MinerLedger<RpcMiningWeight<MiningWeight>, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        api.miner_ledger(&at, who)
            .map(|miner_ledger| {
                miner_ledger
                    .into_iter()
                    .map(|(id, miner_ledger)| {
                        (
                            id,
                            MinerLedger {
                                last_mining_weight: miner_ledger.last_mining_weight.into(),
                                last_mining_weight_update: miner_ledger.last_mining_weight_update,
                                last_claim: miner_ledger.last_claim,
                            },
                        )
                    })
                    .collect()
            })
            .map_err(runtime_error_into_rpc_err)
    }
}

/*
这段代码定义了ChainX区块链的交易支付模块的RPC(远程过程调用)接口.RPC接口允许外部客户端通过JSON-RPC与区块链节点进行交互,
查询和执行操作.这个特定的接口专注于提供有关挖矿资产的信息,包括资产的挖矿权重,分红信息和矿工账本.

以下是代码的主要组成部分和它们的功能:

1. **`#![allow(clippy::type_complexity)]`**:
   - 这是一个编译警告抑制属性,用于忽略`clippy`工具报告的复杂类型警告.

2. **依赖项**:
   - 引入了必要的外部依赖项,包括`std`库中的类型,`codec`库用于序列化和反序列化,`jsonrpc_derive`库用于自动生成RPC方法,
   以及其他Substrate和ChainX项目中的类型和特质.

3. **`XMiningAssetApi` trait**:
   - 定义了挖矿资产相关的RPC方法.这些方法允许客户端查询:
     - `mining_assets`: 获取所有挖矿资产的总体信息.
     - `mining_dividend`: 获取特定账户的资产挖矿分红信息.
     - `miner_ledger`: 获取特定账户的矿工账本详细信息.

4. **`XMiningAsset` struct**:
   - 实现了`XMiningAssetApi` trait,提供了与ChainX区块链节点进行交互的实际RPC方法.它持有对客户端的引用,
   并通过`PhantomData`来指定它服务的区块链块类型.

5. **实现`XMiningAssetApi` trait`**:
   - 为`XMiningAsset` struct提供了实际的RPC方法实现.这些方法使用客户端的运行时API来获取所需的信息,并将其转换为RPC结果类型.
   - 使用了`map_err`来将可能发生的运行时错误转换为RPC错误.

整体来看,这段代码为ChainX区块链提供了一个RPC接口,使得外部客户端可以查询挖矿资产的相关信息.这对于开发者来说是一个强大的工具,
因为它允许他们构建应用程序和服务,这些应用程序和服务可以基于区块链上的挖矿资产数据进行决策和交互.
*/
