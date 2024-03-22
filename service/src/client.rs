// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use sp_runtime::traits::BlakeTwo256;

use chainx_primitives::{AccountId, Balance, Block, BlockNumber, Index};
use xpallet_mining_asset_rpc_runtime_api::MiningWeight;
use xpallet_mining_staking_rpc_runtime_api::VoteWeight;

/// A set of APIs that chainx-like runtimes must implement.
pub trait RuntimeApiCollection:
    sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
    + sp_api::ApiExt<Block>
    + sp_consensus_babe::BabeApi<Block>
    + sp_finality_grandpa::GrandpaApi<Block>
    + sp_block_builder::BlockBuilder<Block>
    + frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index>
    + pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
    + sp_api::Metadata<Block>
    + sp_offchain::OffchainWorkerApi<Block>
    + sp_session::SessionKeys<Block>
    + sp_authority_discovery::AuthorityDiscoveryApi<Block>
    + xpallet_assets_rpc_runtime_api::XAssetsApi<Block, AccountId, Balance>
    + xpallet_dex_spot_rpc_runtime_api::XSpotApi<Block, AccountId, Balance, BlockNumber, Balance>
    + xpallet_gateway_bitcoin_rpc_runtime_api::XGatewayBitcoinApi<Block, AccountId>
    + xpallet_gateway_common_rpc_runtime_api::XGatewayCommonApi<
        Block,
        AccountId,
        Balance,
        BlockNumber,
    > + xpallet_gateway_records_rpc_runtime_api::XGatewayRecordsApi<
        Block,
        AccountId,
        Balance,
        BlockNumber,
    > + xpallet_mining_staking_rpc_runtime_api::XStakingApi<
        Block,
        AccountId,
        Balance,
        VoteWeight,
        BlockNumber,
    > + xpallet_mining_asset_rpc_runtime_api::XMiningAssetApi<
        Block,
        AccountId,
        Balance,
        MiningWeight,
        BlockNumber,
    > + xpallet_transaction_fee_rpc_runtime_api::XTransactionFeeApi<Block, Balance>
    + xpallet_btc_ledger_runtime_api::BtcLedgerApi<Block, AccountId, Balance>
    + fp_rpc::EthereumRuntimeRPCApi<Block>
    + fp_rpc::ConvertTransactionRuntimeApi<Block>
where
    <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}

impl<Api> RuntimeApiCollection for Api
where
    Api: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
        + sp_api::ApiExt<Block>
        + sp_consensus_babe::BabeApi<Block>
        + sp_finality_grandpa::GrandpaApi<Block>
        + sp_block_builder::BlockBuilder<Block>
        + frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index>
        + pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
        + sp_api::Metadata<Block>
        + sp_offchain::OffchainWorkerApi<Block>
        + sp_session::SessionKeys<Block>
        + sp_authority_discovery::AuthorityDiscoveryApi<Block>
        + xpallet_assets_rpc_runtime_api::XAssetsApi<Block, AccountId, Balance>
        + xpallet_dex_spot_rpc_runtime_api::XSpotApi<Block, AccountId, Balance, BlockNumber, Balance>
        + xpallet_gateway_bitcoin_rpc_runtime_api::XGatewayBitcoinApi<Block, AccountId>
        + xpallet_gateway_common_rpc_runtime_api::XGatewayCommonApi<
            Block,
            AccountId,
            Balance,
            BlockNumber,
        > + xpallet_gateway_records_rpc_runtime_api::XGatewayRecordsApi<
            Block,
            AccountId,
            Balance,
            BlockNumber,
        > + xpallet_mining_staking_rpc_runtime_api::XStakingApi<
            Block,
            AccountId,
            Balance,
            VoteWeight,
            BlockNumber,
        > + xpallet_mining_asset_rpc_runtime_api::XMiningAssetApi<
            Block,
            AccountId,
            Balance,
            MiningWeight,
            BlockNumber,
        > + xpallet_transaction_fee_rpc_runtime_api::XTransactionFeeApi<Block, Balance>
        + xpallet_btc_ledger_runtime_api::BtcLedgerApi<Block, AccountId, Balance>
        + fp_rpc::EthereumRuntimeRPCApi<Block>
        + fp_rpc::ConvertTransactionRuntimeApi<Block>,
    <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}

/*
这段代码定义了一个名为 `RuntimeApiCollection` 的trait(特质),它规定了类似ChainX的运行时必须实现的一系列API.
这些API提供了与Substrate区块链运行时交互的不同功能,包括交易处理,状态查询,共识机制,账户信息等.
代码还提供了这些API的默认实现,任何实现了这些trait的类型都将具备这些功能.

以下是代码中定义的主要组件和它们的功能:

1. **`RuntimeApiCollection` trait**:这是一个组合trait,它要求实现多个不同的API,这些API是与Substrate运行时交互所必需的.

2. **`sp_transaction_pool`**:提供与交易池相关的API,如验证和处理交易.

3. **`sp_api`**:提供通用的API,用于查询区块链的状态和历史信息.

4. **`sp_consensus_babe` 和 `sp_finality_grandpa`**:提供与Babe和Grandpa共识机制相关的API,用于处理区块的产生和最终确定.

5. **`sp_block_builder`**:提供构建新区块的API.

6. **`frame_system_rpc_runtime_api`**:提供与系统模块相关的RPC API,如获取账户的nonce(操作计数).

7. **`pallet_transaction_payment_rpc_runtime_api`**:提供与交易费用相关的API.

8. **`sp_api::Metadata`**:提供关于运行时版本和元数据的API.

9. **`sp_offchain`**:提供与离线操作相关的API.

10. **`sp_session`**:提供与会话密钥相关的API.

11. **`sp_authority_discovery`**:提供与发现权威节点相关的API.

12. **`xpallet_assets`**,**`xpallet_dex_spot`**,**`xpallet_gateway_bitcoin`**,**`xpallet_gateway_common`**,
**`xpallet_gateway_records`**,**`xpallet_mining_staking`**,**`xpallet_mining_asset`**:
这些是ChainX项目特有的pallets,提供了与资产,去中心化交易所(DEX),比特币网关,记录和挖矿相关的API.

13. **`fp_rpc`**:提供与以太坊兼容性相关的RPC API,包括交易转换和以太坊运行时API.

14. **`impl` 块**:为 `RuntimeApiCollection` trait 提供了默认实现,任何实现了所有必需trait的类型都可以被视为实现了 `RuntimeApiCollection`.

这个trait的设计允许ChainX运行时与各种外部工具和服务进行交互,例如区块浏览器,钱包和其他RPC客户端.
这些API是构建基于Substrate的区块链项目时的关键组件,它们使得与链上的智能合约和交易进行交互成为可能.

这段Rust代码定义了一个RuntimeApiCollection trait,它规定了一组API,chainx-like runtimes(链下运行时环境)必须实现这些API.
该trait继承并组合了多个其他trait,以确保运行时环境具备处理交易池,区块链API,共识(Babe),最终性(Grandpa),
区块构建,账户nonce,交易支付,元数据,Offchain Worker,会话密钥,权威发现,资产,DEX spot交易,Gateway比特币,Gateway通用,
Gateway记录,Staking,Mining资产,Transaction Fee,BTC账本,Ethereum RPC等功能的能力.每个继承的trait都代表着运行时环境需要支持的一个特定功能或一组功能.

这个trait的实现者必须同时实现TaggedTransactionQueue,ApiExt,BabeApi,GrandpaApi,BlockBuilder,AccountNonceApi,
TransactionPaymentApi,Metadata,OffchainWorkerApi,SessionKeys,AuthorityDiscoveryApi,XAssetsApi,XSpotApi,
XGatewayBitcoinApi,XGatewayCommonApi,XGatewayRecordsApi,XStakingApi,XMiningAssetApi,XTransactionFeeApi,
BtcLedgerApi,EthereumRuntimeRPCApi和ConvertTransactionRuntimeApi中定义的方法.其中,实现者需要处理的类型参数包括区块类型(Block),
账户ID类型(AccountId),余额类型(Balance),索引类型(Index),投票权重类型(VoteWeight),挖掘权重类型(MiningWeight)以及块编号类型(BlockNumber).

这段代码通过trait bound限制了ApiExt的StateBackend实现必须是sp_api::StateBackend<BlakeTwo256>,确保了区块链状态后端的哈希算法为BlakeTwo256.

总的来说,这段代码通过定义一个包含多个功能trait的RuntimeApiCollection trait,为chainx-like runtimes规定了一个全面而丰富的API集合,以支持各种区块链功能的实现.

*/

