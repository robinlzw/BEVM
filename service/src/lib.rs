// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.
#![allow(clippy::type_complexity)]
use sc_client_api::{BlockBackend, ExecutorProvider};
use sc_consensus_babe::SlotProportion;
use sc_executor::{NativeElseWasmExecutor, NativeExecutionDispatch};
use sc_finality_grandpa::FinalityProofProvider as GrandpaFinalityProofProvider;
use sc_network::{Event, NetworkService};
use sc_service::{config::Configuration, error::Error as ServiceError, RpcHandlers, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sp_api::ConstructRuntimeApi;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;
use std::time::Duration;

use chainx_primitives::Block;

mod client;
use client::RuntimeApiCollection;

// EVM
use fc_consensus::FrontierBlockImport;
use fc_mapping_sync::{MappingSyncWorker, SyncStrategy::Normal};
use fc_rpc::EthTask;
use fc_rpc_core::types::{FeeHistoryCache, FilterPool};
use futures::StreamExt;
use maplit::hashmap;
use sc_client_api::BlockchainEvents;
use sc_keystore::LocalKeystore;
use sc_service::config::PrometheusConfig;
use sc_service::BasePath;
use std::{collections::BTreeMap, sync::Mutex};
use substrate_prometheus_endpoint::Registry;

type FullClient<RuntimeApi, Executor> =
    sc_service::TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<Executor>>;

type FullBackend = sc_service::TFullBackend<Block>;

type FullGrandpaBlockImport<RuntimeApi, Executor> = sc_finality_grandpa::GrandpaBlockImport<
    FullBackend,
    Block,
    FullClient<RuntimeApi, Executor>,
    FullSelectChain,
>;

type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

pub type ConsensusResult<RuntimeApi, Executor> = (
    sc_finality_grandpa::GrandpaBlockImport<
        FullBackend,
        Block,
        FullClient<RuntimeApi, Executor>,
        FullSelectChain,
    >,
    sc_finality_grandpa::LinkHalf<Block, FullClient<RuntimeApi, Executor>, FullSelectChain>,
);

pub fn frontier_database_dir(config: &Configuration) -> std::path::PathBuf {
    let config_dir = config
        .base_path
        .as_ref()
        .map(|base_path| base_path.config_dir(config.chain_spec.id()))
        .unwrap_or_else(|| {
            BasePath::from_project("", "", "chainx").config_dir(config.chain_spec.id())
        });
    config_dir.join("frontier").join("db")
}

pub fn open_frontier_backend(config: &Configuration) -> Result<Arc<fc_db::Backend<Block>>, String> {
    Ok(Arc::new(fc_db::Backend::<Block>::new(
        &fc_db::DatabaseSettings {
            source: fc_db::DatabaseSettingsSrc::RocksDb {
                path: frontier_database_dir(config),
                cache_size: 0,
            },
        },
    )?))
}

// If we're using prometheus, use a registry with a prefix of `frontier`.
fn set_prometheus_registry(config: &mut Configuration) -> Result<(), ServiceError> {
    if let Some(PrometheusConfig { registry, .. }) = config.prometheus_config.as_mut() {
        let labels = hashmap! {
            "chain".into() => config.chain_spec.id().into(),
        };
        *registry = Registry::new_custom(Some("frontier".into()), Some(labels))?;
    }

    Ok(())
}

/*
这段Rust代码定义了一个名为 `new_partial` 的函数,它是Substrate节点启动过程中的一部分,用于创建节点的部分组件.
这个函数不接受远程密钥库配置,并设置了Prometheus监控和Telemetry(遥测)服务,创建了执行器,初始化了客户端,后端和任务管理器,
并配置了交易池,共识导入队列和其他相关组件.以下是对函数及其内部逻辑的详细解释:

### 函数签名

- `pub fn new_partial<RuntimeApi, Executor>(config: &mut Configuration) -> Result<..., ServiceError>`: 
这是一个公共函数,它接受一个可变引用的 `Configuration` 对象作为参数,并返回一个 `Result` 类型的结果.
`Result` 包含 `sc_service::PartialComponents` 结构体的实例或 `ServiceError` 错误.

### 函数实现

1. **远程密钥库检查**:如果配置中包含远程密钥库,则返回错误,因为远程密钥库不受支持.
2. **Prometheus监控设置**:通过调用 `set_prometheus_registry` 函数,根据配置设置Prometheus监控注册表.
3. **Telemetry初始化**:如果配置了Telemetry端点,则创建Telemetry工作器并启动它.
4. **执行器创建**:使用 `NativeElseWasmExecutor` 创建一个新的执行器,这个执行器根据配置决定是使用原生执行还是WASM执行.
5. **客户端和后端初始化**:通过调用 `sc_service::new_full_parts` 函数,创建客户端,后端和任务管理器的组件.
6. **选择链逻辑**:使用 `sc_consensus::LongestChain` 创建一个新的选择链逻辑实例.
7. **交易池初始化**:创建一个新的完整交易池实例.
8. **过滤器池和费用历史缓存初始化**:创建过滤器池和费用历史缓存实例.
9. **Frontier后端打开**:打开Frontier后端数据库.
10. **GRANDPA和BABE共识组件初始化**:创建GRANDPA区块导入,BABE区块导入和链接实例.
11. **导入队列设置**:创建BABE导入队列,并配置相关的内在数据提供者.
12. **组装部分组件**:将所有创建的组件组装成 `sc_service::PartialComponents` 结构体,并返回.

### 返回类型

- `sc_service::PartialComponents`: 这是一个包含节点部分组件的结构体,它包括客户端,后端,任务管理器,选择链,交易池,导入队列和其他相关设置.
*/
pub fn new_partial<RuntimeApi, Executor>(
    config: &mut Configuration,
) -> Result<
    sc_service::PartialComponents<
        FullClient<RuntimeApi, Executor>,
        FullBackend,
        FullSelectChain,
        sc_consensus::DefaultImportQueue<Block, FullClient<RuntimeApi, Executor>>,
        sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi, Executor>>,
        (
            (
                sc_consensus_babe::BabeBlockImport<
                    Block,
                    FullClient<RuntimeApi, Executor>,
                    FullGrandpaBlockImport<RuntimeApi, Executor>,
                >,
                sc_finality_grandpa::LinkHalf<
                    Block,
                    FullClient<RuntimeApi, Executor>,
                    FullSelectChain,
                >,
                sc_consensus_babe::BabeLink<Block>,
            ),
            Option<Telemetry>,
            (
                Option<FilterPool>,
                FeeHistoryCache,
                Arc<fc_db::Backend<Block>>,
            ),
        ),
    >,
    ServiceError,
>
where
    RuntimeApi:
        ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi:
        RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
    Executor: NativeExecutionDispatch + 'static,
{
    if config.keystore_remote.is_some() {
        return Err(ServiceError::Other(
            "Remote Keystores are not supported.".into(),
        ));
    }

    set_prometheus_registry(config)?;

    let telemetry = config
        .telemetry_endpoints
        .clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, sc_telemetry::Error> {
            let worker = TelemetryWorker::new(16)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    let executor = NativeElseWasmExecutor::<Executor>::new(
        config.wasm_method,
        config.default_heap_pages,
        config.max_runtime_instances,
        config.runtime_cache_size,
    );

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, NativeElseWasmExecutor<Executor>>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
        )?;
    let client = Arc::new(client);

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager
            .spawn_handle()
            .spawn("telemetry", None, worker.run());
        telemetry
    });

    let select_chain = sc_consensus::LongestChain::new(backend.clone());

    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_essential_handle(),
        client.clone(),
    );

    let filter_pool: Option<FilterPool> = Some(Arc::new(Mutex::new(BTreeMap::new())));
    let fee_history_cache: FeeHistoryCache = Arc::new(Mutex::new(BTreeMap::new()));
    let frontier_backend = open_frontier_backend(config)?;

    let (grandpa_block_import, grandpa_link) = sc_finality_grandpa::block_import(
        client.clone(),
        &(client.clone() as Arc<_>),
        select_chain.clone(),
        telemetry.as_ref().map(|x| x.handle()),
    )?;
    let justification_import = grandpa_block_import.clone();

    let (babe_block_import, babe_link) = sc_consensus_babe::block_import(
        sc_consensus_babe::Config::get(&*client)?,
        grandpa_block_import,
        client.clone(),
    )?;

    let frontier_block_import = FrontierBlockImport::new(
        babe_block_import.clone(),
        client.clone(),
        frontier_backend.clone(),
    );

    let slot_duration = babe_link.config().slot_duration();
    let import_queue = sc_consensus_babe::import_queue(
        babe_link.clone(),
        frontier_block_import,
        Some(Box::new(justification_import)),
        client.clone(),
        select_chain.clone(),
        move |_, ()| async move {
            let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

            let slot =
                sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                    *timestamp,
                    slot_duration,
                );

            let uncles =
                sp_authorship::InherentDataProvider::<<Block as BlockT>::Header>::check_inherents();

            Ok((timestamp, slot, uncles))
        },
        &task_manager.spawn_essential_handle(),
        config.prometheus_registry(),
        sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone()),
        telemetry.as_ref().map(|x| x.handle()),
    )?;

    let import_setup = (babe_block_import, grandpa_link, babe_link);
    let frontier_setup = (filter_pool, fee_history_cache, frontier_backend);

    Ok(sc_service::PartialComponents {
        client,
        backend,
        task_manager,
        keystore_container,
        select_chain,
        import_queue,
        transaction_pool,
        other: (import_setup, telemetry, frontier_setup),
    })
}

/*
这段Rust代码定义了一个名为 `NewFullBase` 的结构体,它似乎是Substrate框架中用于初始化完整节点服务的一部分.
`NewFullBase` 结构体包含了在节点启动时需要的各种组件和服务的引用.下面是对结构体及其成员的详细解释:

### `NewFullBase` 结构体

- `RuntimeApi`: 这是一个泛型参数,代表运行时API的类型.这个类型需要实现 `ConstructRuntimeApi` trait,
这个trait用于构造运行时API,并且需要满足一些其他的限制(如 `Send`, `Sync`, `'static`),以确保它可以在多线程环境中安全使用.
- `Executor`: 这是另一个泛型参数,代表执行器的类型.执行器需要实现 `NativeExecutionDispatch` trait,这个trait定义了如何执行原生(WASM)代码.

### 约束条件

- `RuntimeApi` 需要实现 `ConstructRuntimeApi` trait,这个trait允许 `RuntimeApi` 被构造用于处理区块(`Block`)和完整客户端(`FullClient`).
- `RuntimeApi::RuntimeApi` 需要实现 `RuntimeApiCollection` trait,这个trait定义了一个运行时API集合.这里的 `StateBackend` 
被指定为 `sc_client_api::StateBackendFor<FullBackend, Block>`,这意味着状态后端用于存储和检索链的状态信息.
- `Executor` 需要实现 `NativeExecutionDispatch` trait,这个trait定义了如何调度和执行原生运行时代码.

### 结构体字段

- `task_manager`: 节点的任务管理器.这个字段包含一个 `TaskManager` 类型的值,负责管理和调度节点的各种后台任务.
- `client`: 节点的客户端实例.这个字段是一个 `Arc`(原子引用计数)智能指针,指向 `FullClient` 类型的值,它包含了与区块链交互所需的所有信息和状态.
- `network`: 节点的网络服务.这个字段也是一个 `Arc` 智能指针,指向 `NetworkService` 类型的值,负责处理节点的网络通信,如区块和交易的传播.
- `transaction_pool`: 节点的交易池.这个字段是一个 `Arc` 智能指针,指向 `sc_transaction_pool::FullPool` 类型的值,
它管理待处理的交易,以及与交易相关的其他任务,如交易选择和验证.
- `rpc_handlers`: 节点的RPC处理器.这个字段包含了 `RpcHandlers` 类型的值,负责处理通过RPC(远程过程调用)接口发起的请求.

`NewFullBase` 结构体的设计允许它在创建节点服务时,将所有必要的组件封装在一起,从而简化了节点的初始化过程.通过使用 `Arc` 智能指针,
它确保了这些组件可以在多线程环境中安全地共享和访问.
*/
pub struct NewFullBase<RuntimeApi, Executor>
where
    RuntimeApi:
        ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi:
        RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
    Executor: NativeExecutionDispatch + 'static,
{
    /// The task manager of the node.
    pub task_manager: TaskManager,
    /// The client instance of the node.
    pub client: Arc<FullClient<RuntimeApi, Executor>>,
    /// The networking service of the node.
    pub network: Arc<NetworkService<Block, <Block as BlockT>::Hash>>,
    /// The transaction pool of the node.
    pub transaction_pool:
        Arc<sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi, Executor>>>,
    /// The rpc handlers of the node.
    pub rpc_handlers: RpcHandlers,
}

fn remote_keystore(_url: &str) -> Result<Arc<LocalKeystore>, &'static str> {
    // FIXME: here would the concrete keystore be built,
    //        must return a concrete type (NOT `LocalKeystore`) that
    //        implements `CryptoStore` and `SyncCryptoStore`
    Err("Remote Keystore not supported.")
}

/// Creates a full service from the configuration.
// 1.创建部分组件:通过调用 new_partial 函数,从配置中创建了节点的部分组件,包括客户端,后端,任务管理器,导入队列,密钥库容器,选择链和交易池等.
// 2.远程密钥库:如果配置中指定了远程密钥库的URL,尝试连接并设置远程密钥库.
// 3.导入设置:从 new_partial 函数返回的 PartialComponents 中提取导入设置,包括BABE区块导入,GRANDPA链接和BABE链接.
// 4.网络和RPC:构建网络服务,系统RPC传输通道和网络启动器,并根据配置启动网络.
// 5.离线工作器:如果启用了离线工作器,构建并启动它们.
// 6.角色和共识:根据配置确定节点的角色(权威节点或普通节点),并相应地设置共识机制.如果节点是权威节点,创建BABE提议者和GRANDPA投票者.
// 7.RPC扩展:构建RPC扩展,包括必要的依赖项和任务执行器.
// 8.启动任务:使用 spawn_tasks 函数启动RPC和其他相关任务.
// 9.Frontier EVM任务:启动与Frontier EVM相关的维护任务,包括映射同步工作器,过滤器池任务和费用历史缓存任务.
// 10.启动GRANDPA:如果启用了GRANDPA共识机制,启动GRANDPA投票者.
// 11.网络启动:启动网络.
// 12.返回结果:如果所有步骤都成功完成,返回包含任务管理器,客户端,网络和交易池的 NewFullBase 结构体实例.
pub fn new_full_base<RuntimeApi, Executor>(
    mut config: Configuration,
) -> Result<NewFullBase<RuntimeApi, Executor>, ServiceError>
where
    RuntimeApi:
        ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi:
        RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
    Executor: NativeExecutionDispatch + 'static,
{
    let sc_service::PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue,
        mut keystore_container,
        select_chain,
        transaction_pool,
        other: (import_setup, mut telemetry, frontier_setup),
    } = new_partial(&mut config)?;

    if let Some(url) = &config.keystore_remote {
        match remote_keystore(url) {
            Ok(k) => keystore_container.set_remote_keystore(k),
            Err(e) => {
                return Err(ServiceError::Other(format!(
                    "Error hooking up remote keystore for {}: {}",
                    url, e
                )))
            }
        };
    }

    let (babe_block_import, grandpa_link, babe_link) = import_setup;
    let (filter_pool, fee_history_cache, frontier_backend) = frontier_setup;

    let auth_disc_publish_non_global_ips = config.network.allow_non_globals_in_dht;
    let grandpa_protocol_name = sc_finality_grandpa::protocol_standard_name(
        &client
            .block_hash(0)
            .ok()
            .flatten()
            .expect("Genesis block exists; qed"),
        &config.chain_spec,
    );

    config
        .network
        .extra_sets
        .push(sc_finality_grandpa::grandpa_peers_set_config(
            grandpa_protocol_name.clone(),
        ));

    let warp_sync = Arc::new(sc_finality_grandpa::warp_proof::NetworkProvider::new(
        backend.clone(),
        grandpa_link.shared_authority_set().clone(),
        Vec::default(),
    ));

    let (network, system_rpc_tx, network_starter) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            block_announce_validator_builder: None,
            warp_sync: Some(warp_sync),
        })?;

    if config.offchain_worker.enabled {
        sc_service::build_offchain_workers(
            &config,
            task_manager.spawn_handle(),
            client.clone(),
            network.clone(),
        );
    }

    let role = config.role.clone();
    let force_authoring = config.force_authoring;
    // we are not interested in using any backoff from block authoring in case finality is
    // lagging, in particular because we use a small session duration (50 slots) and this
    // could be problematic.
    let backoff_authoring_blocks: Option<()> = None;
    let name = config.network.node_name.clone();
    let enable_grandpa = !config.disable_grandpa;
    let prometheus_registry = config.prometheus_registry().cloned();

    // EVM
    let subscription_task_executor =
        sc_rpc::SubscriptionTaskExecutor::new(task_manager.spawn_handle());
    let overrides = chainx_rpc::overrides_handle(client.clone());
    let fee_history_limit = 2048;
    let block_data_cache = Arc::new(fc_rpc::EthBlockDataCacheTask::new(
        task_manager.spawn_handle(),
        overrides.clone(),
        50,
        50,
        prometheus_registry.clone(),
    ));

    let rpc_extensions_builder = {
        let justification_stream = grandpa_link.justification_stream();
        let shared_authority_set = grandpa_link.shared_authority_set().clone();

        let finality_proof_provider = GrandpaFinalityProofProvider::new_for_service(
            backend.clone(),
            Some(shared_authority_set.clone()),
        );

        let babe_config = babe_link.config().clone();
        let shared_epoch_changes = babe_link.epoch_changes().clone();

        let client = client.clone();
        let pool = transaction_pool.clone();
        let select_chain = select_chain.clone();
        let keystore = keystore_container.sync_keystore();
        let chain_spec = config.chain_spec.cloned_box();

        // EVM
        let network = network.clone();
        let filter_pool = filter_pool.clone();
        let frontier_backend = frontier_backend.clone();
        let overrides = overrides.clone();
        let fee_history_cache = fee_history_cache.clone();
        let is_authority = false;
        let max_past_logs = 10000;

        Box::new(move |deny_unsafe, subscription_executor| {
            let deps = chainx_rpc::FullDeps {
                client: client.clone(),
                pool: pool.clone(),
                select_chain: select_chain.clone(),
                chain_spec: chain_spec.cloned_box(),
                deny_unsafe,
                babe: chainx_rpc::BabeDeps {
                    babe_config: babe_config.clone(),
                    shared_epoch_changes: shared_epoch_changes.clone(),
                    keystore: keystore.clone(),
                },
                grandpa: chainx_rpc::GrandpaDeps {
                    shared_voter_state: sc_finality_grandpa::SharedVoterState::empty(),
                    shared_authority_set: shared_authority_set.clone(),
                    justification_stream: justification_stream.clone(),
                    subscription_executor,
                    finality_provider: finality_proof_provider.clone(),
                },
                frontier: chainx_rpc::FrontierDeps {
                    graph: pool.pool().clone(),
                    is_authority,
                    network: network.clone(),
                    filter_pool: filter_pool.clone(),
                    backend: frontier_backend.clone(),
                    max_past_logs,
                    fee_history_limit,
                    fee_history_cache: fee_history_cache.clone(),
                    overrides: overrides.clone(),
                    block_data_cache: block_data_cache.clone(),
                },
            };

            chainx_rpc::create_full(deps, subscription_task_executor.clone()).map_err(Into::into)
        })
    };

    let rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        config,
        backend: backend.clone(),
        client: client.clone(),
        keystore: keystore_container.sync_keystore(),
        network: network.clone(),
        rpc_extensions_builder,
        transaction_pool: transaction_pool.clone(),
        task_manager: &mut task_manager,
        system_rpc_tx,
        telemetry: telemetry.as_mut(),
    })?;

    task_manager.spawn_essential_handle().spawn(
        "frontier-mapping-sync-worker",
        Some("frontier"),
        MappingSyncWorker::new(
            client.import_notification_stream(),
            Duration::new(6, 0),
            client.clone(),
            backend,
            frontier_backend.clone(),
            3,
            0,
            Normal,
        )
        .for_each(|()| futures::future::ready(())),
    );

    // Spawn Frontier EthFilterApi maintenance task.
    if let Some(filter_pool) = filter_pool {
        // Each filter is allowed to stay in the pool for 100 blocks.
        const FILTER_RETAIN_THRESHOLD: u64 = 100;
        task_manager.spawn_essential_handle().spawn(
            "frontier-filter-pool",
            Some("frontier"),
            EthTask::filter_pool_task(Arc::clone(&client), filter_pool, FILTER_RETAIN_THRESHOLD),
        );
    }

    // Spawn Frontier FeeHistory cache maintenance task.
    task_manager.spawn_essential_handle().spawn(
        "frontier-fee-history",
        Some("frontier"),
        EthTask::fee_history_task(
            Arc::clone(&client),
            Arc::clone(&overrides),
            fee_history_cache,
            fee_history_limit,
        ),
    );

    task_manager.spawn_essential_handle().spawn(
        "frontier-schema-cache-task",
        Some("frontier"),
        EthTask::ethereum_schema_cache_task(Arc::clone(&client), Arc::clone(&frontier_backend)),
    );

    if let sc_service::config::Role::Authority { .. } = &role {
        let proposer = sc_basic_authorship::ProposerFactory::new(
            task_manager.spawn_handle(),
            client.clone(),
            transaction_pool.clone(),
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|x| x.handle()),
        );

        let can_author_with =
            sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone());

        let client_clone = client.clone();
        let slot_duration = babe_link.config().slot_duration();
        let babe_config = sc_consensus_babe::BabeParams {
            keystore: keystore_container.sync_keystore(),
            client: client.clone(),
            select_chain,
            env: proposer,
            block_import: babe_block_import,
            sync_oracle: network.clone(),
            justification_sync_link: network.clone(),
            create_inherent_data_providers: move |parent, ()| {
                let client_clone = client_clone.clone();
                async move {
                    let uncles = sc_consensus_uncles::create_uncles_inherent_data_provider(
                        &*client_clone,
                        parent,
                    )?;

                    let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                    let slot = sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                        *timestamp,
                        slot_duration,
                    );

                    Ok((timestamp, slot, uncles))
                }
            },
            force_authoring,
            backoff_authoring_blocks,
            babe_link,
            can_author_with,
            block_proposal_slot_portion: SlotProportion::new(0.5),
            max_block_proposal_slot_portion: None,
            telemetry: telemetry.as_ref().map(|x| x.handle()),
        };

        let babe = sc_consensus_babe::start_babe(babe_config)?;
        task_manager.spawn_essential_handle().spawn_blocking(
            "babe-proposer",
            Some("block-authoring"),
            babe,
        );
    }

    // Spawn authority discovery module.
    if role.is_authority() {
        let authority_discovery_role =
            sc_authority_discovery::Role::PublishAndDiscover(keystore_container.keystore());
        let dht_event_stream =
            network
                .event_stream("authority-discovery")
                .filter_map(|e| async move {
                    match e {
                        Event::Dht(e) => Some(e),
                        _ => None,
                    }
                });
        let (authority_discovery_worker, _service) =
            sc_authority_discovery::new_worker_and_service_with_config(
                sc_authority_discovery::WorkerConfig {
                    publish_non_global_ips: auth_disc_publish_non_global_ips,
                    ..Default::default()
                },
                client.clone(),
                network.clone(),
                Box::pin(dht_event_stream),
                authority_discovery_role,
                prometheus_registry.clone(),
            );

        task_manager.spawn_handle().spawn(
            "authority-discovery-worker",
            Some("networking"),
            authority_discovery_worker.run(),
        );
    }

    // if the node isn't actively participating in consensus then it doesn't
    // need a keystore, regardless of which protocol we use below.
    let keystore = if role.is_authority() {
        Some(keystore_container.sync_keystore())
    } else {
        None
    };

    let config = sc_finality_grandpa::Config {
        // FIXME #1578 make this available through chainspec
        gossip_duration: Duration::from_millis(333),
        justification_period: 512,
        name: Some(name),
        observer_enabled: false,
        keystore,
        local_role: role,
        telemetry: telemetry.as_ref().map(|x| x.handle()),
        protocol_name: grandpa_protocol_name,
    };

    if enable_grandpa {
        // start the full GRANDPA voter
        // NOTE: non-authorities could run the GRANDPA observer protocol, but at
        // this point the full voter should provide better guarantees of block
        // and vote data availability than the observer. The observer has not
        // been tested extensively yet and having most nodes in a network run it
        // could lead to finality stalls.
        let grandpa_config = sc_finality_grandpa::GrandpaParams {
            config,
            link: grandpa_link,
            network: network.clone(),
            telemetry: telemetry.as_ref().map(|x| x.handle()),
            voting_rule: sc_finality_grandpa::VotingRulesBuilder::default().build(),
            prometheus_registry,
            shared_voter_state: sc_finality_grandpa::SharedVoterState::empty(),
        };

        // the GRANDPA voter task is considered infallible, i.e.
        // if it fails we take down the service with it.
        task_manager.spawn_essential_handle().spawn_blocking(
            "grandpa-voter",
            None,
            sc_finality_grandpa::run_grandpa_voter(grandpa_config)?,
        );
    }

    network_starter.start_network();

    Ok(NewFullBase {
        task_manager,
        client,
        network,
        transaction_pool,
        rpc_handlers,
    })
}

/// Builds a new service for a full client.
pub fn new_full<RuntimeApi, Executor>(config: Configuration) -> Result<TaskManager, ServiceError>
where
    RuntimeApi:
        ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi:
        RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
    Executor: NativeExecutionDispatch + 'static,
{
    new_full_base(config).map(|base: NewFullBase<RuntimeApi, Executor>| base.task_manager)
}

/// Can be called for a `Configuration` to check if it is a configuration for the `ChainX` network.
pub trait IdentifyVariant {
    /// Returns if this is a configuration for the `ChainX` network.
    fn is_chainx(&self) -> bool;

    /// Returns if this is a configuration for the `Malan` network.
    fn is_malan(&self) -> bool;

    /// Returns if this is a configuration for the `Development` network.
    fn is_dev(&self) -> bool;
}

impl IdentifyVariant for Box<dyn sc_service::ChainSpec> {
    fn is_chainx(&self) -> bool {
        self.id() == "chainx"
    }
    fn is_malan(&self) -> bool {
        self.id().contains("malan")
    }
    fn is_dev(&self) -> bool {
        self.id() == "dev"
    }
}

pub fn build_full(config: Configuration) -> Result<TaskManager, ServiceError> {
    if config.chain_spec.is_chainx() {
        new_full::<chainx_runtime::RuntimeApi, chainx_executor::ChainXExecutor>(config)
    } else if config.chain_spec.is_malan() {
        new_full::<malan_runtime::RuntimeApi, chainx_executor::MalanExecutor>(config)
    } else {
        new_full::<dev_runtime::RuntimeApi, chainx_executor::DevExecutor>(config)
    }
}

/*

这段代码是ChainX项目的节点服务和工厂实现,它提供了一个专门针对Substrate服务的包装器.
这个包装器允许ChainX节点与区块链网络交互,处理交易,共识,状态同步等核心功能.代码使用了Rust语言和Substrate框架,下面是对代码的详细解释:

1. **模块和特性导入**:代码开始部分导入了所需的外部模块和特性,包括客户端API,共识机制,网络服务,任务管理器,RPC处理程序等.

2. **类型定义**:定义了一系列与Substrate节点相关的类型别名,如`FullClient`,`FullBackend`,`FullGrandpaBlockImport`等,这些类型别名简化了代码中的类型表达.

3. **`frontier_database_dir` 函数**:根据配置计算Frontier数据库的目录路径.

4. **`open_frontier_backend` 函数**:打开并返回Frontier数据库后端的Arc(原子引用计数).

5. **`set_prometheus_registry` 函数**:如果配置了Prometheus监控,为Frontier设置一个带有前缀的Prometheus注册表.

6. **`new_partial` 函数**:创建节点的部分组件,这些组件是构建完整节点服务的基础.

7. **`NewFullBase` 结构体**:包含了节点服务所需的所有组件,如任务管理器,客户端,网络服务,交易池和RPC处理程序.

8. **`new_full_base` 函数**:根据配置创建一个完整的节点服务基础结构.

9. **`new_full` 函数**:根据配置创建一个完整的节点服务,这是节点启动的主要入口点.

10. **`IdentifyVariant` trait**:提供了一种方法来识别配置是针对哪个网络(ChainX,Malan或Development).

11. **`build_full` 函数**:根据链的规范构建并返回一个完整的节点服务.

整体来看,这段代码是ChainX节点的核心部分,它定义了节点如何与区块链网络交互,如何处理交易和共识,以及如何通过RPC接口与外部应用通信.
代码的结构和注释清晰地展示了各个组件的作用和相互之间的关系,为ChainX项目提供了一个健壮和可扩展的节点实现.
*/
