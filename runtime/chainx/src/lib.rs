// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

//! The Substrate Node Template runtime. This can be compiled with `#[no_std]`, ready for Wasm.
#![allow(clippy::unnecessary_cast)]
#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use static_assertions::const_assert;

use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
#[cfg(feature = "runtime-benchmarks")]
use sp_runtime::RuntimeString;
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    traits::{
        self, AccountIdConversion, BlakeTwo256, Block as BlockT, Convert, DispatchInfoOf,
        NumberFor, OpaqueKeys, SaturatedConversion, Saturating, SignedExtension, StaticLookup,
    },
    transaction_validity::{
        InvalidTransaction, TransactionPriority, TransactionSource, TransactionValidity,
        TransactionValidityError, ValidTransaction,
    },
    ApplyExtrinsicResult, DispatchError, Perbill, Percent, Permill, RuntimeDebug,
};
use sp_std::{collections::btree_map::BTreeMap, prelude::*};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use frame_system::EnsureRoot;
use pallet_grandpa::{
    fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_session::historical as pallet_session_historical;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryPlainSlots;

use chainx_runtime_common::{BlockLength, BlockWeights, BASE_FEE};
use xpallet_dex_spot::{Depth, FullPairInfo, RpcOrder, TradingPairId};
use xpallet_mining_asset::{MinerLedger, MiningAssetInfo, MiningDividendInfo};
use xpallet_mining_staking::{NominatorInfo, NominatorLedger, ValidatorInfo};
use xpallet_support::traits::MultisigAddressFor;

// A few exports that help ease life for downstream crates.
pub use frame_support::{
    construct_runtime, debug, parameter_types,
    traits::{
        ConstBool, ConstU32, Contains, Currency, EnsureOneOf, EqualPrivilegeOnly, Get, Imbalance,
        InstanceFilter, KeyOwnerProofSystem, LockIdentifier, OnRuntimeUpgrade, OnUnbalanced,
        Randomness,
    },
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
        Weight,
    },
    PalletId, StorageValue,
};
pub use pallet_timestamp::Call as TimestampCall;

pub use chainx_primitives::{
    AccountId, AccountIndex, AddrStr, Amount, AssetId, Balance, BlockNumber, ChainAddress, Hash,
    Index, Moment, ReferralId, Signature, Token,
};
pub use sp_staking::SessionIndex;
pub use xp_protocol::*;
pub use xp_runtime::Memo;

// xpallet re-exports
pub use xpallet_assets::{
    AssetInfo, AssetRestrictions, AssetType, Chain, TotalAssetInfo, WithdrawalLimit,
};
#[cfg(feature = "std")]
pub use xpallet_gateway_bitcoin::h256_rev;
pub use xpallet_gateway_bitcoin::{
    hash_rev, types::BtcHeaderInfo, BtcHeader, BtcNetwork, BtcParams, BtcTxVerifier,
    BtcWithdrawalProposal, Compact, H256,
};
pub use xpallet_gateway_common::{
    trustees,
    types::{
        GenericTrusteeIntentionProps, GenericTrusteeSessionInfo, ScriptInfo, TrusteeInfoConfig,
    },
};
pub use xpallet_gateway_records::{Withdrawal, WithdrawalRecordId};
pub use xpallet_mining_asset::MiningWeight;
pub use xpallet_mining_staking::VoteWeight;

/// Constant values used within the runtime.
pub mod constants;
/// Implementations of some helper traits passed into runtime modules as associated types.
pub mod impls;
mod migrations;

use self::constants::{currency::*, time::*};
use self::impls::{ChargeExtraFee, DealWithBTCFees, DealWithFees, SlowAdjustingFeeUpdate};

// EVM
use chainx_runtime_common::NORMAL_DISPATCH_RATIO;
use fp_rpc::TransactionStatus;
use pallet_ethereum::{Call::transact, Transaction as EthereumTransaction};
#[cfg(feature = "std")]
pub use pallet_evm::GenesisAccount;
use pallet_evm::{
    Account as EVMAccount, EnsureAddressNever, EnsureAddressRoot, FeeCalculator,
    HashedAddressMapping, Runner,
};
use sp_core::{H160, U256};
use sp_runtime::traits::{Dispatchable, PostDispatchInfoOf};
mod precompiles;
mod withdraw;

pub use precompiles::ChainXPrecompiles;

/*
Frontier 是 Substrate 框架的一个模块,它提供了与以太坊虚拟机(Ethereum Virtual Machine, EVM)相关的功能.
Frontier 是 Substrate 支持以太坊智能合约和与以太坊兼容的交易的一个重要组成部分.
它允许开发者在 Substrate 基础上构建的区块链上部署和执行 EVM 智能合约,从而利用以太坊庞大的智能合约生态系统.

Frontier 模块的主要特点和功能包括:

1. **EVM 支持**:Frontier 模块实现了 Ethereum 的虚拟机,使得 Substrate 链能够执行 EVM 字节码,从而兼容现有的以太坊智能合约.

2. **预编译合约**:Frontier 包含了一组预编译合约,这些是以太坊网络上内置的合约,用于执行特定的计算任务,如加密算法和数学运算.

3. **交易处理**:Frontier 模块负责处理以太坊风格的交易,包括交易的验证,执行和状态转换.

4. **账户管理**:Frontier 支持以太坊的账户模型,包括账户的创建,余额管理和 nonce(交易计数器)的更新.

5. **链上存储**:Frontier 管理智能合约的状态存储,确保合约的状态在交易执行后得到正确的更新和保存.

6. **Gas 计费**:Frontier 实现了以太坊的 Gas 计费机制,确保执行交易和智能合约操作需要消耗 Gas,从而防止资源滥用.

7. **以太坊 API**:Frontier 提供了一系列以太坊 JSON-RPC API,使得开发者可以使用现有的以太坊工具和库与 Substrate 链进行交互.

Frontier 模块使得 Substrate 框架能够兼容以太坊的智能合约.
通过 Frontier,Substrate 链可以更容易地集成到现有的以太坊生态系统中,同时也为以太坊开发者提供了一个新的平台来部署他们的应用.
*/

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("chainx"),
    impl_name: create_runtime_str!("chainx-net"),
    authoring_version: 1,
    spec_version: 32,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 7,
    state_version: 0,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}
/*
native_version 函数定义了一个 NativeVersion 结构体,它在编译为本地代码时用于标识运行时版本.
这个结构体包含了 runtime_version 字段,它存储了 VERSION 结构体的副本,以及 can_author_with 字段,后者指定了可以用于出块的密钥类型.

在实际的区块链项目中,这些版本信息有助于确保节点之间的兼容性,以及在进行软件升级时能够正确地处理状态迁移和数据兼容性问题.
通过维护详细的版本信息,区块链项目可以支持平滑的升级路径,同时允许开发者和用户跟踪软件的变化.
*/

/// The BABE epoch configuration at genesis.
/// The existing chain is running with PrimaryAndSecondaryPlainSlots,
/// you should keep returning the same thing in BabeApi::configuration()
/// as you were doing before.
///
/// Edit: it's okay to change this here as BabeApi::configuration()
/// is only used on genesis, so this change won't have any effect on
/// the existing chains. But maybe it makes it more clear if you still
/// keep the original value.
pub const BABE_GENESIS_EPOCH_CONFIG: sp_consensus_babe::BabeEpochConfiguration =
    sp_consensus_babe::BabeEpochConfiguration {
        c: PRIMARY_PROBABILITY,
        allowed_slots: PrimaryAndSecondaryPlainSlots,
    };

#[derive(Debug, Clone, Eq, PartialEq, codec::Encode, codec::Decode, MaxEncodedLen, TypeInfo)]
pub struct BaseFilter;
impl Contains<Call> for BaseFilter {
    fn contains(call: &Call) -> bool {
        use frame_support::dispatch::GetCallMetadata;

        let metadata = call.get_call_metadata();
        !XSystem::is_paused(metadata)
    }
}
/*
这段代码是 ChainX 区块链运行时配置的一部分,它涉及到 BABE 共识机制的创世(genesis)配置和基础调用过滤器(BaseFilter)的定义.

### BABE 创世配置(BABE Genesis Configuration)

`BABE_GENESIS_EPOCH_CONFIG` 是 BABE 共识算法在区块链创世时的配置.BABE(Block-Aware Bayesian Elections)是
 Substrate 框架中实现的一种随机区块产生算法,用于选择区块生产者(也称为出块节点).

- `c`: 这是 BABE 算法中的一个参数,用于设置区块产生的概率.`PRIMARY_PROBABILITY` 是一个之前定义的常量,它决定了在 BABE 算法中,一个节点被选为出块节点的概率.
- `allowed_slots`: 这个参数定义了哪些类型的插槽(slots)是被允许的.`PrimaryAndSecondaryPlainSlots` 表示
允许主插槽(primary slots)和辅助插槽(secondary slots),这是一种混合共识机制,结合了 BABE 和其他共识算法(如 Grandpa)的特点.

注释中提到,现有的链正在运行 `PrimaryAndSecondaryPlainSlots` 配置,因此在 `BabeApi::configuration()` 方法中应该返回相同的配置.
注释还提到,可以在创世配置中更改这个值,因为它只在创世时使用,不会影响已经存在的链.但是,为了清晰起见,通常建议保持原始值.

### 基础调用过滤器(BaseFilter)

`BaseFilter` 是一个结构体,它实现了 `Contains` trait,用于过滤不需要的调用(calls).在 Substrate 框架中,调用过滤器用于决定哪些交易调用是允许的,哪些是禁止的.

- `contains` 函数检查一个给定的调用(`Call`)是否应该被包含.这里,它通过获取调用的元数据(`GetCallMetadata`)并检查是否
被 `XSystem::is_paused(metadata)` 函数标记为暂停.如果调用没有被标记为暂停,那么 `contains` 函数返回 `true`,表示该调用应该被包含.

这个过滤器可以用于实现一些安全策略,例如,在系统升级或维护期间暂停所有交易调用,或者只允许某些特定的调用执行.

总的来说,这段代码展示了如何配置 BABE 共识机制的创世参数,以及如何定义一个基础的调用过滤器来控制区块链上允许执行的交易调用.这些配置对于区块链的安全性和治理至关重要.

*/

pub const FORBIDDEN_CALL: u8 = 255;
pub const FORBIDDEN_ACCOUNT: u8 = 254;

impl SignedExtension for BaseFilter {
    const IDENTIFIER: &'static str = "BaseFilter";
    type AccountId = AccountId;
    type Call = Call;
    type AdditionalSigned = ();
    type Pre = ();
    fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
        Ok(())
    }

    fn pre_dispatch(
        self,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        self.validate(who, call, info, len).map(|_| ())
    }

    fn validate(
        &self,
        who: &Self::AccountId,
        call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> TransactionValidity {
        if !Self::contains(call) {
            return Err(InvalidTransaction::Custom(FORBIDDEN_CALL).into());
        }
        if XSystem::blacklist(who) {
            return Err(InvalidTransaction::Custom(FORBIDDEN_ACCOUNT).into());
        }
        Ok(ValidTransaction::default())
    }
}

/*
这段代码定义了一个名为 `BaseFilter` 的结构体,并为其实现了 `SignedExtension` trait.
`SignedExtension` 是 Substrate 框架中的一个 trait,它允许在交易签名时附加额外的信息,这些信息不会影响交易的签名验证,但可以在交易分发之前进行额外的检查.

`BaseFilter` 用作一个交易过滤器,它决定哪些交易可以进入区块链网络,哪些应该被拒绝.这是通过实现 `validate` 函数来完成的,该函数会在交易被执行之前被调用.

### 常量定义

- `FORBIDDEN_CALL`: 一个自定义的错误代码,用于表示交易调用被禁止.
- `FORBIDDEN_ACCOUNT`: 另一个自定义的错误代码,用于表示交易发起者被禁止.

### `SignedExtension` 实现

- `IDENTIFIER`: 一个静态字符串,用于标识 `BaseFilter` 扩展.
- `AccountId`: 交易发起者的账户 ID 类型.
- `Call`: 区块链上的调用或交易类型.
- `AdditionalSigned`: 额外签名的数据类型,这里为空元组 `()` 表示没有额外签名数据.
- `Pre`: 预分发阶段的输出类型,这里为空元组 `()` 表示没有预分发输出.

### 方法实现

- `additional_signed`: 一个返回 `Result<(), TransactionValidityError>` 的函数,用于处理额外签名的数据.
在 `BaseFilter` 的情况下,没有额外签名的数据,因此它返回 `Ok(())`.

- `pre_dispatch`: 在交易分发之前调用的函数,它使用 `validate` 函数来验证交易,并映射结果到 `Self::Pre` 类型.
如果验证失败,它将返回一个 `TransactionValidityError`.

- `validate`: 这是 `BaseFilter` 核心的验证逻辑.它检查交易调用是否被 `contains` 方法接受,以及交易发起者是否在黑名单中.
如果交易调用被禁止或发起者账户被禁止,它将返回一个错误交易有效性(`InvalidTransaction::Custom`).
如果一切正常,它将返回一个有效的交易有效性(`ValidTransaction::default()`).

通过这种方式,`BaseFilter` 可以作为一个安全机制,防止不受欢迎的交易进入区块链网络,并确保只有经过验证的账户才能发起交易.这对于保护区块链免受恶意交易和攻击非常重要.

*/

const AVERAGE_ON_INITIALIZE_WEIGHT: Perbill = Perbill::from_percent(10);
parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    /// We allow for 2 seconds of compute with a 6 second average block time.
    pub const MaximumBlockWeight: Weight = 2 * WEIGHT_PER_SECOND;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    /// Assume 10% of weight for average on_initialize calls.
    pub MaximumExtrinsicWeight: Weight =
        AvailableBlockRatio::get().saturating_sub(AVERAGE_ON_INITIALIZE_WEIGHT)
        * MaximumBlockWeight::get();
    pub const MaximumBlockLength: u32 = 5 * 1024 * 1024;
    pub const Version: RuntimeVersion = VERSION;
    pub const SS58Prefix: u8 = xp_protocol::MAINNET_ADDRESS_FORMAT_ID;
}

const_assert!(
    AvailableBlockRatio::get().deconstruct() >= AVERAGE_ON_INITIALIZE_WEIGHT.deconstruct()
);

impl frame_system::Config for Runtime {
    type BaseCallFilter = BaseFilter;
    type BlockWeights = BlockWeights;
    type BlockLength = BlockLength;
    /// The ubiquitous origin type.
    type Origin = Origin;
    /// The aggregated dispatch type that is available for extrinsics.
    type Call = Call;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = Index;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = Indices;
    /// The header type.
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// The ubiquitous event type.
    type Event = Event;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// The weight of database operations that the runtime can invoke.
    type DbWeight = RocksDbWeight;
    /// Version of the runtime.
    type Version = Version;
    /// Converts a module to the index of the module in `construct_runtime!`.
    ///
    /// This type is being generated by `construct_runtime!`.
    type PalletInfo = PalletInfo;
    /// The data to be stored in an account.
    type AccountData = pallet_balances::AccountData<Balance>;
    /// What to do if a new account is created.
    type OnNewAccount = ();
    /// What to do if an account is fully reaped from the system.
    type OnKilledAccount = ();
    /// Weight information for the extrinsics of this pallet.
    type SystemWeightInfo = frame_system::weights::SubstrateWeight<Runtime>;
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

/*
这段代码是 Substrate 框架中 ChainX 区块链的运行时配置的一部分,它定义了一些关键的参数类型和常量,
以及 `frame_system` 模块的配置.让我们逐一解释这些组件:

### 常量定义

- `AVERAGE_ON_INITIALIZE_WEIGHT`: 一个 `Perbill` 类型,表示 `on_initialize` 调用平均占用的区块权重的百分比.这里设置为 10%.
- `BlockHashCount`: 一个 `BlockNumber` 类型,定义了区块链保留块哈希的数目.这里设置为 2400.
- `MaximumBlockWeight`: 一个 `Weight` 类型,定义了区块的最大权重.这里设置为 `WEIGHT_PER_SECOND` 的两倍,
假设每个区块平均时间为 6 秒,允许 2 秒的计算时间.
- `AvailableBlockRatio`: 一个 `Perbill` 类型,表示可用于交易和其他操作的区块权重的百分比.这里设置为 75%.
- `MaximumExtrinsicWeight`: 一个 `Weight` 类型,定义了单个交易(extrinsic)的最大权重.
它是根据 `AvailableBlockRatio` 减去 `AVERAGE_ON_INITIALIZE_WEIGHT` 后,乘以 `MaximumBlockWeight` 计算得出的.
- `MaximumBlockLength`: 一个 `u32` 类型,定义了区块的最大长度(以字节为单位).这里设置为 5MB.
- `Version`: 一个 `RuntimeVersion` 类型,包含了运行时的版本信息.
- `SS58Prefix`: 一个 `u8` 类型,定义了地址格式的 SS58 前缀.这里使用了 `xp_protocol::MAINNET_ADDRESS_FORMAT_ID`,这是一个特定于 ChainX 的前缀.

### `frame_system::Config` 实现

`frame_system::Config` trait 为 Substrate 框架的系统模块提供了配置.这段代码为 `Runtime` 类型实现了这个 trait,定义了运行时的各种类型和行为:

- `BaseCallFilter`: 用于基础调用过滤的类型,这里设置为之前定义的 `BaseFilter`.
- `BlockWeights`: 用于管理区块权重的类型,这里设置为 `BlockWeights`.
- `BlockLength`: 区块长度的类型,这里设置为 `BlockLength`.
- `Origin`: 交易来源的类型.
- `Call`: 可用于交易的调用集合的类型.
- `Index`: 账户签署的交易索引类型.
- `BlockNumber`: 区块编号的类型.
- `Hash`: 用于区块和 trie 的哈希类型.
- `Hashing`: 用于哈希计算的算法类型.
- `AccountId`: 账户 ID 的类型.
- `Lookup`: 从不同类型的来源获取账户 ID 的查找机制.
- `Header`: 区块头部的类型.
- `Event`: 事件记录的类型.
- `BlockHashCount`: 保留的块哈希数量.
- `DbWeight`: 运行时可以调用的数据库操作的权重类型.
- `PalletInfo`: 用于 `construct_runtime!` 宏的模块索引信息.
- `AccountData`: 存储在账户中的账户数据类型.
- `OnNewAccount`, `OnKilledAccount`: 分别用于处理新创建账户和被完全移除账户的逻辑.
- `SystemWeightInfo`: 系统交易权重信息的类型.
- `MaxConsumers`: 最大消费者数量.

这些配置确保了 ChainX 区块链的系统模块能够按照预定的规则和参数运行,包括交易处理,区块构建,事件记录等关键功能.
通过这些配置,ChainX 能够维护其安全性,效率和治理结构.

------------------------------------------------------------------------------------------------------------
区块链保留块哈希的数目(`BlockHashCount`)是一个重要的配置参数,它定义了区块链节点需要保留的最近区块哈希的数量.这个参数有以下几个主要用途:

1. **状态回溯**:保留区块哈希允许节点快速回溯历史状态,这对于验证区块和交易的有效性至关重要.
即使在没有完整的历史数据的情况下,节点也可以通过这些哈希值来验证新区块的合法性.

2. **链的连续性**:区块链依赖于区块之间的哈希指针来保持连续性.每个新区块都包含前一个区块的哈希值,这样可以帮助节点验证链的完整性和一致性.

3. **安全性**:通过保留一定数量的区块哈希,节点可以检测和防止重放攻击.如果一个区块或交易试图被重新添加到链上,
节点可以通过比较哈希值来识别并拒绝这种尝试.

4. **性能优化**:节点不需要无限期地保留所有区块的完整数据.一旦区块被确认并且其状态已经最终确定,
就可以将其从内存中移除,只保留必要的哈希值.这样可以减少存储需求,提高节点的运行效率.

5. **同步和恢复**:当新节点加入网络或现有节点重新启动时,它们需要下载和验证区块链的历史数据.
保留一定数量的区块哈希可以帮助这些节点更快地同步到网络状态.

在 Substrate 框架中,`BlockHashCount` 参数通常设置为一个足够大的数值,以确保节点能够处理可能的状态变化,
同时也不会消耗过多的存储资源.这个参数的值需要在安全性,性能和存储需求之间做出权衡.
*/

parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = Babe;
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = pallet_timestamp::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const IndexDeposit: Balance = 10 * DOLLARS;
}

impl pallet_indices::Config for Runtime {
    type AccountIndex = AccountIndex;
    type Currency = Balances;
    type Deposit = IndexDeposit;
    type Event = Event;
    type WeightInfo = pallet_indices::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const MaxAuthorities: u32 = 10_000;
}
impl pallet_authority_discovery::Config for Runtime {
    type MaxAuthorities = MaxAuthorities;
}

parameter_types! {
    pub const UncleGenerations: BlockNumber = 0;
}

impl pallet_authorship::Config for Runtime {
    type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
    type UncleGenerations = UncleGenerations;
    type FilterUncle = ();
    type EventHandler = ImOnline;
}

parameter_types! {
    pub const EpochDuration: u64 = EPOCH_DURATION_IN_BLOCKS as u64;
    pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
}

pub struct ReportLongevity;

impl Get<u64> for ReportLongevity {
    fn get() -> u64 {
        xpallet_mining_staking::BondingDuration::<Runtime>::get() as u64
            * xpallet_mining_staking::SessionsPerEra::<Runtime>::get() as u64
            * EpochDuration::get()
    }
}

impl pallet_babe::Config for Runtime {
    type EpochDuration = EpochDuration;
    type ExpectedBlockTime = ExpectedBlockTime;
    type EpochChangeTrigger = pallet_babe::ExternalTrigger;

    type DisabledValidators = Session;

    type KeyOwnerProof = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
        KeyTypeId,
        pallet_babe::AuthorityId,
    )>>::Proof;

    type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
        KeyTypeId,
        pallet_babe::AuthorityId,
    )>>::IdentificationTuple;

    type KeyOwnerProofSystem = Historical;

    type HandleEquivocation =
        pallet_babe::EquivocationHandler<Self::KeyOwnerIdentification, Offences, ReportLongevity>;
    type WeightInfo = ();
    type MaxAuthorities = MaxAuthorities;
}

impl pallet_grandpa::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type KeyOwnerProof =
        <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;
    type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
        KeyTypeId,
        GrandpaId,
    )>>::IdentificationTuple;
    type KeyOwnerProofSystem = Historical;
    type HandleEquivocation = pallet_grandpa::EquivocationHandler<
        Self::KeyOwnerIdentification,
        Offences,
        ReportLongevity,
    >;
    type WeightInfo = ();
    type MaxAuthorities = MaxAuthorities;
}

/*
这段代码是 Substrate 框架中 ChainX 区块链的运行时配置的一部分,它涉及到 BABE 共识机制和 GRANDPA 最终性保证的配置.

### BABE 共识配置 (`pallet_babe::Config`)

BABE(Block-Aware Bayesian Elections)是 Substrate 框架中实现的一种随机区块产生算法,用于选择区块生产者(也称为出块节点).

- `EpochDuration`: 定义了一个 epoch 的持续时间,即一组出块节点的轮换周期.
- `ExpectedBlockTime`: 期望的区块产生时间.
- `EpochChangeTrigger`: 触发 epoch 更换的机制,这里使用 `pallet_babe::ExternalTrigger`,允许外部事件触发 epoch 更换.
- `DisabledValidators`: 被禁用的验证者集合,这里使用 `Session` 模块来管理.
- `KeyOwnerProof`: 验证者身份的证明类型.
- `KeyOwnerIdentification`: 验证者身份的识别元组类型.
- `KeyOwnerProofSystem`: 用于验证者身份的证明系统,这里使用 `Historical`,意味着使用历史记录来验证.
- `HandleEquivocation`: 处理不一致投票(equivocation)的处理器.
- `WeightInfo`: 用于配置权重信息的类型,这里为空(`()`).
- `MaxAuthorities`: 允许的最大验证者数量.

### GRANDPA 最终性保证配置 (`pallet_grandpa::Config`)

GRANDPA(GHOST-based Recursive Ancestor Deriving Polling Algorithm)是 Substrate 框架中实现的一种最终性保证算法,
用于确定哪些区块已经被最终确定并不可更改.

- `Event`: 区块链上的事件类型.
- `Call`: 可用于交易的调用集合的类型.
- `KeyOwnerProof` 和 `KeyOwnerIdentification`: 类似于 BABE 配置,用于验证者身份的证明和识别.
- `KeyOwnerProofSystem`: 同样使用 `Historical` 系统.
- `HandleEquivocation`: 处理不一致投票的处理器,与 BABE 配置相同.
- `WeightInfo`: 同样为空(`()`).
- `MaxAuthorities`: 与 BABE 配置共享相同的最大验证者数量.

### 会话密钥配置 (`SessionKeys`)

`impl_opaque_keys!` 宏用于定义一组密钥,这些密钥在运行时用于不同的目的.

- `SessionKeys` 结构体定义了不同类型的密钥:
  - `babe`: BABE 共识机制使用的密钥.
  - `grandpa`: GRANDPA 最终性保证算法使用的密钥.
  - `im_online`: 在线验证者集合使用的密钥.
  - `authority_discovery`: 用于权威发现的密钥.

这些配置确保了 ChainX 区块链的共识机制和最终性保证算法能够按照预定的规则和参数运行,这对于维护区块链的安全性和去中心化至关重要.
通过这些配置,ChainX 能够实现有效的区块生产和最终确定性,同时保持网络的稳定性和可靠性.

*/

parameter_types! {
    pub const Offset: BlockNumber = 0;
    pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
}

impl_opaque_keys! {
    pub struct SessionKeys {
        pub babe: Babe,
        pub grandpa: Grandpa,
        pub im_online: ImOnline,
        pub authority_discovery: AuthorityDiscovery,
    }
}

/// Substrate has the controller/stash concept, the according `Convert`
/// implementation is used to find the stash of the given controller
/// account. There is no such concept in the context of ChainX, the
/// _stash_ account is also the _controller_ account.
pub struct SimpleValidatorIdConverter;

impl Convert<AccountId, Option<AccountId>> for SimpleValidatorIdConverter {
    fn convert(controller: AccountId) -> Option<AccountId> {
        Some(controller)
    }
}

impl pallet_session::Config for Runtime {
    type Event = Event;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    type ValidatorIdOf = SimpleValidatorIdConverter;
    type ShouldEndSession = Babe;
    type NextSessionRotation = Babe;
    // We do not make use of the historical feature of pallet-session, hereby use XStaking only.
    type SessionManager = XStaking;
    type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
    type Keys = SessionKeys;
    type WeightInfo = pallet_session::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    /// No dusty accounts in ChainX.
    pub const ExistentialDeposit: Balance = 0;
    // For weight estimation, we assume that the most locks on an individual account will be 50.
    // This number may need to be adjusted in the future if this assumption no longer holds true.
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
}

parameter_types! {
    pub const TransactionByteFee: Balance = 10 * MILLICENTS; // 100 => 0.000001 pcx
    pub const OperationalFeeMultiplier: u8 = 5;
}

impl pallet_transaction_payment::Config for Runtime {
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, DealWithFees>;
    type TransactionByteFee = TransactionByteFee;
    type OperationalFeeMultiplier = OperationalFeeMultiplier;
    type WeightToFee = self::constants::fee::WeightToFee;
    type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
}

impl xpallet_transaction_fee::Config for Runtime {
    type Event = Event;
}

parameter_types! {
    pub const SessionDuration: BlockNumber = EPOCH_DURATION_IN_BLOCKS;
    pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::MAX;
    /// We prioritize im-online heartbeats over election solution submission.
    pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::MAX / 2;
    pub const MaxKeys: u32 = 10_000;
    pub const MaxPeerInHeartbeats: u32 = 10_000;
    pub const MaxPeerDataEncodingSize: u32 = 1_000;
}

impl pallet_im_online::Config for Runtime {
    type AuthorityId = ImOnlineId;
    type Event = Event;
    // FIXME: replace Session using Staking
    type ValidatorSet = Self;
    type NextSessionRotation = Babe;
    type ReportUnresponsiveness = Offences;
    type UnsignedPriority = ImOnlineUnsignedPriority;
    type WeightInfo = pallet_im_online::weights::SubstrateWeight<Runtime>;
    type MaxKeys = MaxKeys;
    type MaxPeerInHeartbeats = MaxPeerInHeartbeats;
    type MaxPeerDataEncodingSize = MaxPeerDataEncodingSize;
}

impl frame_support::traits::ValidatorSet<AccountId> for Runtime {
    type ValidatorId = AccountId;
    type ValidatorIdOf = SimpleValidatorIdConverter;

    fn session_index() -> SessionIndex {
        Session::current_index()
    }

    fn validators() -> Vec<Self::ValidatorId> {
        Session::validators()
    }
}

impl frame_support::traits::ValidatorSetWithIdentification<AccountId> for Runtime {
    type Identification = AccountId;
    type IdentificationOf = SimpleValidatorIdConverter;
}

/// Dummy implementation for the trait bound of pallet_im_online.
/// We actually make no use of the historical feature of pallet_session.
impl pallet_session_historical::Config for Runtime {
    type FullIdentification = AccountId;
    /// Substrate: given the stash account ID, find the active exposure of nominators on that account.
    /// ChainX: the full identity is always the validator account itself.
    type FullIdentificationOf = SimpleValidatorIdConverter;
}

/*
这段代码继续定义了 ChainX 区块链运行时的几个关键模块的配置.主要包括会话管理 (`pallet_session`),
余额管理 (`pallet_balances`),交易费用 (`pallet_transaction_payment` 和 `xpallet_transaction_fee`) 以及在线验证者管理 (`pallet_im_online`).

### 会话管理 (`pallet_session`)

- `ValidatorId`: 验证者的账户 ID 类型.
- `ValidatorIdOf`: 用于将控制器账户 ID 转换为提名者 ID 的转换器,这里使用 `SimpleValidatorIdConverter`,它简单地将控制器 ID 作为提名者 ID.
- `ShouldEndSession`: 决定会话是否应该结束的逻辑,这里使用 BABE 共识机制.
- `NextSessionRotation`: 下一次会话轮换的逻辑,同样使用 BABE.
- `SessionManager`: 会话管理器,这里使用 `XStaking`.
- `SessionHandler`: 会话密钥的类型提供者.
- `Keys`: 会话密钥的结构体.

### 余额管理 (`pallet_balances`)

- `ExistentialDeposit`: 账户存在所需的最小余额,这里设置为 0,意味着在 ChainX 上创建账户不需要存款.
- `MaxLocks`: 单个账户上最大的锁定数量.
- `MaxReserves`: 单个账户上最大的储备数量.
- `ReserveIdentifier`: 用于标识账户锁定的保留标识符.

### 交易费用 (`pallet_transaction_payment` 和 `xpallet_transaction_fee`)

- `TransactionByteFee`: 每字节交易费用的基准值.
- `OperationalFeeMultiplier`: 操作费用的乘数,用于调整交易费用.
- `WeightToFee`: 将交易权重转换为费用的逻辑.
- `FeeMultiplierUpdate`: 费用乘数更新逻辑.

### 在线验证者管理 (`pallet_im_online`)

- `ValidatorSet`: 用于确定当前验证者集合的逻辑,这里直接使用 `Self`.
- `NextSessionRotation`: 下一次会话轮换的逻辑,使用 BABE.
- `ReportUnresponsiveness`: 报告不响应的逻辑,这里使用 `Offences`.
- `UnsignedPriority`: 未签名交易的优先级.

### 验证者集合管理

- `ValidatorId`: 验证者的账户 ID 类型.
- `ValidatorIdOf`: 用于获取验证者 ID 的转换器.
- `session_index` 和 `validators`: 分别用于获取当前会话索引和验证者列表.

### 历史会话管理

- `FullIdentificationOf`: 用于获取完整身份标识符的转换器,这里使用 `SimpleValidatorIdConverter`,它将验证者账户 ID 作为完整身份.

这些配置确保了 ChainX 区块链的会话管理,余额管理,交易费用和在线验证者管理等功能能够按照预定的规则和参数运行.
这些模块的配置对于区块链的安全性,治理和经济模型至关重要.

------------------------------------------------------------------------------------------------------------------------
在 Substrate 框架和许多其他区块链系统中,控制器账户(Controller Account)是一个特殊的账户,
它负责管理和控制另一个被称为提名者(Nominator)或质押账户(Stash Account)的账户.
这种设计允许用户将其代币委托给其他用户(即控制器),以便参与网络的共识过程并获得奖励.

控制器账户的主要功能包括:

1. **交易签名**:控制器账户负责签署交易,包括出块和投票等.

2. **共识参与**:在某些共识机制中,如 BABE 或 GRANDPA,控制器账户可能被选为出块节点或验证者,负责创建新区块和验证交易.

3. **奖励分配**:当控制器账户成功出块或验证交易时,它会获得奖励,这些奖励可以按照预设的比例分配给提名者账户和控制器账户.

4. **治理参与**:控制器账户可以代表提名者账户参与链上治理,例如投票决定提案.

在 Substrate 中,控制器账户和提名者账户是分开的,但在 ChainX 的上下文中,如代码注释所述,没有这样的概念,即提名者账户也是控制器账户.
这意味着用户直接控制自己的账户进行交易签名和共识参与,而不是通过一个单独的控制器账户.

这种设计简化了账户管理,使得用户不需要担心控制器和提名者账户之间的分离和通信.用户可以直接使用自己的账户来参与网络的各种活动,包括质押,投票和交易.

------------------------------------------------------------------------------------------------------------------------
在 Substrate 框架中,`ShouldEndSession` 是 `pallet_session` 模块的一个配置项,它定义了一个逻辑,
用于决定是否应该结束当前的会话(session)并开始一个新的会话.会话是区块链中的一个重要概念,特别是在涉及共识机制和验证者轮换时.

### 会话(Session)的概念:

在 Substrate 中,会话是指一组验证者的集合,他们在一定的时间周期内负责区块的生产和网络的维护.这个时间周期被称为一个"epoch"或"session".
在一个会话期间,验证者们会轮流出块,并对网络事务进行验证和投票.会话结束后,可能会有新的验证者被选举进入下一个会话.

### `ShouldEndSession` 的使用场景:

`ShouldEndSession` 通常在以下场景中被调用:

1. **Epoch 更换**:当当前会话的时间周期结束时,`ShouldEndSession` 会被用来确定是否应该开始一个新的会话.这通常与区块链的时间逻辑相关,
例如,每个 epoch 可能对应一定的区块数量或固定的时间长度.

2. **验证者轮换**:在一个会话结束时,可能需要根据新的投票结果或质押情况来选择新的验证者集合.`ShouldEndSession` 有助于确保这个过程顺利进行.

3. **共识机制变更**:如果区块链的共识机制需要变更,`ShouldEndSession` 可以作为一个触发点来执行必要的状态转换.

4. **紧急情况**:在某些紧急情况下,如网络攻击或不可预见的错误,可能需要提前结束当前会话并开始一个新的会话来恢复网络的正常运行.

在 ChainX 的运行时配置中,`ShouldEndSession` 被设置为 `Babe`,这意味着会话结束的逻辑是由 BABE 共识机制来决定的.
BABE 会根据当前的 epoch 持续时间和区块生产情况来决定是否结束当前会话.这种设计使得会话管理与共识机制紧密集成,确保了区块链的稳定性和去中心化.


*/

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
    Call: From<LocalCall>,
{
    fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: Call,
        public: <Signature as traits::Verify>::Signer,
        account: AccountId,
        nonce: Index,
    ) -> Option<(
        Call,
        <UncheckedExtrinsic as traits::Extrinsic>::SignaturePayload,
    )> {
        // take the biggest period possible.
        let period = BlockHashCount::get()
            .checked_next_power_of_two()
            .map(|c| c / 2)
            .unwrap_or(2) as u64;
        /*
        这段代码的目的是计算一个尽可能大的时期(epoch)长度,这个长度是以区块数量来衡量的.
        在 Substrate 框架中,`BlockHashCount` 表示区块链会保留多少个最近的区块哈希值.
        这个值通常被设置为 2 的幂,以便在数据结构中高效地使用.

        让我们逐步分析代码:

        1. `BlockHashCount::get()`:获取当前配置的 `BlockHashCount` 值,即保留的区块哈希数量.

        2. `.checked_next_power_of_two()`:这个方法尝试找到大于或等于当前 `BlockHashCount` 值的下一个 2 的幂.
        这是为了确保计算出的时期长度是 2 的幂,这样可以更好地适应 Substrate 框架的要求.

        3. `.map(|c| c / 2)`:由于 `checked_next_power_of_two()` 返回的值是下一个 2 的幂,
        这里将其除以 2 来得到一个较大的时期长度.这样做的原因是,如果 `BlockHashCount` 已经是 2 的幂,
        那么下一个 2 的幂可能会太大,导致不必要的存储开销.通过除以 2,我们可以得到一个仍然较大的时期长度,但不会过大.

        4. `.unwrap_or(2)`:如果 `checked_next_power_of_two()` 方法返回 `None`
        (例如,当 `BlockHashCount` 已经是 2 的最大幂时),则使用 2 作为默认的时期长度.

        5. `as u64`:将结果转换为 `u64` 类型,以便后续处理.

        为什么要尽可能选择最大的时期长度呢?这主要是为了确保区块链有足够的时间来处理和最终确定区块.
        在 Substrate 中,BABE 共识机制使用时期(epoch)来安排区块的生产.如果时期长度太短,
        可能会导致频繁的验证者轮换和状态转换,这可能会影响区块链的性能和稳定性.
        通过选择一个较大的时期长度,可以减少这些开销,同时仍然保持区块链的安全性和去中心化特性.
        */
        let current_block = System::block_number()
            .saturated_into::<u64>()
            // The `System::block_number` is initialized with `n+1`,
            // so the actual block number is `n`.
            .saturating_sub(1);
        let tip = 0;
        let extra: SignedExtra = (
            frame_system::CheckNonZeroSender::<Runtime>::new(),
            frame_system::CheckSpecVersion::<Runtime>::new(),
            frame_system::CheckTxVersion::<Runtime>::new(),
            frame_system::CheckGenesis::<Runtime>::new(),
            frame_system::CheckEra::<Runtime>::from(generic::Era::mortal(period, current_block)),
            frame_system::CheckNonce::<Runtime>::from(nonce),
            frame_system::CheckWeight::<Runtime>::new(),
            pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
            BaseFilter,
            ChargeExtraFee,
        );
        let raw_payload = SignedPayload::new(call, extra)
            .map_err(|e| {
                frame_support::log::warn!("Unable to create signed payload: {:?}", e);
            })
            .ok()?;
        let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
        let address = Indices::unlookup(account);
        let (call, extra, _) = raw_payload.deconstruct();
        Some((call, (address, signature, extra)))
    }
}

impl frame_system::offchain::SigningTypes for Runtime {
    type Public = <Signature as traits::Verify>::Signer;
    type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
    Call: From<C>,
{
    type Extrinsic = UncheckedExtrinsic;
    type OverarchingCall = Call;
}

/*
这段代码是 Substrate 框架中 `frame_system` 模块的一部分,它定义了 ChainX 区块链的离线(offchain)交易创建和签名逻辑.
这些特性允许在区块链外部创建和签名交易,然后将它们发送到链上.

### `CreateSignedTransaction` 实现

`frame_system::offchain::CreateSignedTransaction` trait 用于创建一个已签名的交易.`Runtime` 类型实现了这个 trait,
以便可以创建符合 ChainX 区块链规则的交易.

- `create_transaction` 函数接受以下参数:
  - `call`: 要执行的调用(`Call` 类型).
  - `public`: 交易发送者的公钥(`Signer` 类型).
  - `account`: 发送者的账户 ID.
  - `nonce`: 发送者的交易计数器(`Index` 类型).

函数执行以下步骤:
1. 计算当前区块号和期望的时期长度.
2. 创建一个 `SignedExtra` 元组,包含一系列用于验证交易的检查,例如非零发送者,规格版本,交易版本,创世区块,时代,交易权重等.
3. 使用 `SignedPayload` 创建一个未检查的负载(`raw_payload`),它包含调用和额外的签名信息.
4. 使用提供的公钥对负载进行签名.
5. 从账户 ID 中获取地址.
6. 解构 `raw_payload` 以获取调用,签名和额外信息.
7. 返回一个包含调用,签名和地址的 `Some` 选项.

### `SigningTypes` 实现

`frame_system::offchain::SigningTypes` trait 用于定义区块链使用的公钥和签名类型.

- `Public` 类型是用于验证签名的公钥类型.
- `Signature` 类型是签名本身.

### `SendTransactionTypes` 实现

`frame_system::offchain::SendTransactionTypes` trait 用于定义如何将交易发送到区块链.

- `Extrinsic` 类型是交易的类型,这里使用 `UncheckedExtrinsic`,表示尚未检查的交易.
- `OverarchingCall` 类型是交易中包含的调用的类型,这里直接使用 `Call`.

这些实现使得 ChainX 区块链能够支持创建,签名和发送交易,这是区块链与外部世界交互的基础.
通过这些机制,用户可以创建交易,对其进行签名,然后将它们发送到链上进行验证和执行.

----------------------------------------------------------------------------------------
在区块链的上下文中,"时期长度"(Epoch Duration)通常指的是一个时间周期,在此期间内,
一组特定的验证者或节点负责网络的共识任务,如区块的创建和交易的验证.
这个概念在许多共识机制中都很重要,尤其是在那些使用轮流或随机选择验证者的系统中.

在 Substrate 框架中,例如 BABE(Block-Aware Bayesian Elections)或 
GRANDPA(GHOST-based Recursive Ancestor Deriving Polling Algorithm)共识机制中,时期长度是一个关键参数,它定义了以下内容:

1. **验证者轮换**:时期长度决定了验证者轮换的频率.在一个时期结束时,系统会根据新的选举或随机选择结果来更换一组验证者.

2. **区块生产**:在 BABE 共识中,时期长度会影响区块的生产时间.验证者在被选中产生区块的时间窗口内尝试创建区块.

3. **最终性**:在 GRANDPA 算法中,时期长度与最终性有关.一旦区块在某个时期内被 GRANDPA 确认为最终区块,它就被认为是不可更改的.

4. **治理和奖励**:时期长度还可能影响治理决策的周期和验证者的奖励分配.

在 ChainX 区块链的配置中,`EpochDuration` 被设置为一个特定的值(例如,`EPOCH_DURATION_IN_BLOCKS`),
这个值表示一个时期包含的区块数量.这个参数对于区块链的稳定性和性能至关重要,因为它影响了网络的响应能力和去中心化程度.
时期长度的选择需要在安全性,效率和网络参与度之间做出权衡.
*/

impl pallet_offences::Config for Runtime {
    type Event = Event;
    type IdentificationTuple = xpallet_mining_staking::IdentificationTuple<Runtime>;
    type OnOffenceHandler = XStaking;
}

impl pallet_utility::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type PalletsOrigin = OriginCaller;
    type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    // One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
    pub const DepositBase: Balance = deposit(1, 88);
    // Additional storage item size of 32 bytes.
    pub const DepositFactor: Balance = deposit(0, 32);
    pub const MaxSignatories: u16 = 100;
}

impl pallet_multisig::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type Currency = Balances;
    type DepositBase = DepositBase;
    type DepositFactor = DepositFactor;
    type MaxSignatories = MaxSignatories;
    type WeightInfo = pallet_multisig::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const LaunchPeriod: BlockNumber = 7 * DAYS;
    pub const VotingPeriod: BlockNumber = 7 * DAYS;
    pub const FastTrackVotingPeriod: BlockNumber = 3 * HOURS;
    pub const InstantAllowed: bool = true;
    // 10 PCX
    pub const MinimumDeposit: Balance = 1000 * DOLLARS;
    pub const EnactmentPeriod: BlockNumber = 8 * DAYS;
    pub const CooloffPeriod: BlockNumber = 7 * DAYS;
    // One cent: $10,000 / MB
    pub const PreimageByteDeposit: Balance = CENTS;
    pub const MaxVotes: u32 = 100;
    pub const MaxProposals: u32 = 100;
}

impl pallet_democracy::Config for Runtime {
    type Proposal = Call;
    type Event = Event;
    type Currency = Balances;
    type EnactmentPeriod = EnactmentPeriod;
    type LaunchPeriod = LaunchPeriod;
    type VotingPeriod = VotingPeriod;
    type VoteLockingPeriod = EnactmentPeriod;
    type MinimumDeposit = MinimumDeposit;
    /// A straight majority of the council can decide what their next motion is.
    type ExternalOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>;
    /// A super-majority can have the next scheduled referendum be a straight majority-carries vote.
    type ExternalMajorityOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 4>;
    /// A unanimous council can have the next scheduled referendum be a straight default-carries
    /// (NTB) vote.
    type ExternalDefaultOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>;
    /// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
    /// be tabled immediately and with a shorter voting/enactment period.
    type FastTrackOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 2, 3>;
    type InstantOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 1, 1>;
    type InstantAllowed = InstantAllowed;
    type FastTrackVotingPeriod = FastTrackVotingPeriod;
    // To cancel a proposal which has been passed, 2/3 of the council must agree to it.
    type CancellationOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>;
    type BlacklistOrigin = EnsureRoot<AccountId>;
    // To cancel a proposal before it has been passed, the technical committee must be unanimous or
    // Root must agree.
    type CancelProposalOrigin = EnsureOneOf<
        pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 1, 1>,
        EnsureRoot<AccountId>,
    >;
    // Any single technical committee member may veto a coming council proposal, however they can
    // only do it once and it lasts only for the cooloff period.
    type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
    type CooloffPeriod = CooloffPeriod;
    type PreimageByteDeposit = PreimageByteDeposit;
    type OperationalPreimageOrigin = pallet_collective::EnsureMember<AccountId, CouncilCollective>;
    type Slash = Treasury;
    type Scheduler = Scheduler;
    type PalletsOrigin = OriginCaller;
    type MaxVotes = MaxVotes;
    type WeightInfo = pallet_democracy::weights::SubstrateWeight<Runtime>;
    type MaxProposals = MaxProposals;
}

parameter_types! {
    pub const CouncilMotionDuration: BlockNumber = 7 * DAYS;
    pub const CouncilMaxProposals: u32 = 100;
    pub const CouncilMaxMembers: u32 = 100;
}

/*
这段代码是 Substrate 框架中 ChainX 区块链运行时配置的一部分,它涉及到多个与治理和多签名相关的模块的配置.

### `pallet_offences` 配置

`pallet_offences` 模块用于处理区块链上的违规行为.配置包括:

- `Event`: 区块链上的事件类型.
- `IdentificationTuple`: 用于识别和处理违规行为的元组类型,这里使用了 `xpallet_mining_staking` 模块的 `IdentificationTuple`.
- `OnOffenceHandler`: 处理违规行为的处理器,这里使用 `XStaking` 模块.

### `pallet_utility` 配置

`pallet_utility` 模块提供了一些辅助功能,如执行无需直接与区块链状态交互的调用.配置包括:

- `Event`: 事件类型.
- `Call`: 调用类型.
- `PalletsOrigin`: 用于追踪调用来源的类型.
- `WeightInfo`: 权重信息,用于衡量操作的计算成本.

### `pallet_multisig` 配置

`pallet_multisig` 模块允许创建需要多个签名者同意的多签名交易.配置包括:

- `Event`: 事件类型.
- `Call`: 调用类型.
- `Currency`: 货币类型.
- `DepositBase` 和 `DepositFactor`: 创建多签名账户时的基础和额外存储押金.
- `MaxSignatories`: 允许的最大签名者数量.
- `WeightInfo`: 权重信息.

### 治理参数配置

定义了与 ChainX 区块链治理相关的参数,如:

- `LaunchPeriod`, `VotingPeriod`, `FastTrackVotingPeriod`: 提案启动,投票和快速通道投票期的区块数量.
- `InstantAllowed`: 是否允许即时提案.
- `MinimumDeposit`: 提交提案所需的最低存款.
- `EnactmentPeriod`, `CooloffPeriod`: 法案实施和冷却期的区块数量.
- `PreimageByteDeposit`: 每个字节预图像(提案哈希)的存款.
- `MaxVotes` 和 `MaxProposals`: 最大投票数和提案数.

### `pallet_democracy` 配置

`pallet_democracy` 模块实现了一个基本的链上治理系统.配置包括:

- `Proposal`: 提案类型.
- `Event`: 事件类型.
- `Currency`: 货币类型.
- `EnactmentPeriod`, `LaunchPeriod`, `VotingPeriod`: 法案实施,启动和投票期的区块数量.
- `MinimumDeposit`: 提交提案所需的最低存款.
- `ExternalOrigin`, `ExternalMajorityOrigin`, `ExternalDefaultOrigin`: 不同类型的提案来源和通过门槛.
- `FastTrackOrigin`, `InstantOrigin`: 快速通道和即时提案的来源.
- `CancellationOrigin`, `BlacklistOrigin`, `CancelProposalOrigin`: 取消提案,黑名单提案和取消未通过提案的来源.
- `VetoOrigin`: 否决提案的来源.
- `CooloffPeriod`: 冷却期的区块数量.
- `PreimageByteDeposit`: 预图像字节存款.
- `Scheduler`: 调度器类型.
- `PalletsOrigin`: 调用来源类型.
- `MaxVotes`, `MaxProposals`: 最大投票数和提案数.
- `WeightInfo`: 权重信息.

### `CouncilMotionDuration` 配置

定义了理事会提案的持续时间,这里设置为 7 天.

### `CouncilMaxProposals` 和 `CouncilMaxMembers` 配置

定义了理事会允许的最大提案数和成员数.

这些配置确保了 ChainX 区块链的治理机制能够按照预定的规则和参数运行,允许社区成员通过投票来决定链上的更改,
如协议升级,参数调整和资金分配等.通过这些配置,ChainX 能够实现有效的去中心化治理,同时保持网络的稳定性和可靠性.

----------------------------------------------------------------------------------------------------
理事会(Council)在区块链治理系统中通常是一个由社区选举产生的代表团体,负责就链上的各种提案和决策进行讨论,审议和投票.
理事会的成员通常是社区中积极参与和有影响力的成员,他们被赋予了代表社区做出决策的权力.

理事会的主要职责和功能包括:

1. **提案提交**:理事会成员可以提交新的提案,这些提案可能涉及协议的更改,资金分配,社区治理规则的更新等.

2. **讨论与审议**:理事会成员就提交的提案进行讨论和审议,确保提案的合理性和可行性.

3. **投票决策**:理事会成员对提案进行投票,根据投票结果决定是否将提案付诸实施.投票可能包括普通投票,快速通道投票或即时投票等不同形式.

4. **社区沟通**:理事会作为社区成员和区块链治理系统之间的桥梁,负责传达社区的意见和需求,并就决策结果向社区进行反馈.

5. **监督执行**:理事会监督提案的执行情况,确保决策得到妥善实施,并根据实施效果进行必要的调整.

6. **维护社区利益**:理事会需要考虑整个社区的长期利益,确保决策符合社区的发展方向和价值观.

在 Substrate 框架中,理事会的功能通常通过 `pallet_collective` 模块实现,该模块提供了一套治理工具,
允许理事会成员提交提案,进行投票和执行决策.理事会的存在使得区块链治理更加民主化和去中心化,允许社区成员通过选举代表来共同管理区块链的未来.
*/

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
    type Origin = Origin;
    type Proposal = Call;
    type Event = Event;
    type MotionDuration = CouncilMotionDuration;
    type MaxProposals = CouncilMaxProposals;
    type MaxMembers = CouncilMaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    // 10 PCX
    pub const CandidacyBond: Balance = 1000 * DOLLARS;
    // 1 storage item created, key size is 32 bytes, value size is 16+16.
    pub const VotingBondBase: Balance = deposit(1, 64);
    // additional data per vote is 32 bytes (account id).
    pub const VotingBondFactor: Balance = deposit(0, 32);
    pub const VotingBond: Balance = DOLLARS;
    pub const TermDuration: BlockNumber = DAYS;
    pub const DesiredMembers: u32 = 11;
    pub const DesiredRunnersUp: u32 = 7;
    pub const ElectionsPhragmenPalletId: LockIdentifier = *b"pcx/phre";
}

// Make sure that there are no more than `MaxMembers` members elected via elections-phragmen.
const_assert!(DesiredMembers::get() <= CouncilMaxMembers::get());

impl pallet_elections_phragmen::Config for Runtime {
    type Event = Event;
    type PalletId = ElectionsPhragmenPalletId;
    type Currency = Balances;
    type ChangeMembers = Council;
    // NOTE: this implies that council's genesis members cannot be set directly and must come from
    // this module.
    type InitializeMembers = Council;
    type CurrencyToVote = frame_support::traits::U128CurrencyToVote;
    type CandidacyBond = CandidacyBond;
    type VotingBondBase = VotingBondBase;
    type VotingBondFactor = VotingBondFactor;
    type LoserCandidate = Treasury;
    type KickedMember = Treasury;
    type DesiredMembers = DesiredMembers;
    type DesiredRunnersUp = DesiredRunnersUp;
    type TermDuration = TermDuration;
    type WeightInfo = pallet_elections_phragmen::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const TechnicalMotionDuration: BlockNumber = 5 * DAYS;
    pub const TechnicalMaxProposals: u32 = 100;
    pub const TechnicalMaxMembers: u32 = 100;
}

type TechnicalCollective = pallet_collective::Instance2;
impl pallet_collective::Config<TechnicalCollective> for Runtime {
    type Origin = Origin;
    type Proposal = Call;
    type Event = Event;
    type MotionDuration = TechnicalMotionDuration;
    type MaxProposals = TechnicalMaxProposals;
    type MaxMembers = TechnicalMaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

type EnsureRootOrHalfCouncil = EnsureOneOf<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
>;
impl pallet_membership::Config<pallet_membership::Instance1> for Runtime {
    type Event = Event;
    type AddOrigin = EnsureRootOrHalfCouncil;
    type RemoveOrigin = EnsureRootOrHalfCouncil;
    type SwapOrigin = EnsureRootOrHalfCouncil;
    type ResetOrigin = EnsureRootOrHalfCouncil;
    type PrimeOrigin = EnsureRootOrHalfCouncil;
    type MembershipInitialized = TechnicalCommittee;
    type MembershipChanged = TechnicalCommittee;
    type MaxMembers = TechnicalMaxMembers;
    type WeightInfo = pallet_membership::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const ProposalBond: Permill = Permill::from_percent(5);
    // 10 PCX
    pub const ProposalBondMinimum: Balance = 1000 * DOLLARS;
    // 100 PCX
    pub const ProposalBondMaximum: Balance = 10000 * DOLLARS;
    pub const SpendPeriod: BlockNumber = 6 * DAYS;
    pub const NoBurn: Permill = Permill::from_percent(0);
    pub const TipCountdown: BlockNumber = DAYS;
    pub const TipFindersFee: Percent = Percent::from_percent(20);
    pub const TipReportDepositBase: Balance = DOLLARS;
    pub const DataDepositPerByte: Balance = CENTS;
    pub const BountyDepositBase: Balance = DOLLARS;
    pub const BountyDepositPayoutDelay: BlockNumber = 4 * DAYS;
    pub const TreasuryPalletId: PalletId = PalletId(*b"pcx/trsy");
    pub const BountyUpdatePeriod: BlockNumber = 90 * DAYS;
    pub const MaximumReasonLength: u32 = 16384;
    pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
    pub const BountyValueMinimum: Balance = 10 * DOLLARS;
    pub const MaxApprovals: u32 = 100;
}

/*
这段代码定义了 ChainX 区块链运行时的几个关键治理和选举模块的配置.
这些模块包括集体决策(`pallet_collective`),_phragmen 选举(`pallet_elections_phragmen`),
技术集体(`pallet_membership`)以及财政库(`pallet_treasury`).下面是每个模块的详细解释:

### `pallet_collective` 配置

`CouncilCollective` 是 `pallet_collective` 模块的一个实例,用于管理理事会(Council)的治理机制.

- `Origin`: 允许发起提案的来源类型.
- `Proposal`: 提案的类型,这里与区块链上的调用类型 `Call` 相关联.
- `Event`: 事件类型.
- `MotionDuration`: 提案持续时间.
- `MaxProposals`: 最大提案数量.
- `MaxMembers`: 理事会的最大成员数.
- `DefaultVote`: 默认投票行为.
- `WeightInfo`: 权重信息,用于衡量操作的计算成本.

### 选举参数配置

定义了与理事会选举相关的参数:

- `CandidacyBond`: 参选保证金.
- `VotingBondBase` 和 `VotingBondFactor`: 投票保证金的基础和乘数.
- `VotingBond`: 投票所需的保证金.
- `TermDuration`: 任期持续时间.
- `DesiredMembers` 和 `DesiredRunnersUp`: 期望的理事会成员和候选人数量.
- `ElectionsPhragmenPalletId`: 选举模块的锁标识符.

### `pallet_elections_phragmen` 配置

`pallet_elections_phragmen` 模块用于管理_phragmen 选举机制.

- `Event`: 事件类型.
- `PalletId`: 选举模块的标识符.
- `Currency`: 货币类型.
- `ChangeMembers`: 理事会变更成员的逻辑.
- `InitializeMembers`: 初始化理事会成员的逻辑.
- 其他参数与选举相关的配置相同.

### 技术集体(`pallet_collective`)配置

定义了技术集体的治理机制,这可能是一个专门负责技术决策的小组.

- `MotionDuration`: 技术提案的持续时间.
- `MaxProposals` 和 `MaxMembers`: 最大提案数量和技术集体的最大成员数.

### `pallet_membership` 配置

`pallet_membership` 模块用于管理集体成员的加入,移除和更换.

- `Origin`: 允许进行成员管理操作的来源类型.
- `AddOrigin`, `RemoveOrigin`, `SwapOrigin`, `ResetOrigin`, `PrimeOrigin`: 
分别用于添加,移除,更换,重置和初始化成员的逻辑.
- `MembershipInitialized` 和 `MembershipChanged`: 成员初始化和变更的事件.

### 财政库参数配置

定义了与财政库相关的治理参数:

- `ProposalBond`: 提案保证金的百分比.
- `ProposalBondMinimum` 和 `ProposalBondMaximum`: 提案保证金的最小和最大值.
- `SpendPeriod`: 提案支出期.
- `NoBurn`: 不燃烧(销毁)代币的比例.
- `TipCountdown`: 小费倒计时期.
- `TipFindersFee`: 小费发现者费用的百分比.
- `TipReportDepositBase`: 举报小费的最低存款.
- `DataDepositPerByte`: 每字节数据的存款.
- `BountyDepositBase` 和 `BountyDepositPayoutDelay`: 赏金存款的基数和支付延迟期.
- `TreasuryPalletId`: 财政库模块的标识符.
- `BountyUpdatePeriod`: 赏金更新期.
- `MaximumReasonLength`: 理由的最大长度.
- `BountyCuratorDeposit`: 赏金策展人存款的百分比.
- `BountyValueMinimum`: 赏金价值的最小值.
- `MaxApprovals`: 最大批准数.

这些配置确保了 ChainX 区块链的治理和选举机制能够按照预定的规则和参数运行,允许社区成员通过选举代表和投票来共同管理区块链的未来.
通过这些配置,ChainX 能够实现有效的去中心化治理,同时保持网络的稳定性和可靠性.
*/


impl pallet_treasury::Config for Runtime {
    type Currency = Balances;
    type ApproveOrigin = EnsureOneOf<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 5>,
    >;
    type RejectOrigin = EnsureOneOf<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>,
    >;
    type Event = Event;
    type OnSlash = Treasury;
    type ProposalBond = ProposalBond;
    type ProposalBondMinimum = ProposalBondMinimum;
    type ProposalBondMaximum = ProposalBondMaximum;
    type SpendPeriod = SpendPeriod;
    type Burn = NoBurn;
    type PalletId = TreasuryPalletId;
    type BurnDestination = ();
    type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
    type SpendFunds = Bounties;
    type MaxApprovals = MaxApprovals;
}

impl pallet_bounties::Config for Runtime {
    type BountyDepositBase = BountyDepositBase;
    type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
    type BountyUpdatePeriod = BountyUpdatePeriod;
    type BountyCuratorDeposit = BountyCuratorDeposit;
    type BountyValueMinimum = BountyValueMinimum;
    type DataDepositPerByte = DataDepositPerByte;
    type Event = Event;
    type MaximumReasonLength = MaximumReasonLength;
    type WeightInfo = pallet_bounties::weights::SubstrateWeight<Runtime>;
    type ChildBountyManager = ();
}

impl pallet_tips::Config for Runtime {
    type Event = Event;
    type MaximumReasonLength = MaximumReasonLength;
    type DataDepositPerByte = DataDepositPerByte;
    type TipCountdown = TipCountdown;
    type TipFindersFee = TipFindersFee;
    type TipReportDepositBase = TipReportDepositBase;
    type Tippers = Elections;
    type WeightInfo = pallet_tips::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * MaximumBlockWeight::get();
    // Retry a scheduled item every 10 blocks (1 minute) until the preimage exists.
    pub const NoPreimagePostponement: Option<u32> = Some(10);
}

impl pallet_scheduler::Config for Runtime {
    type Event = Event;
    type Origin = Origin;
    type PalletsOrigin = OriginCaller;
    type Call = Call;
    type MaximumWeight = MaximumSchedulerWeight;
    type ScheduleOrigin = EnsureRoot<AccountId>;
    type OriginPrivilegeCmp = EqualPrivilegeOnly;
    type MaxScheduledPerBlock = ConstU32<50>;
    type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
    type PreimageProvider = ();
    type NoPreimagePostponement = NoPreimagePostponement;
}

parameter_types! {
    pub const BasicDeposit: Balance = 10 * DOLLARS;       // 258 bytes on-chain
    pub const FieldDeposit: Balance = 250 * CENTS;        // 66 bytes on-chain
    pub const SubAccountDeposit: Balance = 2 * DOLLARS;   // 53 bytes on-chain
    pub const MaxSubAccounts: u32 = 100;
    pub const MaxAdditionalFields: u32 = 100;
    pub const MaxRegistrars: u32 = 20;
}

impl pallet_identity::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type BasicDeposit = BasicDeposit;
    type FieldDeposit = FieldDeposit;
    type SubAccountDeposit = SubAccountDeposit;
    type MaxSubAccounts = MaxSubAccounts;
    type MaxAdditionalFields = MaxAdditionalFields;
    type MaxRegistrars = MaxRegistrars;
    type Slashed = Treasury;
    type ForceOrigin = EnsureRootOrHalfCouncil;
    type RegistrarOrigin = EnsureRootOrHalfCouncil;
    type WeightInfo = pallet_identity::weights::SubstrateWeight<Runtime>;
}

/*

这段代码继续配置 ChainX 区块链运行时的几个治理和实用模块,包括财政库(`pallet_treasury`),
赏金(`pallet_bounties`),小费(`pallet_tips`),调度器(`pallet_scheduler`)和身份(`pallet_identity`).

### `pallet_treasury` 配置

`pallet_treasury` 模块用于管理区块链的财政库,处理预算,拨款和资金分配.

- `Currency`: 区块链使用的货币类型.
- `ApproveOrigin`: 批准提案的来源,需要是 Root 或者超过 3/5 理事会成员的同意.
- `RejectOrigin`: 拒绝提案的来源,需要是 Root 或者超过 1/2 理事会成员的同意.
- `Event`: 事件类型.
- `OnSlash`: 处理 slashing(削减)事件的模块.
- `ProposalBond`, `ProposalBondMinimum`, `ProposalBondMaximum`: 提案保证金的参数.
- `SpendPeriod`: 提案支出期.
- `Burn`: 燃烧(销毁)代币的比例.
- `PalletId`: 财政库模块的标识符.
- `BurnDestination`: 燃烧代币的目的地.
- `WeightInfo`: 权重信息.
- `SpendFunds`: 用于处理资金支出的模块.
- `MaxApprovals`: 最大批准数.

### `pallet_bounties` 配置

`pallet_bounties` 模块用于管理赏金系统,允许用户为有价值的贡献提供资金激励.

- `BountyDepositBase`, `BountyDepositPayoutDelay`: 赏金存款的基数和支付延迟期.
- `BountyUpdatePeriod`: 赏金更新期.
- `BountyCuratorDeposit`: 赏金策展人存款的百分比.
- `BountyValueMinimum`: 赏金价值的最小值.
- `DataDepositPerByte`: 每字节数据的存款.
- `Event`: 事件类型.
- `MaximumReasonLength`: 理由的最大长度.
- `WeightInfo`: 权重信息.
- `ChildBountyManager`: 子赏金管理器.

### `pallet_tips` 配置

`pallet_tips` 模块用于管理小费系统,允许用户为有价值的内容或服务提供小费.

- `Event`: 事件类型.
- `MaximumReasonLength`: 理由的最大长度.
- `DataDepositPerByte`: 每字节数据的存款.
- `TipCountdown`: 小费倒计时期.
- `TipFindersFee`: 小费发现者费用的百分比.
- `TipReportDepositBase`: 举报小费的最低存款.
- `Tippers`: 提供小费者.
- `WeightInfo`: 权重信息.

### `pallet_scheduler` 配置

`pallet_scheduler` 模块用于管理链上调度任务,允许用户安排未来的代码执行.

- `Event`: 事件类型.
- `Origin`: 调度任务的来源.
- `PalletsOrigin`: 调用来源.
- `Call`: 调用类型.
- `MaximumWeight`: 调度任务的最大权重.
- `ScheduleOrigin`: 调度任务的来源.
- `OriginPrivilegeCmp`: 来源权限比较.
- `MaxScheduledPerBlock`: 每个区块中最多可以调度的任务数.
- `WeightInfo`: 权重信息.
- `PreimageProvider`: 预图像提供者.
- `NoPreimagePostponement`: 没有预图像时的延迟重试次数.

### `pallet_identity` 配置

`pallet_identity` 模块用于管理身份信息,允许用户注册和验证他们的身份.

- `Event`: 事件类型.
- `Currency`: 货币类型.
- `BasicDeposit`, `FieldDeposit`, `SubAccountDeposit`: 注册身份,字段和子账户所需的存款.
- `MaxSubAccounts`, `MaxAdditionalFields`, `MaxRegistrars`: 子账户,额外字段和注册员的最大数量.
- `Slashed`: 被削减的资金处理.
- `ForceOrigin`, `RegistrarOrigin`: 强制和注册员的来源.
- `WeightInfo`: 权重信息.

这些配置确保了 ChainX 区块链的治理和实用模块能够按照预定的规则和参数运行,提供了一套完整的工具来管理资金,激励贡献,
调度任务和验证身份.通过这些配置,ChainX 能够实现有效的去中心化治理和社区激励.

*/

parameter_types! {
    // One storage item; key size 32, value size 8; .
    pub const ProxyDepositBase: Balance = deposit(1, 8);
    // Additional storage item size of 33 bytes.
    pub const ProxyDepositFactor: Balance = deposit(0, 33);
    pub const MaxProxies: u16 = 32;
    pub const AnnouncementDepositBase: Balance = deposit(1, 8);
    pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
    pub const MaxPending: u16 = 32;
}

/// The type used to represent the kinds of proxying allowed.
#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Encode,
    Decode,
    RuntimeDebug,
    MaxEncodedLen,
    TypeInfo,
)]
pub enum ProxyType {
    Any = 0,
    NonTransfer = 1,
    Governance = 2,
    Staking = 3,
    IdentityJudgement = 4,
    CancelProxy = 5,
}

impl Default for ProxyType {
    fn default() -> Self {
        Self::Any
    }
}

impl InstanceFilter<Call> for ProxyType {
    fn filter(&self, c: &Call) -> bool {
        match self {
            ProxyType::Any => true,
            ProxyType::NonTransfer => matches!(
                c,
                Call::System(..)
                    | Call::Scheduler(..)
                    | Call::Babe(..)
                    | Call::Timestamp(..)
                    | Call::Indices(pallet_indices::Call::claim{..})
                    | Call::Indices(pallet_indices::Call::free{..})
                    | Call::Indices(pallet_indices::Call::freeze{..})
                    // Specifically omitting Indices `transfer`, `force_transfer`
                    // Specifically omitting the entire Balances pallet
                    | Call::Authorship(..)
                    | Call::XStaking(..)
                    | Call::Session(..)
                    | Call::Grandpa(..)
                    | Call::ImOnline(..)
                    | Call::Democracy(..)
                    | Call::Council(..)
                    | Call::TechnicalCommittee(..)
                    | Call::Elections(..)
                    | Call::TechnicalMembership(..)
                    | Call::Treasury(..)
                    | Call::Utility(..)
                    | Call::Identity(..)
                    | Call::Proxy(..)
                    | Call::Multisig(..)
            ),
            ProxyType::Governance => matches!(
                c,
                Call::Democracy(..)
                    | Call::Council(..)
                    | Call::TechnicalCommittee(..)
                    | Call::Elections(..)
                    | Call::Treasury(..)
                    | Call::Utility(..)
            ),
            ProxyType::Staking => matches!(
                c,
                Call::XStaking(..) | Call::Session(..) | Call::Utility(..)
            ),
            ProxyType::IdentityJudgement => matches!(
                c,
                Call::Identity(pallet_identity::Call::provide_judgement { .. }) | Call::Utility(..)
            ),
            ProxyType::CancelProxy => {
                matches!(
                    c,
                    Call::Proxy(pallet_proxy::Call::reject_announcement { .. })
                )
            }
        }
    }
    fn is_superset(&self, o: &Self) -> bool {
        match (self, o) {
            (x, y) if x == y => true,
            (ProxyType::Any, _) => true,
            (_, ProxyType::Any) => false,
            (ProxyType::NonTransfer, _) => true,
            _ => false,
        }
    }
}

impl pallet_proxy::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type Currency = Balances;
    type ProxyType = ProxyType;
    type ProxyDepositBase = ProxyDepositBase;
    type ProxyDepositFactor = ProxyDepositFactor;
    type MaxProxies = MaxProxies;
    type WeightInfo = pallet_proxy::weights::SubstrateWeight<Runtime>;
    type MaxPending = MaxPending;
    type CallHasher = BlakeTwo256;
    type AnnouncementDepositBase = AnnouncementDepositBase;
    type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

///////////////////////////////////////////
// Chainx pallets
///////////////////////////////////////////
impl xpallet_system::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
}

parameter_types! {
    pub const ChainXAssetId: AssetId = xp_protocol::PCX;
}

impl xpallet_assets_registrar::Config for Runtime {
    type Event = Event;
    type NativeAssetId = ChainXAssetId;
    type RegistrarHandler = XMiningAsset;
    type WeightInfo = xpallet_assets_registrar::weights::SubstrateWeight<Runtime>;
}

impl xpallet_assets::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type TreasuryAccount = SimpleTreasuryAccount;
    type OnCreatedAccount = frame_system::Provider<Runtime>;
    type OnAssetChanged = XMiningAsset;
    type WeightInfo = xpallet_assets::weights::SubstrateWeight<Runtime>;
}

impl xpallet_gateway_records::Config for Runtime {
    type Event = Event;
    type WeightInfo = xpallet_gateway_records::weights::SubstrateWeight<Runtime>;
}

/*
这段代码配置了 ChainX 区块链运行时中的代理(proxy)功能,以及与之相关的几个 ChainX 特有的 pallets.
下面是每个配置的详细解释:

### 代理相关参数配置

- `ProxyDepositBase`: 创建代理关系时的基础存款.
- `ProxyDepositFactor`: 代理关系中每个额外存储项的存款因子.
- `MaxProxies`: 允许的最大代理数量.
- `AnnouncementDepositBase`: 发布代理声明时的基础存款.
- `AnnouncementDepositFactor`: 代理声明中每个额外存储项的存款因子.
- `MaxPending`: 允许的最大挂起代理声明数量.

### 代理类型枚举(`ProxyType`)

定义了允许的代理类型,包括:

- `Any`: 代理任何类型的调用.
- `NonTransfer`: 代理不涉及资产转移的调用.
- `Governance`: 代理治理相关的调用.
- `Staking`: 代理质押相关的调用.
- `IdentityJudgement`: 代理身份判断相关的调用.
- `CancelProxy`: 代理取消代理声明的调用.

### `pallet_proxy` 配置

`pallet_proxy` 模块用于实现代理功能,允许账户将其投票权或执行交易的能力委托给其他账户.

- `Event`: 事件类型.
- `Call`: 调用类型.
- `Currency`: 货币类型.
- `ProxyType`: 代理类型枚举.
- `ProxyDepositBase` 和 `ProxyDepositFactor`: 代理存款的基础和因子.
- `MaxProxies`: 最大代理数量.
- `WeightInfo`: 权重信息.
- `MaxPending`: 最大挂起代理声明数量.
- `CallHasher`: 用于计算调用哈希的算法.
- `AnnouncementDepositBase` 和 `AnnouncementDepositFactor`: 代理声明的存款基础和因子.

### ChainX 特有 pallets 配置

- `xpallet_system`: ChainX 的系统 pallet 配置,定义了事件和货币类型.
- `xpallet_assets_registrar`: 用于资产管理的 pallet 配置,包括事件类型,本地资产 ID,注册处理器和权重信息.
- `xpallet_assets`: 用于资产创建和管理的 pallet 配置,包括事件类型,货币类型,财政账户,账户创建和资产变更处理程序以及权重信息.
- `xpallet_gateway_records`: 用于管理网关记录的 pallet 配置,包括事件类型和权重信息.

这些配置确保了 ChainX 区块链的代理系统和资产管理功能能够按照预定的规则和参数运行,
提供了一套完整的工具来管理代理关系,资产创建和网关记录.

*/

pub struct MultisigProvider;
impl MultisigAddressFor<AccountId> for MultisigProvider {
    fn calc_multisig(who: &[AccountId], threshold: u16) -> AccountId {
        Multisig::multi_account_id(who, threshold)
    }
}

impl xpallet_gateway_common::Config for Runtime {
    type Event = Event;
    type Validator = XStaking;
    type DetermineMultisigAddress = MultisigProvider;
    type CouncilOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>;
    type Bitcoin = XGatewayBitcoin;
    type BitcoinTrustee = XGatewayBitcoin;
    type BitcoinTrusteeSessionProvider = trustees::bitcoin::BtcTrusteeSessionManager<Runtime>;
    type BitcoinTotalSupply = XGatewayBitcoin;
    type BitcoinWithdrawalProposal = XGatewayBitcoin;
    type WeightInfo = xpallet_gateway_common::weights::SubstrateWeight<Runtime>;
}

impl xpallet_gateway_bitcoin::Config for Runtime {
    type Event = Event;
    type UnixTime = Timestamp;
    type CouncilOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>;
    type AccountExtractor = xp_gateway_bitcoin::OpReturnExtractor;
    type TrusteeSessionProvider = trustees::bitcoin::BtcTrusteeSessionManager<Runtime>;
    type TrusteeInfoUpdate = XGatewayCommon;
    type ReferralBinding = XGatewayCommon;
    type AddressBinding = XGatewayCommon;
    type WeightInfo = xpallet_gateway_bitcoin::weights::SubstrateWeight<Runtime>;
}

impl xpallet_dex_spot::Config for Runtime {
    type Event = Event;
    type Price = Balance;
    type WeightInfo = xpallet_dex_spot::weights::SubstrateWeight<Runtime>;
}

pub struct SimpleTreasuryAccount;
impl xpallet_support::traits::TreasuryAccount<AccountId> for SimpleTreasuryAccount {
    fn treasury_account() -> Option<AccountId> {
        Some(TreasuryPalletId::get().into_account())
    }
}

parameter_types! {
    // Total issuance is 7723350PCX by the end of ChainX 1.0.
    // 210000 - (7723350 / 50) = 55533
    pub const MigrationSessionOffset: SessionIndex = 55533;
    pub const MinimumReferralId: u32 = 2;
    pub const MaximumReferralId: u32 = 12;
}

impl xpallet_mining_staking::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type SessionDuration = SessionDuration;
    type MinimumReferralId = MinimumReferralId;
    type MaximumReferralId = MaximumReferralId;
    type SessionInterface = Self;
    type TreasuryAccount = SimpleTreasuryAccount;
    type AssetMining = XMiningAsset;
    type DetermineRewardPotAccount =
        xpallet_mining_staking::SimpleValidatorRewardPotAccountDeterminer<Runtime>;
    type ValidatorRegistration = Session;
    type WeightInfo = xpallet_mining_staking::weights::SubstrateWeight<Runtime>;
}

/*

这段代码继续配置 ChainX 区块链运行时的多个 pallets,包括多重签名(`pallet_multisig`),
网关共同逻辑(`xpallet_gateway_common`),比特币网关(`xpallet_gateway_bitcoin`),
现货交易(`xpallet_dex_spot`),财政库(`xpallet_support`)以及挖矿和质押(`xpallet_mining_staking`).

### 多重签名(`pallet_multisig`)配置

- `MultisigProvider`: 一个结构体,用于计算多重签名账户的 ID.
- `calc_multisig`: 根据提供的账户列表和阈值,计算多重签名账户的 ID.
- `DetermineMultisigAddress`: 使用 `MultisigProvider` 来确定多重签名地址.

### 网关共同逻辑(`xpallet_gateway_common`)配置

- `Validator`: 用于验证比特币交易的质押模块.
- `DetermineMultisigAddress`: 使用 `MultisigProvider` 来确定多重签名地址.
- `CouncilOrigin`: 理事会至少 2/3 的成员同意才能执行的操作.
- `Bitcoin`, `BitcoinTrustee`, `BitcoinTrusteeSessionProvider`, `BitcoinTotalSupply`, `BitcoinWithdrawalProposal`: 与比特币网关相关的类型.
- `WeightInfo`: 权重信息,用于衡量操作的计算成本.

### 比特币网关(`xpallet_gateway_bitcoin`)配置

- `Event`: 事件类型.
- `UnixTime`: 时间戳类型.
- `CouncilOrigin`: 理事会至少 2/3 的成员同意才能执行的操作.
- `AccountExtractor`: 用于从比特币交易中提取账户信息的工具.
- `TrusteeSessionProvider`, `TrusteeInfoUpdate`, `ReferralBinding`, `AddressBinding`: 与比特币信托和推荐绑定相关的类型.
- `WeightInfo`: 权重信息.

### 现货交易(`xpallet_dex_spot`)配置

- `Event`: 事件类型.
- `Price`: 价格类型,使用余额(`Balance`)表示.
- `WeightInfo`: 权重信息.

### 财政库(`xpallet_support`)配置

- `TreasuryAccount`: 一个结构体,表示财政库账户.
- `treasury_account`: 返回财政库账户的 ID.

### 挖矿和质押(`xpallet_mining_staking`)配置

- `Event`: 事件类型.
- `Currency`: 货币类型.
- `SessionDuration`: 会话持续时间.
- `MinimumReferralId`, `MaximumReferralId`: 最小和最大推荐 ID.
- `SessionInterface`: 会话接口.
- `TreasuryAccount`: 财政库账户.
- `AssetMining`: 资产挖矿模块.
- `DetermineRewardPotAccount`: 确定奖励池账户的逻辑.
- `ValidatorRegistration`: 验证者注册的会话.
- `WeightInfo`: 权重信息.

### 参数类型配置

- `MigrationSessionOffset`: 迁移会话偏移量.
- `MinimumReferralId`, `MaximumReferralId`: 最小和最大推荐 ID.

这些配置确保了 ChainX 区块链的多重签名,网关,现货交易,财政库以及挖矿和质押功能能够按照预定的规则和参数运行.
*/

pub struct ReferralGetter;
impl xpallet_mining_asset::GatewayInterface<AccountId> for ReferralGetter {
    fn referral_of(who: &AccountId, asset_id: AssetId) -> Option<AccountId> {
        use xpallet_gateway_common::traits::ReferralBinding;
        XGatewayCommon::referral(&asset_id, who)
    }
}

impl xpallet_mining_asset::Config for Runtime {
    type Event = Event;
    type StakingInterface = Self;
    type GatewayInterface = ReferralGetter;
    type TreasuryAccount = SimpleTreasuryAccount;
    type DetermineRewardPotAccount =
        xpallet_mining_asset::SimpleAssetRewardPotAccountDeterminer<Runtime>;
    type WeightInfo = xpallet_mining_asset::weights::SubstrateWeight<Runtime>;
}

impl xpallet_genesis_builder::Config for Runtime {}

impl xpallet_ethereum_chain_id::Config for Runtime {}

impl xpallet_btc_ledger::Config for Runtime {
    type Balance = Balance;
    type Event = Event;
    type CouncilOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>;
    type PalletId = TreasuryPalletId;
}

/*
这段代码继续配置 ChainX 区块链运行时的特定模块,包括挖矿资产(`xpallet_mining_asset`),
创世构建器(`xpallet_genesis_builder`),以太坊链 ID(`xpallet_ethereum_chain_id`)和比特币账本(`xpallet_btc_ledger`).

### 挖矿资产(`xpallet_mining_asset`)配置

- `ReferralGetter`: 一个结构体,实现了 `GatewayInterface` trait,用于获取特定资产的推荐人账户 ID.
- `referral_of`: 根据给定的账户和资产 ID,返回推荐人的账户 ID.
- `StakingInterface`: 质押接口,这里指代自身(`Self`),意味着质押逻辑是内置的.
- `GatewayInterface`: 网关接口,使用 `ReferralGetter` 来获取推荐信息.
- `TreasuryAccount`: 财政库账户,使用 `SimpleTreasuryAccount` 结构体.
- `DetermineRewardPotAccount`: 确定奖励池账户的逻辑.
- `WeightInfo`: 权重信息,用于衡量操作的计算成本.

### 比特币账本(`xpallet_btc_ledger`)配置

- `Balance`: 比特币账本使用的余额类型.
- `Event`: 事件类型.
- `CouncilOrigin`: 理事会至少 2/3 的成员同意才能执行的操作.
- `PalletId`: 财政库模块的标识符.

这些配置确保了 ChainX 区块链的挖矿资产,创世构建器,以太坊链 ID 和比特币账本功能能够按照预定的规则和参数运行.
*/

/// Current approximation of the gas/s consumption considering
/// EVM execution over compiled WASM (on 4.4Ghz CPU).
/// Given the 500ms Weight, from which 75% only are used for transactions,
/// the total EVM execution gas limit is: GAS_PER_SECOND * 0.500 * 0.75 ~= 15_000_000.
pub const GAS_PER_SECOND: u64 = 40_000_000;

/// Approximate ratio of the amount of Weight per Gas.
/// u64 works for approximations because Weight is a very small unit compared to gas.
pub const WEIGHT_PER_GAS: u64 = WEIGHT_PER_SECOND / GAS_PER_SECOND;

/// Maximum weight per block
pub const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;

parameter_types! {
    pub BlockGasLimit: U256
        = U256::from(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT / WEIGHT_PER_GAS);
    pub PrecompilesValue: ChainXPrecompiles<Runtime> = ChainXPrecompiles::<_>::new();
}

pub struct ChainXGasWeightMapping;
impl pallet_evm::GasWeightMapping for ChainXGasWeightMapping {
    fn gas_to_weight(gas: u64) -> Weight {
        gas.saturating_mul(WEIGHT_PER_GAS)
    }
    fn weight_to_gas(weight: Weight) -> u64 {
        weight.wrapping_div(WEIGHT_PER_GAS)
    }
}

impl pallet_evm::Config for Runtime {
    type FeeCalculator = BaseFee;
    type GasWeightMapping = ChainXGasWeightMapping;
    type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
    type CallOrigin = EnsureAddressRoot<AccountId>;
    type WithdrawOrigin = EnsureAddressNever<AccountId>;
    type AddressMapping = HashedAddressMapping<BlakeTwo256>;
    type Currency = XBtcLedger;
    type Event = Event;
    type Runner = pallet_evm::runner::stack::Runner<Self>;
    type PrecompilesType = ChainXPrecompiles<Runtime>;
    type PrecompilesValue = PrecompilesValue;
    type ChainId = EthereumChainId;
    type OnChargeTransaction = pallet_evm::EVMCurrencyAdapter<XBtcLedger, DealWithBTCFees>;
    type BlockGasLimit = BlockGasLimit;
    type FindAuthor = ();
    type WeightInfo = pallet_evm::weights::SubstrateWeight<Self>;
}

impl pallet_ethereum::Config for Runtime {
    type Event = Event;
    type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
}

parameter_types! {
    pub DefaultBaseFeePerGas: U256 = U256::from(BASE_FEE);
}

pub struct BaseFeeThreshold;
impl pallet_base_fee::BaseFeeThreshold for BaseFeeThreshold {
    fn lower() -> Permill {
        Permill::zero()
    }
    fn ideal() -> Permill {
        Permill::from_parts(500_000)
    }
    fn upper() -> Permill {
        Permill::from_parts(1_000_000)
    }
}

impl pallet_base_fee::Config for Runtime {
    type Event = Event;
    type Threshold = BaseFeeThreshold;
    // Tells `pallet_base_fee` whether to calculate a new BaseFee `on_finalize` or not.
    type IsActive = ConstBool<false>;
    type DefaultBaseFeePerGas = DefaultBaseFeePerGas;
}

parameter_types! {
    // 0x1111111111111111111111111111111111111111
    pub EvmCaller: H160 = H160::from_slice(&[17u8;20][..]);
    pub ClaimBond: Balance = PCXS;
}
impl xpallet_assets_bridge::Config for Runtime {
    type Event = Event;
    type EvmCaller = EvmCaller;
    type ClaimBond = ClaimBond;
}

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = chainx_primitives::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        // Basic stuff.
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,
        Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 2,

        // Must be before session.
        Babe: pallet_babe::{Pallet, Call, Storage, Config, ValidateUnsigned} = 3,

        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 4,
        Indices: pallet_indices::{Pallet, Call, Storage, Config<T>, Event<T>} = 5,
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 6,
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage} = 7,

        // Consensus support.
        Authorship: pallet_authorship::{Pallet, Call, Storage, Inherent} = 8,
        Offences: pallet_offences::{Pallet, Storage, Event} = 9,
        Historical: pallet_session_historical::{Pallet} = 10,
        Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 11,
        Grandpa: pallet_grandpa::{Pallet, Call, Storage, Config, Event, ValidateUnsigned} = 12,
        ImOnline: pallet_im_online::{Pallet, Call, Storage, Event<T>, ValidateUnsigned, Config<T>} = 13,
        AuthorityDiscovery: pallet_authority_discovery::{Pallet, Config} = 14,

        // Governance stuff.
        Democracy: pallet_democracy::{Pallet, Call, Storage, Config<T>, Event<T>} = 15,
        Council: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 16,
        TechnicalCommittee: pallet_collective::<Instance2>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 17,
        Elections: pallet_elections_phragmen::{Pallet, Call, Storage, Event<T>, Config<T>} = 18,
        TechnicalMembership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} = 19,
        Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>} = 20,

        Identity: pallet_identity::{Pallet, Call, Storage, Event<T>} = 21,

        Utility: pallet_utility::{Pallet, Call, Event} = 22,
        Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 23,

        // ChainX basics.
        XSystem: xpallet_system::{Pallet, Call, Storage, Event<T>, Config} = 24,
        XAssetsRegistrar: xpallet_assets_registrar::{Pallet, Call, Storage, Event<T>, Config} = 25,
        XAssets: xpallet_assets::{Pallet, Call, Storage, Event<T>, Config<T>} = 26,

        // Mining, must be after XAssets.
        XStaking: xpallet_mining_staking::{Pallet, Call, Storage, Event<T>, Config<T>} = 27,
        XMiningAsset: xpallet_mining_asset::{Pallet, Call, Storage, Event<T>, Config<T>} = 28,

        // Crypto gateway stuff.
        XGatewayRecords: xpallet_gateway_records::{Pallet, Call, Storage, Event<T>} = 29,
        XGatewayCommon: xpallet_gateway_common::{Pallet, Call, Storage, Event<T>, Config<T>} = 30,
        XGatewayBitcoin: xpallet_gateway_bitcoin::{Pallet, Call, Storage, Event<T>, Config<T>} = 31,

        // DEX
        XSpot: xpallet_dex_spot::{Pallet, Call, Storage, Event<T>, Config<T>} = 32,

        XGenesisBuilder: xpallet_genesis_builder::{Pallet, Config<T>} = 33,

        // It might be possible to merge this module into pallet_transaction_payment in future, thus
        // we put it at the end for keeping the extrinsic ordering.
        XTransactionFee: xpallet_transaction_fee::{Pallet, Event<T>} = 35,

        Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>} = 36,

        Bounties: pallet_bounties::{Pallet, Call, Storage, Event<T>} = 37,
        Tips: pallet_tips::{Pallet, Call, Storage, Event<T>} = 38,

        // Ethereum compatibility
        EthereumChainId: xpallet_ethereum_chain_id::{Pallet, Call, Storage, Config} = 40,
        Evm: pallet_evm::{Pallet, Config, Call, Storage, Event<T>} = 41,
        Ethereum: pallet_ethereum::{Pallet, Call, Storage, Event, Config, Origin} = 42,
        BaseFee: pallet_base_fee::{Pallet, Call, Storage, Config<T>, Event} = 44,

        // Dependency on xpallet_assets and pallet_evm
        XAssetsBridge: xpallet_assets_bridge::{Pallet, Call, Storage, Config<T>, Event<T>} = 45,

        XBtcLedger: xpallet_btc_ledger::{Pallet, Call, Storage, Config<T>, Event<T>} = 46,
    }
);

/*
这段代码定义了 ChainX 区块链运行时的一些关键配置,特别是与以太坊虚拟机(EVM)和交易费用相关的部分.下面是每个配置的详细解释:

### 交易费用和气体限制参数配置

- `BlockGasLimit`: 定义了区块中可以执行的最大气体(Gas)数量.
这是通过将 `NORMAL_DISPATCH_RATIO` 乘以 `MAXIMUM_BLOCK_WEIGHT` 然后除以 `WEIGHT_PER_GAS` 来计算得出的.

- `PrecompilesValue`: 预编译合约的值,这是 `ChainXPrecompiles` 的一个新实例.

### EVM 配置

- `ChainXGasWeightMapping`: 实现了 `GasWeightMapping` trait,用于将 Gas 转换为 Substrate 的权重(Weight).
- `pallet_evm::Config`: 为 EVM 模块提供了配置,包括费用计算器,GasWeightMapping,区块哈希映射,调用起源,地址映射,货币类型,事件类型等.

### 基本费用配置

- `BaseFeeThreshold`: 定义了基本费用的上下界限.这是 `pallet_base_fee` 模块的一部分,用于实现动态基本费用调整.
- `pallet_base_fee::Config`: 为基本费用模块提供了配置,包括事件类型,阈值设置,是否激活,默认基本费用每 Gas 以及权重信息.

### 资产桥接配置

- `EvmCaller`: 定义了 EVM 调用者的地址.
- `ClaimBond`: 定义了在资产桥接过程中索赔所需的保证金.
- `xpallet_assets_bridge::Config`: 为资产桥接模块提供了配置,包括事件类型,EVM 调用者地址和索赔保证金.

### 构造运行时(`construct_runtime!`)

这是一个宏,用于构建 ChainX 区块链的运行时环境.它定义了运行时中包含的所有 pallets,以及它们的顺序和类型.
这个宏是 Substrate 框架的一部分,用于生成运行时的最终代码.

这些配置确保了 ChainX 区块链的 EVM 兼容性,交易费用管理和资产桥接功能能够按照预定的规则和参数运行.通过这些配置,
ChainX 能够实现与以太坊智能合约的互操作性,同时保持网络的稳定性和可靠性,并有效管理交易费用.

------------------------------------------------------------------------------------------------
在 ChainX 区块链中,将 Gas 转换为 Substrate 的权重是为了将 EVM(以太坊虚拟机)的交易费用模型与 Substrate 框架的权重系统相结合.
这种转换允许 ChainX 区块链在处理交易时,能够考虑到不同操作的计算复杂性和资源消耗.

### EVM 的 Gas 模型

以太坊使用 Gas 作为衡量交易和智能合约操作成本的单位.每个操作(如执行指令,存储数据等)都需要消耗一定量的 Gas.
交易发起者必须支付 Gas 费用,这些费用以以太币(ETH)的形式支付给矿工作为交易处理的激励.

### Substrate 的权重模型

Substrate 框架使用权重(Weight)来衡量交易和区块执行所需的计算资源.权重系统允许节点预估执行交易所需的资源量,
从而防止恶意交易或那些可能导致节点资源耗尽的操作.

### 转换的目的

将 Gas 转换为权重的目的是在 ChainX 区块链上实现 EVM 与 Substrate 框架的兼容性.这种转换使得 ChainX 能够:

1. **资源管理**:通过权重系统管理区块链资源的使用,确保网络的稳定性和安全性.
2. **费用预估**:允许节点在执行交易前预估所需的计算资源,避免执行可能导致资源耗尽的交易.
3. **交易优先级**:根据交易的权重(Gas)费用来决定交易的处理顺序,实现类似于 Gas 竞价的机制.
4. **兼容性**:使得以太坊上的智能合约和 DApps 能够在 ChainX 区块链上运行,无需或只需很少的修改.

### 实现细节

`ChainXGasWeightMapping` 结构体实现了 `pallet_evm::GasWeightMapping` trait,它定义了如何将 Gas 转换为权重:

- `gas_to_weight` 函数:将 Gas 转换为权重.这通常涉及到将 Gas 数量乘以一个预定义的转换率(`WEIGHT_PER_GAS`).
- `weight_to_gas` 函数:将权重转换回 Gas.这通常涉及到将权重除以相同的转换率.

通过这种方式,ChainX 区块链能够将 EVM 的 Gas 费用模型与 Substrate 的权重系统相结合,实现两者的互操作性和资源管理.
*/


/// The address format for describing accounts.
pub type Address = <Indices as StaticLookup>::Source;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
    BaseFilter,
    ChargeExtraFee,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
    fp_self_contained::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = fp_self_contained::CheckedExtrinsic<AccountId, Call, SignedExtra, H160>;

/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
    AssetsBridgeMigration,
>;

pub struct AssetsBridgeMigration;
impl OnRuntimeUpgrade for AssetsBridgeMigration {
    fn on_runtime_upgrade() -> Weight {
        use frame_support::storage::migration;

        frame_support::log::info!("🔍️ AssetsBridgeMigration start");

        // Remove the storage value `HotAccount` from  pallet `XAssetsBridge`
        migration::remove_storage_prefix(b"XAssetsBridge", b"HotAccount", b"");

        frame_support::log::info!("🚀 AssetsBridgeMigration end");

        <Runtime as frame_system::Config>::DbWeight::get().writes(1)
    }
}

pub struct TransactionConverter;
impl fp_rpc::ConvertTransaction<UncheckedExtrinsic> for TransactionConverter {
    fn convert_transaction(&self, transaction: pallet_ethereum::Transaction) -> UncheckedExtrinsic {
        UncheckedExtrinsic::new_unsigned(
            pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
        )
    }
}
impl fp_rpc::ConvertTransaction<sp_runtime::OpaqueExtrinsic> for TransactionConverter {
    fn convert_transaction(
        &self,
        transaction: pallet_ethereum::Transaction,
    ) -> sp_runtime::OpaqueExtrinsic {
        let extrinsic = UncheckedExtrinsic::new_unsigned(
            pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
        );
        let encoded = extrinsic.encode();
        sp_runtime::OpaqueExtrinsic::decode(&mut &encoded[..])
            .expect("Encoded extrinsic is always valid")
    }
}

impl fp_self_contained::SelfContainedCall for Call {
    type SignedInfo = H160;

    fn is_self_contained(&self) -> bool {
        match self {
            Call::Ethereum(call) => call.is_self_contained(),
            _ => false,
        }
    }

    fn check_self_contained(&self) -> Option<Result<Self::SignedInfo, TransactionValidityError>> {
        match self {
            Call::Ethereum(call) => call.check_self_contained(),
            _ => None,
        }
    }

    fn validate_self_contained(&self, info: &Self::SignedInfo) -> Option<TransactionValidity> {
        match self {
            Call::Ethereum(call) => call.validate_self_contained(info),
            _ => None,
        }
    }

    fn pre_dispatch_self_contained(
        &self,
        info: &Self::SignedInfo,
    ) -> Option<Result<(), TransactionValidityError>> {
        match self {
            Call::Ethereum(call) => call.pre_dispatch_self_contained(info),
            _ => None,
        }
    }

    fn apply_self_contained(
        self,
        info: Self::SignedInfo,
    ) -> Option<sp_runtime::DispatchResultWithInfo<PostDispatchInfoOf<Self>>> {
        match self {
            call @ Call::Ethereum(pallet_ethereum::Call::transact { .. }) => Some(call.dispatch(
                Origin::from(pallet_ethereum::RawOrigin::EthereumTransaction(info)),
            )),
            _ => None,
        }
    }
}

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            OpaqueMetadata::new(Runtime::metadata().into())
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx, block_hash)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_consensus_babe::BabeApi<Block> for Runtime {
        fn configuration() -> sp_consensus_babe::BabeGenesisConfiguration {
            // The choice of `c` parameter (where `1 - c` represents the
            // probability of a slot being empty), is done in accordance to the
            // slot duration and expected target block time, for safely
            // resisting network delays of maximum two seconds.
            // <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
            sp_consensus_babe::BabeGenesisConfiguration {
                slot_duration: Babe::slot_duration(),
                epoch_length: EpochDuration::get(),
                c: BABE_GENESIS_EPOCH_CONFIG.c,
                genesis_authorities: Babe::authorities().to_vec(),
                randomness: Babe::randomness(),
                allowed_slots: BABE_GENESIS_EPOCH_CONFIG.allowed_slots,
            }
        }

        fn current_epoch_start() -> sp_consensus_babe::Slot {
            Babe::current_epoch_start()
        }

        fn current_epoch() -> sp_consensus_babe::Epoch {
            Babe::current_epoch()
        }

        fn next_epoch() -> sp_consensus_babe::Epoch {
            Babe::next_epoch()
        }

        fn generate_key_ownership_proof(
            _slot: sp_consensus_babe::Slot,
            authority_id: sp_consensus_babe::AuthorityId,
        ) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
            Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
            key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;

            Babe::submit_unsigned_equivocation_report(
                equivocation_proof,
                key_owner_proof,
            )
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl fg_primitives::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> GrandpaAuthorityList {
            Grandpa::grandpa_authorities()
        }

        fn current_set_id() -> fg_primitives::SetId {
            Grandpa::current_set_id()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: fg_primitives::EquivocationProof<
                <Block as BlockT>::Hash,
                NumberFor<Block>,
            >,
            key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;

            Grandpa::submit_unsigned_equivocation_report(
                equivocation_proof,
                key_owner_proof,
            )
        }

        fn generate_key_ownership_proof(
            _set_id: fg_primitives::SetId,
            authority_id: GrandpaId,
        ) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
            Historical::prove((fg_primitives::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(fg_primitives::OpaqueKeyOwnershipProof::new)
        }
    }

    impl sp_authority_discovery::AuthorityDiscoveryApi<Block> for Runtime {
        fn authorities() -> Vec<AuthorityDiscoveryId> {
            AuthorityDiscovery::authorities()
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
        fn query_info(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            if let Some(extra_fee) = ChargeExtraFee::has_extra_fee(&uxt.0.function) {
                let base_info = TransactionPayment::query_info(uxt, len);
                pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo {
                    partial_fee: base_info.partial_fee + extra_fee,
                    ..base_info
                }
            } else {
                TransactionPayment::query_info(uxt, len)
            }
        }
        fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
    }

    impl xpallet_transaction_fee_rpc_runtime_api::XTransactionFeeApi<Block, Balance> for Runtime {
        fn query_fee_details(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> xpallet_transaction_fee::FeeDetails<Balance> {
            let maybe_extra = ChargeExtraFee::has_extra_fee(&uxt.0.function);
            let base = TransactionPayment::query_fee_details(uxt, len);
            xpallet_transaction_fee::FeeDetails::new(base, maybe_extra)
        }
    }

    impl xpallet_assets_rpc_runtime_api::XAssetsApi<Block, AccountId, Balance> for Runtime {
        fn assets_for_account(who: AccountId) -> BTreeMap<AssetId, BTreeMap<AssetType, Balance>> {
            XAssets::valid_assets_of(&who)
        }

        fn assets() -> BTreeMap<AssetId, TotalAssetInfo<Balance>> {
            XAssets::total_asset_infos()
        }
    }

    impl xpallet_mining_staking_rpc_runtime_api::XStakingApi<Block, AccountId, Balance, VoteWeight, BlockNumber> for Runtime {
        fn validators() -> Vec<ValidatorInfo<AccountId, Balance, VoteWeight, BlockNumber>> {
            XStaking::validators_info()
        }
        fn validator_info_of(who: AccountId) -> ValidatorInfo<AccountId, Balance, VoteWeight, BlockNumber> {
            XStaking::validator_info_of(who)
        }
        fn staking_dividend_of(who: AccountId) -> BTreeMap<AccountId, Balance> {
            XStaking::staking_dividend_of(who)
        }
        fn nomination_details_of(who: AccountId) -> BTreeMap<AccountId, NominatorLedger<Balance, VoteWeight, BlockNumber>> {
            XStaking::nomination_details_of(who)
        }
        fn nominator_info_of(who: AccountId) -> NominatorInfo<BlockNumber> {
            XStaking::nominator_info_of(who)
        }
    }

    impl xpallet_dex_spot_rpc_runtime_api::XSpotApi<Block, AccountId, Balance, BlockNumber, Balance> for Runtime {
        fn trading_pairs() -> Vec<FullPairInfo<Balance, BlockNumber>> {
            XSpot::trading_pairs()
        }

        fn orders(who: AccountId, page_index: u32, page_size: u32) -> Vec<RpcOrder<TradingPairId, AccountId, Balance, Balance, BlockNumber>> {
            XSpot::orders(who, page_index, page_size)
        }

        fn depth(pair_id: TradingPairId, depth_size: u32) -> Option<Depth<Balance, Balance>> {
            XSpot::depth(pair_id, depth_size)
        }
    }

    impl xpallet_mining_asset_rpc_runtime_api::XMiningAssetApi<Block, AccountId, Balance, MiningWeight, BlockNumber> for Runtime {
        fn mining_assets() -> Vec<MiningAssetInfo<AccountId, Balance, MiningWeight, BlockNumber>> {
            XMiningAsset::mining_assets()
        }

        fn mining_dividend(who: AccountId) -> BTreeMap<AssetId, MiningDividendInfo<Balance>> {
            XMiningAsset::mining_dividend(who)
        }

        fn miner_ledger(who: AccountId) -> BTreeMap<AssetId, MinerLedger<MiningWeight, BlockNumber>> {
            XMiningAsset::miner_ledger(who)
        }
    }

    impl xpallet_gateway_records_rpc_runtime_api::XGatewayRecordsApi<Block, AccountId, Balance, BlockNumber> for Runtime {
        fn withdrawal_list() -> BTreeMap<u32, Withdrawal<AccountId, Balance, BlockNumber>> {
            XGatewayRecords::withdrawal_list()
        }

        fn withdrawal_list_by_chain(chain: Chain) -> BTreeMap<u32, Withdrawal<AccountId, Balance, BlockNumber>> {
            XGatewayRecords::withdrawals_list_by_chain(chain)
        }
    }

    impl xpallet_gateway_bitcoin_rpc_runtime_api::XGatewayBitcoinApi<Block, AccountId> for Runtime {
        fn verify_tx_valid(
            raw_tx: Vec<u8>,
            withdrawal_id_list: Vec<u32>,
            full_amount: bool,
        ) -> Result<bool, DispatchError> {
            XGatewayBitcoin::verify_tx_valid(raw_tx, withdrawal_id_list, full_amount)
        }

        fn get_withdrawal_proposal() -> Option<BtcWithdrawalProposal<AccountId>> {
            XGatewayBitcoin::get_withdrawal_proposal()
        }

        fn get_genesis_info() -> (BtcHeader, u32) {
            XGatewayBitcoin::get_genesis_info()
        }

        fn get_btc_block_header(txid: H256) -> Option<BtcHeaderInfo> {
            XGatewayBitcoin::get_btc_block_header(txid)
        }
    }

    impl xpallet_btc_ledger_runtime_api::BtcLedgerApi<Block, AccountId, Balance> for Runtime {
        fn get_balance(who: AccountId) -> Balance {
            XBtcLedger::free_balance(&who)
        }
        fn get_total() -> Balance {
            XBtcLedger::get_total()
        }
    }

    impl xpallet_gateway_common_rpc_runtime_api::XGatewayCommonApi<Block, AccountId, Balance, BlockNumber> for Runtime {
        fn bound_addrs(who: AccountId) -> BTreeMap<Chain, Vec<ChainAddress>> {
            XGatewayCommon::bound_addrs(&who)
        }

        fn withdrawal_limit(asset_id: AssetId) -> Result<WithdrawalLimit<Balance>, DispatchError> {
            XGatewayCommon::withdrawal_limit(&asset_id)
        }

        #[allow(clippy::type_complexity)]
        fn withdrawal_list_with_fee_info(asset_id: AssetId) -> Result<
            BTreeMap<
                WithdrawalRecordId,
                (
                    Withdrawal<AccountId, Balance, BlockNumber>,
                    WithdrawalLimit<Balance>,
                ),
            >,
            DispatchError,
        >
        {
            XGatewayCommon::withdrawal_list_with_fee_info(&asset_id)
        }

        fn verify_withdrawal(asset_id: AssetId, value: Balance, addr: AddrStr, memo: Memo) -> Result<(), DispatchError> {
            XGatewayCommon::verify_withdrawal(asset_id, value, &addr, &memo)
        }

        fn trustee_multisigs() -> BTreeMap<Chain, AccountId> {
            XGatewayCommon::trustee_multisigs()
        }

        fn trustee_properties(chain: Chain, who: AccountId) -> Option<GenericTrusteeIntentionProps<AccountId>> {
            XGatewayCommon::trustee_intention_props_of(who, chain)
        }

        fn trustee_session_info(chain: Chain, session_number: i32) -> Option<GenericTrusteeSessionInfo<AccountId, BlockNumber>> {
            if session_number < 0 {
                let number = match session_number {
                    -1i32 => Some(XGatewayCommon::trustee_session_info_len(chain)),
                    -2i32 => XGatewayCommon::trustee_session_info_len(chain).checked_sub(1),
                    _ => None
                };
                if let Some(number) = number {
                    XGatewayCommon::trustee_session_info_of(chain, number)
                }else{
                    None
                }
            }else{
                let number = session_number as u32;
                XGatewayCommon::trustee_session_info_of(chain, number)
            }

        }

        fn generate_trustee_session_info(chain: Chain, candidates: Vec<AccountId>) -> Result<(GenericTrusteeSessionInfo<AccountId, BlockNumber>, ScriptInfo<AccountId>), DispatchError> {
            let info = XGatewayCommon::try_generate_session_info(chain, candidates)?;
            // check multisig address
            let _ = XGatewayCommon::generate_multisig_addr(chain, &info.0)?;
            Ok(info)
        }
    }

    impl fp_rpc::ConvertTransactionRuntimeApi<Block> for Runtime {
        fn convert_transaction(transaction: EthereumTransaction) -> <Block as BlockT>::Extrinsic {
            UncheckedExtrinsic::new_unsigned(
                pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
            )
        }
    }

    impl fp_rpc::EthereumRuntimeRPCApi<Block> for Runtime {
        fn chain_id() -> u64 {
            <Runtime as pallet_evm::Config>::ChainId::get()
        }

        fn account_basic(address: H160) -> EVMAccount {
            Evm::account_basic(&address)
        }

        fn gas_price() -> U256 {
            <Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price()
        }

        fn account_code_at(address: H160) -> Vec<u8> {
            Evm::account_codes(address)
        }

        fn author() -> H160 {
            <pallet_evm::Pallet<Runtime>>::find_author()
        }

        fn storage_at(address: H160, index: U256) -> H256 {
            let mut tmp = [0u8; 32];
            index.to_big_endian(&mut tmp);
            Evm::account_storages(address, H256::from_slice(&tmp[..]))
        }

        #[allow(clippy::redundant_closure)]
        fn call(
            from: H160,
            to: H160,
            data: Vec<u8>,
            value: U256,
            gas_limit: U256,
            max_fee_per_gas: Option<U256>,
            max_priority_fee_per_gas: Option<U256>,
            nonce: Option<U256>,
            estimate: bool,
            access_list: Option<Vec<(H160, Vec<H256>)>>,
        ) -> Result<pallet_evm::CallInfo, sp_runtime::DispatchError> {
            let config = if estimate {
                let mut config = <Runtime as pallet_evm::Config>::config().clone();
                config.estimate = true;
                Some(config)
            } else {
                None
            };

            let is_transactional = false;
            <Runtime as pallet_evm::Config>::Runner::call(
                from,
                to,
                data,
                value,
                gas_limit.low_u64(),
                max_fee_per_gas,
                max_priority_fee_per_gas,
                nonce,
                access_list.unwrap_or_default(),
                is_transactional,
                config.as_ref().unwrap_or_else(|| <Runtime as pallet_evm::Config>::config()),
            ).map_err(|err| err.into())
        }

        #[allow(clippy::redundant_closure)]
        fn create(
            from: H160,
            data: Vec<u8>,
            value: U256,
            gas_limit: U256,
            max_fee_per_gas: Option<U256>,
            max_priority_fee_per_gas: Option<U256>,
            nonce: Option<U256>,
            estimate: bool,
            access_list: Option<Vec<(H160, Vec<H256>)>>,
        ) -> Result<pallet_evm::CreateInfo, sp_runtime::DispatchError> {
            let config = if estimate {
                let mut config = <Runtime as pallet_evm::Config>::config().clone();
                config.estimate = true;
                Some(config)
            } else {
                None
            };

            let is_transactional = false;
            <Runtime as pallet_evm::Config>::Runner::create(
                from,
                data,
                value,
                gas_limit.low_u64(),
                max_fee_per_gas,
                max_priority_fee_per_gas,
                nonce,
                access_list.unwrap_or_default(),
                is_transactional,
                config.as_ref().unwrap_or_else(|| <Runtime as pallet_evm::Config>::config()),
            ).map_err(|err| err.into())
        }

        fn current_transaction_statuses() -> Option<Vec<TransactionStatus>> {
            Ethereum::current_transaction_statuses()
        }

        fn current_block() -> Option<pallet_ethereum::Block> {
            Ethereum::current_block()
        }

        fn current_receipts() -> Option<Vec<pallet_ethereum::Receipt>> {
            Ethereum::current_receipts()
        }

        fn current_all() -> (
            Option<pallet_ethereum::Block>,
            Option<Vec<pallet_ethereum::Receipt>>,
            Option<Vec<TransactionStatus>>
        ) {
            (
                Ethereum::current_block(),
                Ethereum::current_receipts(),
                Ethereum::current_transaction_statuses()
            )
        }

        fn extrinsic_filter(
            xts: Vec<<Block as BlockT>::Extrinsic>,
        ) -> Vec<EthereumTransaction> {
            xts.into_iter().filter_map(|xt| match xt.0.function {
                Call::Ethereum(transact { transaction }) => Some(transaction),
                _ => None
            }).collect::<Vec<EthereumTransaction>>()
        }

        fn elasticity() -> Option<Permill> {
            Some(BaseFee::elasticity())
        }
    }

    #[cfg(feature = "try-runtime")]
    impl frame_try_runtime::TryRuntime<Block> for Runtime {
        fn on_runtime_upgrade() -> (Weight, Weight) {
            // NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
            // have a backtrace here. If any of the pre/post migration checks fail, we shall stop
            // right here and right now.
            let weight = Executive::try_runtime_upgrade().unwrap();
            (weight, BlockWeights::get().max_block)
        }

        fn execute_block_no_check(block: Block) -> Weight {
            Executive::execute_block_no_check(block)
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn benchmark_metadata(extra: bool) -> (
            Vec<frame_benchmarking::BenchmarkList>,
            Vec<frame_support::traits::StorageInfo>,
        ) {
            use frame_benchmarking::{list_benchmark, Benchmarking, BenchmarkList};
            use frame_support::traits::StorageInfoTrait;

            let mut list = Vec::<BenchmarkList>::new();

            list_benchmark!(list, extra, xpallet_assets, XAssets);
            list_benchmark!(list, extra, xpallet_assets_registrar, XAssetsRegistrar);
            list_benchmark!(list, extra, xpallet_mining_asset, XMiningAsset);
            list_benchmark!(list, extra, xpallet_mining_staking, XStaking);
            list_benchmark!(list, extra, xpallet_gateway_records, XGatewayRecords);
            list_benchmark!(list, extra, xpallet_gateway_common, XGatewayCommon);
            list_benchmark!(list, extra, xpallet_gateway_bitcoin, XGatewayBitcoin);
            list_benchmark!(list, extra, xpallet_dex_spot, XSpot);

            let storage_info = AllPalletsWithSystem::storage_info();

            return (list, storage_info)
        }

        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, RuntimeString> {
            use frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch, TrackedStorageKey};

            use frame_system_benchmarking::Pallet as SystemBench;
            use baseline::Pallet as BaselineBench;

            impl frame_system_benchmarking::Config for Runtime {}
            impl baseline::Config for Runtime {}

            let whitelist: Vec<TrackedStorageKey> = vec![
                // // Block Number
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
                // // Total Issuance
                hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
                // // Execution Phase
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
                // // Event Count
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
                // // System Events
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
                // // Treasury Account
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da95ecffd7b6c0f78751baa9d281e0bfa3a6d6f646c70792f74727372790000000000000000000000000000000000000000").to_vec().into(),
            ];

            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&config, &whitelist);

            add_benchmarks!(params, batches);

            if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
            Ok(batches)
        }
    }
}

/*
这段代码是 ChainX 区块链运行时的一部分,它定义了区块链的数据结构,类型别名和一些特定的运行时 API 实现.
这些定义对于区块链的正常运行至关重要,因为它们涉及到交易处理,区块构建,共识机制,账户管理等多个方面.

以下是代码中定义的一些关键类型和它们的用途:

- `Address`: 用于描述账户地址的类型.
- `Header`: 区块头的类型,包含区块编号和哈希.
- `Block`: 区块的类型,由区块头和未检查的外部交易组成.
- `SignedBlock`: 带有签名的区块类型.
- `BlockId`: 区块 ID 的类型,用于唯一标识区块.
- `SignedExtra`: 交易签名扩展,包含多种检查和费用计算的逻辑.
- `UncheckedExtrinsic` 和 `CheckedExtrinsic`: 分别代表未检查和已检查的外部交易类型.
- `SignedPayload`: 交易的有效载荷,包含调用和签名扩展.
- `Executive`: 执行器,负责将交易分发到不同的模块.

此外,代码还实现了一些特定的运行时 API,例如:

- `sp_api::Core`: 核心 API,提供版本信息和区块执行功能.
- `sp_block_builder::BlockBuilder`: 区块构建 API,允许添加交易和最终化区块.
- `sp_transaction_pool::runtime_api::TaggedTransactionQueue`: 交易池 API,用于验证交易.
- `sp_offchain::OffchainWorkerApi`: 离链工作 API,用于执行与区块链交互的后台任务.
- `sp_consensus_babe::BabeApi`: BABE 共识机制 API,提供共识相关的配置和功能.
- `sp_session::SessionKeys`: 会话密钥 API,用于生成和管理会话密钥.
- `pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi`: 交易支付 API,用于查询交易费用信息.

还有针对 ChainX 特定功能的 API,如 `xpallet_assets_rpc_runtime_api::XAssetsApi`,
`xpallet_mining_staking_rpc_runtime_api::XStakingApi` 等,它们提供了与资产管理,质押挖矿等相关的功能.

最后,代码还包括了一些用于运行时升级和基准测试的配置和实现.

这段代码是 ChainX 区块链运行时的关键组成部分,确保了区块链的各种功能可以正常运行和互操作.通过这些定义和实现,
ChainX 能够支持复杂的交易处理,区块生产和治理机制,同时保持与以太坊等其他区块链系统的兼容性.
*/

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
    define_benchmarks!(
        [frame_benchmarking, BaselineBench::<Runtime>]
        [frame_system, SystemBench::<Runtime>]
        [xpallet_assets, XAssets]
        [xpallet_assets_registrar, XAssetsRegistrar]
        [xpallet_mining_asset, XMiningAsset]
        [xpallet_mining_staking, XStaking]
        [xpallet_gateway_records, XGatewayRecords]
        [xpallet_gateway_common,  XGatewayCommon]
        [xpallet_gateway_bitcoin, XGatewayBitcoin]
        [xpallet_dex_spot, XSpot]
    );
}

/*
这段代码是ChainX区块链的运行时(runtime)配置,它定义了区块链的各种参数,模块和API.
运行时是Substrate框架的核心部分,负责处理区块链的逻辑和状态.以下是代码的详细解释:

### 运行时配置(Runtime Configuration)

代码开始部分定义了一系列的参数类型(`parameter_types!`),这些参数用于配置区块链的不同方面,例如:

- `BlockHashCount`:区块链保留块哈希的数目.
- `MaximumBlockWeight`:区块的最大权重.
- `SS58Prefix`:用于地址格式的SS58前缀.
- `EpochDuration`,`ExpectedBlockTime`:BABE共识算法的配置参数.

### 模块配置(Module Configuration)

接下来,代码配置了ChainX区块链的各个模块.每个模块都有其特定的配置类型,例如:

- `frame_system`:系统模块,负责账户,区块和事件等基本功能.
- `pallet_timestamp`:时间戳模块,用于记录区块的时间.
- `pallet_indices`:索引模块,允许用户创建唯一的账户索引.
- `pallet_balances`:余额模块,处理账户余额和转账.
- `pallet_transaction_payment`:交易支付模块,用于收取交易费用.
- `pallet_im_online`:在线模块,用于跟踪验证者的在线状态.
- `pallet_grandpa`,`pallet_babe`,`pallet_authorship`:共识模块,用于区块的生产和验证.
- `pallet_session`:会话模块,管理验证者集合.
- `pallet_authority_discovery`:权威发现模块,用于发现网络中的验证者.
- `pallet_democracy`,`pallet_collective`,`pallet_elections_phragmen`:治理模块,用于社区投票和选举.
- `pallet_treasury`:国库模块,管理区块链的财政.
- `pallet_identity`:身份模块,用于账户身份验证.
- `pallet_multisig`:多重签名模块,允许创建需要多个签名的交易.
- `pallet_bounties`,`pallet_tips`:赏金和打赏模块,用于激励社区贡献.
- `pallet_scheduler`:调度模块,用于安排未来的任务.
- `pallet_proxy`:代理模块,允许账户代理其投票权.
- `pallet_evm`,`pallet_ethereum`:以太坊虚拟机(EVM)模块,用于兼容以太坊智能合约.

### ChainX特定模块(ChainX Specific Modules)

代码还包括ChainX特有的模块,例如:

- `xpallet_system`:ChainX的系统模块.
- `xpallet_assets`,`xpallet_assets_registrar`:资产管理模块,用于创建和管理资产.
- `xpallet_mining_staking`,`xpallet_mining_asset`:挖矿和质押模块,用于挖矿奖励和资产分配.
- `xpallet_gateway_records`,`xpallet_gateway_common`,`xpallet_gateway_bitcoin`:加密货币网关模块,用于跨链资产转移.

### 运行时升级(Runtime Upgrade)

代码的末尾定义了运行时升级的处理逻辑.`AssetsBridgeMigration`结构体实现了`OnRuntimeUpgrade`特征,用于在运行时升级时执行迁移逻辑.

### API和基准测试(API and Benchmarking)

最后,代码定义了一系列的API,允许外部查询和与区块链交互.此外,如果启用了基准测试功能,代码还包含了基准测试的配置和实现.

整体而言,这段代码是ChainX区块链运行时的完整配置,涵盖了从基本功能到高级特性的各个方面.通过这些配置,ChainX区块链能够实现其设计的功能,并与外部世界进行交互.

-----------------------------------------------------------------------------
账户索引(Account Index)是Substrate框架中的一个概念,它提供了一种机制,允许用户创建与其账户相关联的唯一标识符,
这些标识符可以用于简化交易和通信.在区块链网络中,账户通常是通过公钥或地址来识别的,但这些标识符可能很长且难以记忆.
账户索引提供了一种替代方式,使得用户可以通过一个简短的数字索引来引用账户.

在`pallet_indices`模块中,账户索引被实现和使用.这个模块允许用户通过支付一定的费用来注册一个索引,这个索引将与他们的账户关联.
一旦注册,其他用户就可以使用这个索引来直接与该账户进行交互,而不需要知道完整的公钥或地址.

账户索引的主要用途包括:

1. **简化交易**:用户可以通过索引而不是复杂的公钥或地址来发送交易.
2. **提高可用性**:索引通常是较短的数字,更容易记忆和分享.
3. **链上身份**:用户可以将其索引作为链上身份的一部分,用于社交媒体,市场或其他去中心化应用(DApps).

在ChainX项目的运行时配置中,`pallet_indices`模块被用来定义账户索引的相关配置,如索引的注册费用,最大索引数量等.
通过这种方式,ChainX区块链上的用户可以享受到账户索引带来的便利.

-----------------------------------------------------------------------------
运行时升级(Runtime Upgrade)是指对区块链的运行时环境进行更新或修改的过程.在区块链系统中,
运行时(Runtime)通常指的是智能合约的执行环境,它包括了区块链的核心逻辑,预编译合约,交易处理,账户管理等功能.
运行时升级可以包括修复错误,增加新功能,优化性能或改进安全性等.

在Substrate框架中,运行时升级特别指通过热升级(Hot Upgrade)或硬分叉(Hard Fork)的方式来更新区块链的状态或逻辑.
这些升级可以是非破坏性的,也就是说,它们不会影响现有的链状态,也不会导致区块链分叉;或者是破坏性的,需要所有节点同意并升级到新版本.

### 运行时升级的类型:

1. **热升级(Hot Upgrade)**:
   - 也称为软升级(Soft Fork).
   - 向后兼容,新旧节点可以在同一链上共存.
   - 通常用于修复紧急错误或添加向后兼容的新功能.

2. **硬分叉(Hard Fork)**:
   - 需要所有节点升级到新版本,否则不升级的节点将遵循旧规则,可能导致区块链分叉.
   - 通常用于重大变更,如协议更改或共识机制的更新.

### 运行时升级的过程:

1. **规划**:开发团队规划升级内容,包括新功能,修复的漏洞等.
2. **测试**:在测试网上进行充分的测试,确保新版本无重大缺陷.
3. **发布**:发布新版本的区块链节点软件.
4. **升级**:节点操作者下载并安装新版本,可能需要停机进行升级.
5. **激活**:在预定的时间点,新版本将被激活,所有交易和区块验证将按照新规则进行.

### 运行时升级的挑战:

- **兼容性**:新版本必须与旧版本兼容,或者至少确保升级过程中的数据完整性和链的连续性.
- **共识**:社区和节点操作者需要就升级达成共识,特别是在需要硬分叉的情况下.
- **安全性**:升级过程中可能会引入新的安全风险,因此需要谨慎处理.

在ChainX项目的运行时配置中,`AssetsBridgeMigration`结构体实现了`OnRuntimeUpgrade`特征,
这意味着它定义了在运行时升级时需要执行的特定迁移逻辑.这可能包括清理旧状态,迁移数据到新结构或执行其他必要的状态转换操作.
通过这种方式,ChainX区块链可以平滑地过渡到新版本,同时保持网络的稳定性和安全性.
*/

