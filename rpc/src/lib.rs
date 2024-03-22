// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use std::sync::Arc;

use sc_client_api::AuxStore;
use sc_consensus_babe::Epoch;
use sc_consensus_babe_rpc::BabeRpcHandler;
use sc_finality_grandpa::{
    FinalityProofProvider, GrandpaJustificationStream, SharedAuthoritySet, SharedVoterState,
};
use sc_finality_grandpa_rpc::GrandpaRpcHandler;
use sc_rpc::SubscriptionTaskExecutor;
pub use sc_rpc_api::DenyUnsafe;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_consensus::SelectChain;
use sp_consensus_babe::BabeApi;

use chainx_primitives::{AccountId, Balance, Block, BlockNumber, Hash, Index};

use xpallet_mining_asset_rpc_runtime_api::MiningWeight;
use xpallet_mining_staking_rpc_runtime_api::VoteWeight;

// EVM
use fc_rpc::{
    EthBlockDataCacheTask, OverrideHandle, RuntimeApiStorageOverride, SchemaV1Override,
    SchemaV2Override, SchemaV3Override, StorageOverride,
};
use fc_rpc_core::types::{FeeHistoryCache, FilterPool};
use fp_storage::EthereumStorageSchema;
use jsonrpc_pubsub::manager::SubscriptionManager;
use sc_client_api::{
    backend::{Backend, StateBackend, StorageProvider},
    client::BlockchainEvents,
};
use sc_network::NetworkService;
use sc_transaction_pool::{ChainApi, Pool};
use sp_runtime::traits::BlakeTwo256;
use std::collections::BTreeMap;
use xp_runtime::Never;

/// Extra dependencies for BABE.
pub struct BabeDeps {
    /// BABE protocol config.
    pub babe_config: sc_consensus_babe::Config,
    /// BABE pending epoch changes.
    pub shared_epoch_changes: sc_consensus_epochs::SharedEpochChanges<Block, Epoch>,
    /// The keystore that manages the keys of the node.
    pub keystore: sp_keystore::SyncCryptoStorePtr,
}

/// Extra dependencies for GRANDPA
pub struct GrandpaDeps<B> {
    /// Voting round info.
    pub shared_voter_state: SharedVoterState,
    /// Authority set info.
    pub shared_authority_set: SharedAuthoritySet<Hash, BlockNumber>,
    /// Receives notifications about justification events from Grandpa.
    pub justification_stream: GrandpaJustificationStream<Block>,
    /// Executor to drive the subscription manager in the Grandpa RPC handler.
    pub subscription_executor: SubscriptionTaskExecutor,
    /// Finality proof provider.
    pub finality_provider: Arc<FinalityProofProvider<B, Block>>,
}

/// Frontier client dependencies
pub struct FrontierDeps<A: sc_transaction_pool::ChainApi> {
    /// Graph pool instance.
    pub graph: Arc<Pool<A>>,
    /// The Node authority flag
    pub is_authority: bool,
    /// Network service
    pub network: Arc<NetworkService<Block, Hash>>,
    /// EthFilterApi pool.
    pub filter_pool: Option<FilterPool>,
    /// Backend.
    pub backend: Arc<fc_db::Backend<Block>>,
    /// Maximum number of logs in a query.
    pub max_past_logs: u32,
    /// Maximum fee history cache size.
    pub fee_history_limit: u64,
    /// Fee history cache.
    pub fee_history_cache: FeeHistoryCache,
    /// Ethereum data access overrides.
    pub overrides: Arc<OverrideHandle<Block>>,
    /// Cache for Ethereum block data.
    pub block_data_cache: Arc<EthBlockDataCacheTask<Block>>,
}

/// Full client dependencies.
pub struct FullDeps<C, P, SC, B, A: sc_transaction_pool::ChainApi> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// The SelectionChain Strategy.
    pub select_chain: SC,
    /// A copy of the chain spec.
    pub chain_spec: Box<dyn sc_chain_spec::ChainSpec>,
    /// Whether to deny unsafe calls
    pub deny_unsafe: DenyUnsafe,
    /// BABE specific dependencies.
    pub babe: BabeDeps,
    /// GRANDPA specific dependencies.
    pub grandpa: GrandpaDeps<B>,
    /// Frontier specific dependencies.
    pub frontier: FrontierDeps<A>,
}

/*
这段代码定义了多个结构体,每个结构体都是一组依赖项,用于在区块链客户端中实现不同的功能.这些结构体包括:

BabeDeps:包含BABE协议的配置,待处理的epoch更改,节点的密钥存储等.
GrandpaDeps:包含GRANDPA投票的共享状态,授权集信息,验证事件通知等.
FrontierDeps:包含图池实例,节点权威标志,网络服务,以太坊过滤器池等.
FullDeps:包含客户端实例,交易池实例,选择链策略,链规范等.
这些结构体定义了一系列依赖项,用于构建和配置区块链客户端,支持BABE,GRANDPA和Frontier等不同的共识算法和功能.
*/

pub fn overrides_handle<C, B>(client: Arc<C>) -> Arc<OverrideHandle<Block>>
where
    C: ProvideRuntimeApi<Block> + StorageProvider<Block, B> + AuxStore,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError>,
    C: Send + Sync + 'static,
    C::Api: sp_api::ApiExt<Block>
        + fp_rpc::EthereumRuntimeRPCApi<Block>
        + fp_rpc::ConvertTransactionRuntimeApi<Block>,
    B: Backend<Block> + 'static,
    B::State: StateBackend<BlakeTwo256>,
{
    let mut overrides_map = BTreeMap::new();
    overrides_map.insert(
        EthereumStorageSchema::V1,
        Box::new(SchemaV1Override::new(client.clone()))
            as Box<dyn StorageOverride<_> + Send + Sync>,
    );
    overrides_map.insert(
        EthereumStorageSchema::V2,
        Box::new(SchemaV2Override::new(client.clone()))
            as Box<dyn StorageOverride<_> + Send + Sync>,
    );

    overrides_map.insert(
        EthereumStorageSchema::V3,
        Box::new(SchemaV3Override::new(client.clone()))
            as Box<dyn StorageOverride<_> + Send + Sync>,
    );

    Arc::new(OverrideHandle {
        schemas: overrides_map,
        fallback: Box::new(RuntimeApiStorageOverride::new(client)),
    })
}

/// A IO handler that uses all Full RPC extensions.
pub type RpcExtension = jsonrpc_core::IoHandler<sc_rpc::Metadata>;

/*
 
这个函数定义了一个泛型函数overrides_handle,它接收一个Arc<C>参数,返回一个Arc<OverrideHandle<Block>>.
函数功能是创建并返回一个OverrideHandle结构体的Arc引用.其中,C需要实现多个 trait,并且有特定的约束条件.
函数内部创建了一个BTreeMap,并插入了几个键值对,然后将其和RuntimeApiStorageOverride一起封装成OverrideHandle结构体,
最后通过Arc::new返回这个结构体的Arc引用.根据提供的client参数,创建了一个包含不同版本存储覆盖的OverrideHandle实例,并返回其可变引用.

RpcExtension是一个类型别名,它代表使用所有 Full RPC 扩展的 IO 处理器.
*/


/// Instantiate all Full RPC extensions.
pub fn create_full<C, P, SC, B, A>(
    deps: FullDeps<C, P, SC, B, A>,
    subscription_task_executor: SubscriptionTaskExecutor,
) -> Result<RpcExtension, Box<dyn std::error::Error + Send + Sync>>
where
    C: ProvideRuntimeApi<Block>
        + AuxStore
        + HeaderBackend<Block>
        + HeaderMetadata<Block, Error = BlockChainError>
        + StorageProvider<Block, B>
        + BlockchainEvents<Block>
        + Send
        + Sync
        + 'static,
    C::Api: BlockBuilder<Block>,
    C::Api: BabeApi<Block>,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: xpallet_assets_rpc_runtime_api::XAssetsApi<Block, AccountId, Balance>,
    C::Api:
        xpallet_dex_spot_rpc_runtime_api::XSpotApi<Block, AccountId, Balance, BlockNumber, Balance>,
    C::Api: xpallet_gateway_bitcoin_rpc_runtime_api::XGatewayBitcoinApi<Block, AccountId>,
    C::Api: xpallet_gateway_common_rpc_runtime_api::XGatewayCommonApi<
        Block,
        AccountId,
        Balance,
        BlockNumber,
    >,
    C::Api: xpallet_gateway_records_rpc_runtime_api::XGatewayRecordsApi<
        Block,
        AccountId,
        Balance,
        BlockNumber,
    >,
    C::Api: xpallet_mining_staking_rpc_runtime_api::XStakingApi<
        Block,
        AccountId,
        Balance,
        VoteWeight,
        BlockNumber,
    >,
    C::Api: xpallet_mining_asset_rpc_runtime_api::XMiningAssetApi<
        Block,
        AccountId,
        Balance,
        MiningWeight,
        BlockNumber,
    >,
    C::Api: xpallet_btc_ledger_runtime_api::BtcLedgerApi<Block, AccountId, Balance>,
    C::Api: xpallet_transaction_fee_rpc_runtime_api::XTransactionFeeApi<Block, Balance>,
    C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
    C::Api: fp_rpc::ConvertTransactionRuntimeApi<Block>,
    P: TransactionPool<Block = Block> + Sync + Send + 'static,
    SC: SelectChain<Block> + 'static,
    B: sc_client_api::Backend<Block> + Send + Sync + 'static,
    B::State: sc_client_api::backend::StateBackend<sp_runtime::traits::HashFor<Block>>,
    A: ChainApi<Block = Block> + 'static,
{
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
    use substrate_frame_rpc_system::{FullSystem, SystemApi};
    use xpallet_assets_rpc::{Assets, XAssetsApi};
    use xpallet_btc_ledger_rpc::{BtcLedger, BtcLedgerApi};
    use xpallet_dex_spot_rpc::{XSpot, XSpotApi};
    use xpallet_gateway_bitcoin_rpc::{XGatewayBitcoin, XGatewayBitcoinApi};
    use xpallet_gateway_common_rpc::{XGatewayCommon, XGatewayCommonApi};
    use xpallet_gateway_records_rpc::{XGatewayRecords, XGatewayRecordsApi};
    use xpallet_mining_asset_rpc::{XMiningAsset, XMiningAssetApi};
    use xpallet_mining_staking_rpc::{XStaking, XStakingApi};
    use xpallet_transaction_fee_rpc::{XTransactionFee, XTransactionFeeApi};

    let mut io = jsonrpc_core::IoHandler::default();
    let FullDeps {
        client,
        pool,
        select_chain,
        chain_spec,
        deny_unsafe,
        grandpa,
        babe,
        frontier,
    } = deps;

    let BabeDeps {
        keystore,
        babe_config,
        shared_epoch_changes,
    } = babe;
    let GrandpaDeps {
        shared_voter_state,
        shared_authority_set,
        justification_stream,
        subscription_executor,
        finality_provider,
    } = grandpa;

    io.extend_with(SystemApi::to_delegate(FullSystem::new(
        client.clone(),
        pool.clone(),
        deny_unsafe,
    )));
    io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
        client.clone(),
    )));
    io.extend_with(sc_consensus_babe_rpc::BabeApi::to_delegate(
        BabeRpcHandler::new(
            client.clone(),
            shared_epoch_changes.clone(),
            keystore,
            babe_config,
            select_chain,
            deny_unsafe,
        ),
    ));
    io.extend_with(sc_finality_grandpa_rpc::GrandpaApi::to_delegate(
        GrandpaRpcHandler::new(
            shared_authority_set.clone(),
            shared_voter_state,
            justification_stream,
            subscription_executor,
            finality_provider,
        ),
    ));
    io.extend_with(sc_sync_state_rpc::SyncStateRpcApi::to_delegate(
        sc_sync_state_rpc::SyncStateRpcHandler::new(
            chain_spec,
            client.clone(),
            shared_authority_set,
            shared_epoch_changes,
        )?,
    ));

    io.extend_with(XTransactionFeeApi::to_delegate(XTransactionFee::new(
        client.clone(),
    )));
    io.extend_with(XAssetsApi::to_delegate(Assets::new(client.clone())));
    io.extend_with(XStakingApi::to_delegate(XStaking::new(client.clone())));
    io.extend_with(XSpotApi::to_delegate(XSpot::new(client.clone())));
    io.extend_with(XMiningAssetApi::to_delegate(XMiningAsset::new(
        client.clone(),
    )));
    io.extend_with(XGatewayBitcoinApi::to_delegate(XGatewayBitcoin::new(
        client.clone(),
    )));
    io.extend_with(XGatewayRecordsApi::to_delegate(XGatewayRecords::new(
        client.clone(),
    )));
    io.extend_with(XGatewayCommonApi::to_delegate(XGatewayCommon::new(
        client.clone(),
    )));
    io.extend_with(BtcLedgerApi::to_delegate(BtcLedger::new(client.clone())));

    // EVM
    {
        use fc_rpc::{
            EthApi, EthApiServer, EthFilterApi, EthFilterApiServer, EthPubSubApi,
            EthPubSubApiServer, HexEncodedIdProvider, NetApi, NetApiServer, Web3Api, Web3ApiServer,
        };

        let FrontierDeps {
            graph,
            is_authority,
            network,
            filter_pool,
            backend,
            max_past_logs,
            fee_history_limit,
            fee_history_cache,
            overrides,
            block_data_cache,
        } = frontier;

        let convert_transaction: Option<Never> = None;

        io.extend_with(EthApiServer::to_delegate(EthApi::new(
            client.clone(),
            pool.clone(),
            graph,
            convert_transaction,
            network.clone(),
            Vec::new(),
            overrides.clone(),
            backend.clone(),
            is_authority,
            max_past_logs,
            block_data_cache.clone(),
            fc_rpc::format::Geth,
            fee_history_limit,
            fee_history_cache,
        )));

        if let Some(filter_pool) = filter_pool {
            io.extend_with(EthFilterApiServer::to_delegate(EthFilterApi::new(
                client.clone(),
                backend,
                filter_pool,
                500_usize, // max stored filters
                max_past_logs,
                block_data_cache,
            )));
        }

        io.extend_with(NetApiServer::to_delegate(NetApi::new(
            client.clone(),
            network.clone(),
            // Whether to format the `peer_count` response as Hex (default) or not.
            true,
        )));

        io.extend_with(Web3ApiServer::to_delegate(Web3Api::new(client.clone())));

        io.extend_with(EthPubSubApiServer::to_delegate(EthPubSubApi::new(
            pool,
            client,
            network,
            SubscriptionManager::<HexEncodedIdProvider>::with_id_provider(
                HexEncodedIdProvider::default(),
                Arc::new(subscription_task_executor),
            ),
            overrides,
        )));
    }

    Ok(io)
}

/*
该函数用于创建一个完整的RPC扩展,它依赖于许多其他模块和插件,包括客户端,交易池,选择链,区块链后端等.
函数主要通过将各种API扩展添加到IoHandler中来实现,包括系统API,交易支付API,Babe共识API,Grandpa最终性API等.
此外,还添加了对EVM的支持,包括EthAPI,NetAPI和Web3API等.最后,函数返回一个RpcExtension的结果.

这段代码是一个Rust函数,用于创建一个完整的RPC(远程过程调用)扩展.RPC扩展通常用于区块链节点,允许开发者通过JSON-RPC接口与节点进行交互.
这个函数是Substrate框架的一部分,Substrate是一个用于构建区块链应用的Rust框架.

函数`create_full`接受一个`FullDeps`结构体和一个`SubscriptionTaskExecutor`类型的参数,并返回一个`Result`类型,其中包含`RpcExtension`或者一个错误.

让我们逐步分析这个函数的组成部分:

1. **泛型参数**:函数使用了多个泛型参数,这意味着它可以适用于多种不同类型的区块链配置.
这些参数包括`C`(提供运行时API的客户端),`P`(交易池),`SC`(选择链的逻辑),`B`(区块链的后端),`A`(链API)等.

2. **函数签名**:函数定义了一个泛型函数`create_full`,它接受一个`FullDeps`结构体和一个`SubscriptionTaskExecutor`,
并返回一个`Result`,其中包含`RpcExtension`或错误.

3. **依赖项**:`FullDeps`结构体包含了创建RPC扩展所需的所有依赖项,如客户端,交易池,选择链逻辑,链规范,拒绝不安全的交易,Grandpa和Babe共识机制的依赖项等.

4. **IoHandler**:创建了一个`jsonrpc_core::IoHandler`实例,这是一个处理RPC请求的核心组件.它将被用来注册各种RPC API.

5. **注册API**:函数中通过`extend_with`方法向`IoHandler`注册了多种RPC API.
例如,`SystemApi`,`TransactionPaymentApi`,`BabeApi`,`GrandpaApi`等.这些API提供了与区块链交互的各种功能,
如获取系统信息,处理交易支付,处理Babe共识机制等.

6. **EVM相关API**:代码中还包含了与以太坊虚拟机(EVM)相关的API,如`EthApi`,`EthFilterApi`,`NetApi`,`Web3Api`和`EthPubSubApi`.
这些API允许与EVM进行交互,如获取以太坊网络信息,过滤日志,订阅事件等.

7. **返回结果**:最后,函数返回了配置好的`IoHandler`,现在它已经注册了所有必要的RPC API,可以被用来处理来自客户端的RPC请求.

总的来说,这个函数是区块链节点RPC服务的核心部分,它通过注册各种API来提供与区块链交互的能力.这对于开发者来说是非常有用的,
因为它允许他们通过JSON-RPC接口与区块链节点进行通信,执行各种操作,如发送交易,查询状态,监听事件等.

*/
