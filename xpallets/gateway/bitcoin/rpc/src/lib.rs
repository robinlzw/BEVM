// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

//! RPC interface for the transaction verification.
use codec::Codec;
use jsonrpc_derive::rpc;
use std::sync::Arc;
use std::vec::Vec;

use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

use xp_rpc::{runtime_error_into_rpc_err, Result};
use xpallet_gateway_bitcoin_rpc_runtime_api::{
    BtcHeader, BtcHeaderInfo, BtcWithdrawalProposal,
    XGatewayBitcoinApi as XGatewayBitcoinRuntimeApi, H256,
};

pub struct XGatewayBitcoin<C, B, AccountId> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<(B, AccountId)>,
}

impl<C, B, AccountId> XGatewayBitcoin<C, B, AccountId> {
    /// Create new `XGatewayBitcoin` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

#[rpc]
pub trait XGatewayBitcoinApi<BlockHash, AccountId> {
    /// Verify transaction is valid
    #[rpc(name = "xgatewaybitcoin_verifyTxValid")]
    fn verify_tx_valid(
        &self,
        raw_tx: String,
        withdrawal_id_list: Vec<u32>,
        full_amount: bool,
        at: Option<BlockHash>,
    ) -> Result<bool>;

    /// Get withdrawal proposal
    #[rpc(name = "xgatewaybitcoin_getWithdrawalProposal")]
    fn get_withdrawal_proposal(
        &self,
        at: Option<BlockHash>,
    ) -> Result<Option<BtcWithdrawalProposal<AccountId>>>;

    /// Get genesis info
    #[rpc(name = "xgatewaybitcoin_getGenesisInfo")]
    fn get_genesis_info(&self, at: Option<BlockHash>) -> Result<(BtcHeader, u32)>;

    /// Get block header
    #[rpc(name = "xgatewaybitcoin_getBtcBlockHeader")]
    fn get_btc_block_header(
        &self,
        txid: H256,
        at: Option<BlockHash>,
    ) -> Result<Option<BtcHeaderInfo>>;
}

impl<C, Block, AccountId> XGatewayBitcoinApi<<Block as BlockT>::Hash, AccountId>
    for XGatewayBitcoin<C, Block, AccountId>
where
    Block: BlockT,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: XGatewayBitcoinRuntimeApi<Block, AccountId>,
    AccountId: Codec + Send + Sync + 'static,
{
    fn verify_tx_valid(
        &self,
        raw_tx: String,
        withdrawal_id_list: Vec<u32>,
        full_amount: bool,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<bool> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        let raw_tx = hex::decode(raw_tx).map_err(runtime_error_into_rpc_err)?;
        let result = api
            .verify_tx_valid(&at, raw_tx, withdrawal_id_list, full_amount)
            .map_err(runtime_error_into_rpc_err)?
            .map_err(runtime_error_into_rpc_err)?;
        Ok(result)
    }

    fn get_withdrawal_proposal(
        &self,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Option<BtcWithdrawalProposal<AccountId>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        let result = api
            .get_withdrawal_proposal(&at)
            .map_err(runtime_error_into_rpc_err)?;
        Ok(result)
    }

    fn get_genesis_info(&self, at: Option<<Block as BlockT>::Hash>) -> Result<(BtcHeader, u32)> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        let result = api
            .get_genesis_info(&at)
            .map_err(runtime_error_into_rpc_err)?;
        Ok(result)
    }

    fn get_btc_block_header(
        &self,
        txid: H256,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Option<BtcHeaderInfo>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        let reslut = api
            .get_btc_block_header(&at, txid)
            .map_err(runtime_error_into_rpc_err)?;
        Ok(reslut)
    }
}

/*
这段代码定义了一个 RPC 接口 `XGatewayBitcoinApi`,它是为 ChainX 区块链中的比特币网关模块提供的,
用于交易验证和其他与比特币相关的查询.这个接口允许外部客户端通过 JSON-RPC 与 ChainX 区块链进行交互,
执行诸如验证比特币交易,获取提款提案,获取创世信息和获取比特币区块头等操作.

### RPC 接口实现

- `verify_tx_valid`: 验证比特币交易是否有效.它接受一个十六进制编码的原始交易字符串 `raw_tx`,
一个提款 ID 列表 `withdrawal_id_list`,一个布尔值 `full_amount`(指示是否验证全部金额),以及一个可选的区块哈希 `at`.如果交易有效,返回 `Ok(true)`;否则返回错误.

- `get_withdrawal_proposal`: 获取当前的比特币提款提案.它接受一个可选的区块哈希 `at` 作为参数,并返回一个提款提案的选项 `Option<BtcWithdrawalProposal<AccountId>>`.

- `get_genesis_info`: 获取比特币链的创世信息.它接受一个可选的区块哈希 `at` 作为参数,并返回一个包含创世区块头 `BtcHeader` 和区块版本号 `u32` 的元组.

- `get_btc_block_header`: 根据给定的交易 ID `txid` 获取比特币区块头信息.它接受一个可选的区块哈希 `at` 作为参数,并返回一个区块头信息的选项 `Option<BtcHeaderInfo>`.

### 结构体 `XGatewayBitcoin`

- `XGatewayBitcoin`: 这个结构体封装了对客户端的引用 `client`,并包含了一个类型标记 `_marker`.它提供了一个 `new` 方法来创建一个新的实例.

### 实现细节

- `runtime_api`: 通过调用 `self.client.runtime_api()` 获取运行时 API,这是与 ChainX 区块链状态进行交互的接口.

- `BlockId`: 用于将可选的区块哈希 `at` 转换为 `BlockId`,这是 `sp_runtime` 中用于标识区块的类型.

- `runtime_error_into_rpc_err`: 这是一个辅助函数,用于将运行时错误转换为 RPC 错误,以便它们可以通过 JSON-RPC 响应返回给调用者.

- `Codec` 和 `Send + Sync + 'static`: 这些 trait 约束确保 `AccountId` 类型可以被序列化和在线程间安全传递.

整体而言,这段代码为 ChainX 区块链的比特币网关模块提供了一个 RPC 接口,使得外部服务能够验证比特币交易,查询提款提案和获取区块头信息,从而增强了 ChainX 区块链与比特币网络的互操作性.
*/

