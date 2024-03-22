// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use std::collections::BTreeMap;
use std::fmt::Display;
use std::str::FromStr;
use std::sync::Arc;

use codec::Codec;
use jsonrpc_derive::rpc;

use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{
    generic::BlockId,
    traits::{Block as BlockT, Zero},
};

use xp_rpc::{runtime_error_into_rpc_err, Result, RpcBalance};

use xpallet_assets_rpc_runtime_api::{
    AssetId, AssetType, TotalAssetInfo, XAssetsApi as XAssetsRuntimeApi,
};

pub struct Assets<C, B> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<B>,
}

impl<C, B> Assets<C, B> {
    /// Create new `Contracts` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

#[rpc]
pub trait XAssetsApi<BlockHash, AccountId, Balance>
where
    Balance: Display + FromStr,
{
    /// Return all assets with AssetTypes for an account (exclude native token(PCX)). The returned map would not contains the assets which is not existed for this account but existed in valid assets list.
    #[rpc(name = "xassets_getAssetsByAccount")]
    fn assets_by_account(
        &self,
        who: AccountId,
        at: Option<BlockHash>,
    ) -> Result<BTreeMap<AssetId, BTreeMap<AssetType, RpcBalance<Balance>>>>;

    /// Return all valid assets balance with AssetTypes. (exclude native token(PCX))
    #[rpc(name = "xassets_getAssets")]
    fn assets(
        &self,
        at: Option<BlockHash>,
    ) -> Result<BTreeMap<AssetId, TotalAssetInfo<RpcBalance<Balance>>>>;
}

impl<C, Block, AccountId, Balance> XAssetsApi<<Block as BlockT>::Hash, AccountId, Balance>
    for Assets<C, Block>
where
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: XAssetsRuntimeApi<Block, AccountId, Balance>,
    Block: BlockT,
    AccountId: Clone + Display + Codec,
    Balance: Clone + Copy + Display + FromStr + Codec + Zero,
{
    fn assets_by_account(
        &self,
        who: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<BTreeMap<AssetId, BTreeMap<AssetType, RpcBalance<Balance>>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        api.assets_for_account(&at, who)
            .map(|map| {
                map.into_iter()
                    .map(|(id, m)| {
                        let balance = AssetType::iter()
                            .cloned()
                            .map(|ty| {
                                (ty, m.get(&ty).copied().unwrap_or_else(Balance::zero).into())
                            })
                            .collect::<BTreeMap<_, _>>();
                        (id, balance)
                    })
                    .collect::<BTreeMap<_, _>>()
            })
            .map_err(runtime_error_into_rpc_err)
    }

    fn assets(
        &self,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<BTreeMap<AssetId, TotalAssetInfo<RpcBalance<Balance>>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        api.assets(&at)
            .map(|map| {
                map.into_iter()
                    .map(|(id, info)| {
                        let balance = AssetType::iter()
                            .map(|ty| {
                                (
                                    *ty,
                                    info.balance
                                        .get(ty)
                                        .copied()
                                        .unwrap_or_else(Balance::zero)
                                        .into(),
                                )
                            })
                            .collect::<BTreeMap<_, _>>();
                        (
                            id,
                            TotalAssetInfo::<RpcBalance<Balance>> {
                                info: info.info,
                                balance,
                                is_online: info.is_online,
                                restrictions: info.restrictions,
                            },
                        )
                    })
                    .collect()
            })
            .map_err(runtime_error_into_rpc_err)
    }
}

/*
这段代码是ChainX项目的一部分,它实现了`XAssetsApi` trait,这是一个JSON-RPC API接口,用于查询区块链上的资产信息.
这个API允许外部调用者获取特定账户的资产余额信息,以及整个网络中所有有效资产的总信息.以下是对代码的详细解释:

1. **导入模块**:代码开始部分导入了所需的标准库模块,`codec`库用于序列化和反序列化,
`jsonrpc_derive`库用于自动生成RPC方法的实现,以及其他必要的Substrate和ChainX项目模块.

2. **`Assets` 结构体**:定义了一个名为`Assets`的结构体,它持有对客户端的引用(`Arc<C>`),
并使用`PhantomData`来指定它所操作的区块链块类型`B`.这个结构体是`XAssetsApi` trait实现的基础.

3. **`XAssetsApi` trait**:定义了一个RPC API trait,包含两个方法:
   - `assets_by_account`:获取指定账户的所有资产(不包括原生代币PCX)及其类型和余额的映射.
   - `assets`:获取所有有效资产的总信息,包括每种资产的总余额和其他相关信息.

4. **`impl` 块**:为`Assets`结构体实现了`XAssetsApi` trait.这个实现使用了客户端的运行时API来查询账户的资产和所有资产的总信息,
并将结果转换为RPC API所需的格式.这里使用了`runtime_error_into_rpc_err`函数来将运行时错误转换为RPC错误.

5. **类型约束**:在实现`XAssetsApi`时,对泛型参数`C`,`Block`,`AccountId`和`Balance`施加了类型约束,确保它们具有所需的特征和实现.
特别是,`C`需要能够提供运行时API,`Block`需要实现`BlockT` trait,而`AccountId`和`Balance`需要实现`Clone`,`Display`,`Codec`等特征.

整体来看,这段代码为ChainX区块链上的资产查询提供了一个RPC接口,使得外部调用者可以通过JSON-RPC与区块链交互,获取有关账户资产和网络资产的详细信息.
*/
