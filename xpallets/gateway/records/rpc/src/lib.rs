// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use std::collections::BTreeMap;
use std::fmt::Display;
use std::str::FromStr;
use std::sync::Arc;

use codec::Codec;
use jsonrpc_derive::rpc;
use serde::{Deserialize, Serialize};

use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

use xp_rpc::{runtime_error_into_rpc_err, Result};

use xpallet_gateway_records_rpc_runtime_api::{
    AssetId, Chain, Withdrawal, WithdrawalRecordId, WithdrawalState,
    XGatewayRecordsApi as GatewayRecordsRuntimeApi,
};

pub struct XGatewayRecords<C, B> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<B>,
}

impl<C, B> XGatewayRecords<C, B> {
    /// Create new `Contracts` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

#[rpc]
pub trait XGatewayRecordsApi<BlockHash, AccountId, Balance, BlockNumber>
where
    Balance: Display + FromStr,
{
    /// Return current withdraw list(include Applying and Processing withdraw state)
    #[rpc(name = "xgatewayrecords_withdrawalList")]
    fn withdrawal_list(
        &self,
        at: Option<BlockHash>,
    ) -> Result<BTreeMap<WithdrawalRecordId, RpcWithdrawalRecord<AccountId, Balance, BlockNumber>>>;

    /// Return current withdraw list for a chain(include Applying and Processing withdraw state)
    #[rpc(name = "xgatewayrecords_withdrawalListByChain")]
    fn withdrawal_list_by_chain(
        &self,
        chain: Chain,
        at: Option<BlockHash>,
    ) -> Result<BTreeMap<WithdrawalRecordId, RpcWithdrawalRecord<AccountId, Balance, BlockNumber>>>;

    /// Return current pending withdraw list for a chain
    #[rpc(name = "xgatewayrecords_pendingWithdrawalListByChain")]
    fn pending_withdrawal_list_by_chain(
        &self,
        chain: Chain,
        at: Option<BlockHash>,
    ) -> Result<BTreeMap<WithdrawalRecordId, RpcWithdrawalRecord<AccountId, Balance, BlockNumber>>>;
}

impl<C, Block, AccountId, Balance, BlockNumber>
    XGatewayRecordsApi<<Block as BlockT>::Hash, AccountId, Balance, BlockNumber>
    for XGatewayRecords<C, Block>
where
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: GatewayRecordsRuntimeApi<Block, AccountId, Balance, BlockNumber>,
    Block: BlockT,
    AccountId: Clone + Display + FromStr + Codec,
    Balance: Clone + Display + FromStr + Codec,
    BlockNumber: Clone + Display + Codec,
{
    fn withdrawal_list(
        &self,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<BTreeMap<u32, RpcWithdrawalRecord<AccountId, Balance, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        api.withdrawal_list(&at)
            .map(|map| {
                map.into_iter()
                    .map(|(id, withdrawal)| (id, withdrawal.into()))
                    .collect()
            })
            .map_err(runtime_error_into_rpc_err)
    }

    fn withdrawal_list_by_chain(
        &self,
        chain: Chain,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<BTreeMap<u32, RpcWithdrawalRecord<AccountId, Balance, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        api.withdrawal_list_by_chain(&at, chain)
            .map(|map| {
                map.into_iter()
                    .map(|(id, withdrawal)| (id, withdrawal.into()))
                    .collect()
            })
            .map_err(runtime_error_into_rpc_err)
    }

    fn pending_withdrawal_list_by_chain(
        &self,
        chain: Chain,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<BTreeMap<u32, RpcWithdrawalRecord<AccountId, Balance, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        api.withdrawal_list_by_chain(&at, chain)
            .map(|map| {
                map.into_iter()
                    .filter_map(|(id, withdrawal)| {
                        if withdrawal.state == WithdrawalState::Applying {
                            Some((id, withdrawal.into()))
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .map_err(runtime_error_into_rpc_err)
    }
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcWithdrawalRecord<AccountId, Balance: Display + FromStr, BlockNumber> {
    pub asset_id: AssetId,
    pub applicant: AccountId,
    #[serde(with = "xp_rpc::serde_num_str")]
    pub balance: Balance,
    pub addr: String,
    pub ext: String,
    pub height: BlockNumber,
    pub state: WithdrawalState,
}

impl<AccountId, Balance: Display + FromStr, BlockNumber>
    From<Withdrawal<AccountId, Balance, BlockNumber>>
    for RpcWithdrawalRecord<AccountId, Balance, BlockNumber>
{
    fn from(record: Withdrawal<AccountId, Balance, BlockNumber>) -> Self {
        Self {
            asset_id: record.asset_id,
            applicant: record.applicant,
            balance: record.balance,
            addr: String::from_utf8_lossy(record.addr.as_ref()).into_owned(),
            ext: String::from_utf8_lossy(record.ext.as_ref()).into_owned(),
            height: record.height,
            state: record.state,
        }
    }
}

/*
这段代码定义了 `XGatewayRecordsApi`,这是一个 JSON-RPC 接口,用于与 ChainX 项目的跨链交易记录模块进行交互.
它允许用户查询取款列表,特定链的取款列表以及特定链的挂起取款列表.此外,还提供了 `RpcWithdrawalRecord` 结构体,用于将取款记录序列化为 JSON-RPC 响应.

### 主要组件

1. **XGatewayRecords 结构体**:封装了对客户端的引用,用于与区块链交互.

2. **XGatewayRecordsApi trait**:定义了三个 RPC 方法,用于获取取款列表和挂起取款列表.

3. **RpcWithdrawalRecord 结构体**:表示取款记录的 RPC 版本,包含了取款的详细信息.

### RPC 方法

- **withdrawal_list**:返回当前所有取款记录的映射,包括正在处理和应用中的取款.

- **withdrawal_list_by_chain**:返回特定链上的所有取款记录的映射.

- **pending_withdrawal_list_by_chain**:返回特定链上所有挂起的取款记录的映射.

### 实现细节

- **impl XGatewayRecordsApi**:为 `XGatewayRecords` 结构体实现了 `XGatewayRecordsApi` trait,使其能够处理 RPC 请求.

- **From<Withdrawal<...>> for RpcWithdrawalRecord<...>**:实现了从 `Withdrawal` 类型到 `RpcWithdrawalRecord` 
类型的转换,以便在序列化前将取款记录转换为 RPC 响应格式.

### 序列化和反序列化

- **serde**:使用 Serde 库来序列化和反序列化 `RpcWithdrawalRecord` 结构体.

- **serde_num_str**:自定义序列化策略,用于将数字类型转换为字符串.

### 总结

`XGatewayRecordsApi` 提供了一个接口,使得外部客户端可以查询 ChainX 区块链上的取款记录,这对于监控跨链资产流动和管理取款过程非常重要.
通过这些 RPC 方法,用户和开发者可以获取有关取款状态的详细信息,从而更好地理解和使用 ChainX 项目的功能.
*/
