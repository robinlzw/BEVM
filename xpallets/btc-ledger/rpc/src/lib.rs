// Copyright 2023 BEVM Project Authors. Licensed under GPL-3.0.

use std::{fmt::Display, str::FromStr, sync::Arc};

use parity_scale_codec::Codec;

use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::{Block as BlockT, Zero};

use jsonrpsee::{core::RpcResult, proc_macros::rpc};

use xp_rpc::{runtime_error_into_rpc_err, RpcBalance};

use xpallet_btc_ledger_runtime_api::BtcLedgerApi as BtcLedgerRuntimeApi;

pub struct BtcLedger<C, B> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<B>,
}

impl<C, B> BtcLedger<C, B> {
	/// Create new `Contracts` with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

#[rpc(client, server)]
pub trait BtcLedgerApi<BlockHash, AccountId, Balance>
where
	Balance: Display + FromStr,
{
	/// Return balance for an account
	#[method(name = "btcledger_getBalance")]
	fn btcledger_balance(
		&self,
		who: AccountId,
		at: Option<BlockHash>,
	) -> RpcResult<RpcBalance<Balance>>;

	/// Return total incoming balance of BTC
	#[method(name = "btcledger_getTotalInComing")]
	fn btcledger_total(&self, at: Option<BlockHash>) -> RpcResult<RpcBalance<Balance>>;
}

impl<C, Block, AccountId, Balance> BtcLedgerApiServer<<Block as BlockT>::Hash, AccountId, Balance>
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
	) -> RpcResult<RpcBalance<Balance>> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);
		api.get_balance(at, who).map(|b| b.into()).map_err(runtime_error_into_rpc_err)
	}

	fn btcledger_total(
		&self,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<RpcBalance<Balance>> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);
		api.get_total(at).map(|b| b.into()).map_err(runtime_error_into_rpc_err)
	}
}
