// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

//! RPC interface for the transaction fee module.

use std::fmt::Debug;
use std::sync::Arc;

use codec::{Codec, Decode};
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;

use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::Bytes;
use sp_runtime::{
    generic::BlockId,
    traits::{Block as BlockT, MaybeDisplay, MaybeFromStr},
};

use pallet_transaction_payment_rpc::Error;

use xp_rpc::RpcBalance;
use xpallet_transaction_fee_rpc_runtime_api::{FeeDetails, InclusionFee};

pub use xpallet_transaction_fee_rpc_runtime_api::XTransactionFeeApi as XTransactionFeeRuntimeApi;

#[rpc]
pub trait XTransactionFeeApi<BlockHash, ResponseType> {
    #[rpc(name = "xfee_queryDetails")]
    fn query_fee_details(&self, encoded_xt: Bytes, at: Option<BlockHash>) -> Result<ResponseType>;
}

/// A struct that implements the [`TransactionFeeApi`].
pub struct XTransactionFee<C, P> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<P>,
}

impl<C, P> XTransactionFee<C, P> {
    /// Create new `TransactionPayment` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, Balance> XTransactionFeeApi<<Block as BlockT>::Hash, FeeDetails<RpcBalance<Balance>>>
    for XTransactionFee<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: XTransactionFeeRuntimeApi<Block, Balance>,
    Balance: Codec + MaybeDisplay + MaybeFromStr,
{
    fn query_fee_details(
        &self,
        encoded_xt: Bytes,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<FeeDetails<RpcBalance<Balance>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let encoded_len = encoded_xt.len() as u32;

        let uxt: Block::Extrinsic = Decode::decode(&mut &*encoded_xt).map_err(into_rpc_err)?;

        api.query_fee_details(&at, uxt, encoded_len)
            .map(|fee_details| FeeDetails {
                inclusion_fee: fee_details.inclusion_fee.map(|fee| InclusionFee {
                    base_fee: fee.base_fee.into(),
                    len_fee: fee.len_fee.into(),
                    adjusted_weight_fee: fee.adjusted_weight_fee.into(),
                }),
                tip: fee_details.tip.into(),
                extra_fee: fee_details.extra_fee.into(),
                final_fee: fee_details.final_fee.into(),
            })
            .map_err(into_rpc_err)
    }
}

fn into_rpc_err(err: impl Debug) -> RpcError {
    RpcError {
        code: ErrorCode::ServerError(Error::RuntimeError.into()),
        message: "Unable to query dispatch info.".into(),
        data: Some(format!("{:?}", err).into()),
    }
}

/*
这段代码定义了ChainX区块链项目中的交易费用模块的RPC(远程过程调用)接口.
RPC接口允许外部客户端与区块链节点进行交互,查询交易费用的详细信息.以下是对代码中的关键组件和功能的详细解释:

1. **XTransactionFeeApi**:
   - 这是一个trait,定义了`query_fee_details` RPC方法.这个方法接受一个编码后的交易(`Bytes`)和区块哈希(`BlockHash`),并返回交易的`FeeDetails`.

2. **XTransactionFee**:
   - 这是一个结构体,实现了`XTransactionFeeApi` trait.它持有对客户端(`C`)的引用,并通过`PhantomData`来标记与特定区块类型(`Block`)和余额类型(`Balance`)的关联.

3. **query_fee_details**:
   - 这是`XTransactionFeeApi` trait的一个实现,它使用提供的客户端API来查询交易费用的详细信息.它首先解码传入的交易编码,
   然后调用API的`query_fee_details`方法,并将结果转换为RPC响应格式.

4. **into_rpc_err**:
   - 这是一个辅助函数,用于将任何实现了`Debug` trait的错误转换为RPC错误.它将错误信息格式化为字符串,并将其作为错误数据返回.

5. **依赖项**:
   - `std::fmt::Debug` 和 `std::sync::Arc` 用于错误处理和同步.
   - `codec` 用于序列化和反序列化.
   - `jsonrpc_core` 用于定义RPC错误和结果.
   - `sp_api` 和 `sp_blockchain` 用于与Substrate API和区块链交互.
   - `pallet_transaction_payment_rpc` 和 `xpallet_transaction_fee_rpc_runtime_api` 用于定义交易费用相关的RPC API和运行时API.

这个RPC接口为区块链的用户提供了一个查询交易费用的便捷方式,这对于前端应用程序,钱包服务和其他需要计算交易费用的客户端来说非常有用.
通过这个接口,用户可以在提交交易之前估算费用,从而做出更明智的决策.
*/
