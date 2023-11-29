// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

//! RPC interface for the transaction fee module.

use std::sync::Arc;

use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use parity_scale_codec::{Codec, Decode};

use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::Bytes;
use sp_runtime::traits::{Block as BlockT, MaybeDisplay, MaybeFromStr};

use xp_rpc::{hex_decode_error_into_rpc_err, tx_fee_error_into_rpc_err, RpcBalance};
use xpallet_transaction_fee_rpc_runtime_api::{FeeDetails, InclusionFee};

pub use xpallet_transaction_fee_rpc_runtime_api::XTransactionFeeApi as XTransactionFeeRuntimeApi;

#[rpc(client, server)]
pub trait XTransactionFeeApi<BlockHash, ResponseType> {
	#[method(name = "xfee_queryDetails")]
	fn query_fee_details(
		&self,
		encoded_xt: Bytes,
		at: Option<BlockHash>,
	) -> RpcResult<ResponseType>;
}

/// A struct that implements the [`TransactionFeeApi`].
pub struct XTransactionFee<C, P> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<P>,
}

impl<C, P> XTransactionFee<C, P> {
	/// Create new `TransactionPayment` with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

impl<C, Block, Balance>
	XTransactionFeeApiServer<<Block as BlockT>::Hash, FeeDetails<RpcBalance<Balance>>>
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
	) -> RpcResult<FeeDetails<RpcBalance<Balance>>> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);

		let encoded_len = encoded_xt.len() as u32;

		let uxt: Block::Extrinsic =
			Decode::decode(&mut &*encoded_xt).map_err(hex_decode_error_into_rpc_err)?;

		api.query_fee_details(at, uxt, encoded_len)
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
			.map_err(tx_fee_error_into_rpc_err)
	}
}
