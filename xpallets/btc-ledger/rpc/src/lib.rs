// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

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

use xpallet_btc_ledger_runtime_api::BtcLedgerApi as BtcLedgerRuntimeApi;

pub struct BtcLedger<C, B> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<B>,
}

impl<C, B> BtcLedger<C, B> {
    /// Create new `Contracts` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

#[rpc]
pub trait BtcLedgerApi<BlockHash, AccountId, Balance>
where
    Balance: Display + FromStr,
{
    /// Return balance for an account
    #[rpc(name = "btcledger_getBalance")]
    fn btcledger_balance(
        &self,
        who: AccountId,
        at: Option<BlockHash>,
    ) -> Result<RpcBalance<Balance>>;

    /// Return total incoming balance of BTC
    #[rpc(name = "btcledger_getTotalInComing")]
    fn btcledger_total(&self, at: Option<BlockHash>) -> Result<RpcBalance<Balance>>;
}

impl<C, Block, AccountId, Balance> BtcLedgerApi<<Block as BlockT>::Hash, AccountId, Balance>
    for BtcLedger<C, Block>
where
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: BtcLedgerRuntimeApi<Block, AccountId, Balance>,
    Block: BlockT,
    AccountId: Clone + Display + Codec,
    Balance: Clone + Copy + Display + FromStr + Codec + Zero,
{
    fn btcledger_balance(
        &self,
        who: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<RpcBalance<Balance>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        api.get_balance(&at, who)
            .map(|b| b.into())
            .map_err(runtime_error_into_rpc_err)
    }

    fn btcledger_total(&self, at: Option<<Block as BlockT>::Hash>) -> Result<RpcBalance<Balance>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        api.get_total(&at)
            .map(|b| b.into())
            .map_err(runtime_error_into_rpc_err)
    }
}

/*
这段代码定义了一个名为 `BtcLedgerApi` 的 JSON-RPC 接口,用于与比特币账本进行交互,并提供了实现该接口的结构体 `BtcLedger`.
这个接口允许用户查询特定账户的比特币余额以及比特币的总流入量.以下是代码的详细解释:

### `BtcLedger` 结构体

- `BtcLedger<C, B>`: 这个结构体包含一个指向客户端的引用 `client`,以及一个类型标记 `_marker`.`client` 是客户端的引用,
用于与区块链进行交互.`_marker` 是一个幻数类型,用于确保 `B` 类型的存在,但在实现中不使用.

- `new` 方法: 用于创建 `BtcLedger` 实例,接受一个指向客户端的 `Arc` 引用.

### `BtcLedgerApi` trait

- `BtcLedgerApi<BlockHash, AccountId, Balance>`: 这个 trait 定义了两个 RPC 方法:`btcledger_balance` 和 `btcledger_total`.
这些方法允许通过 JSON-RPC 调用查询账户余额和总流入量.

- `btcledger_balance` 方法: 接受一个账户 ID 和一个可选的区块哈希,返回该账户在指定区块或最新区块的余额.

- `btcledger_total` 方法: 接受一个可选的区块哈希,返回比特币的总流入量.

### 实现 `BtcLedgerApi`

- `impl<C, Block, AccountId, Balance> BtcLedgerApi<<Block as BlockT>::Hash, AccountId, Balance> for BtcLedger<C, Block>`: 
这里为 `BtcLedger` 结构体实现了 `BtcLedgerApi` trait.实现中使用了客户端的运行时 API 来获取余额和总流入量.

- `btcledger_balance` 和 `btcledger_total` 方法的实现: 这些方法使用客户端的运行时 API 来获取所需的信息.它们处理了区块哈希的可选性,
并使用 `runtime_error_into_rpc_err` 函数将运行时错误转换为 RPC 错误.

### 总结

这段代码通过定义和实现 `BtcLedgerApi` trait,为比特币账本提供了一个与外部进行交互的接口.

*/