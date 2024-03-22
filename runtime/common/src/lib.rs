//! Common runtime code for ChainX.

#![cfg_attr(not(feature = "std"), no_std)]

use static_assertions::const_assert;

use frame_support::{
    parameter_types,
    traits::Currency,
    weights::{constants::WEIGHT_PER_SECOND, DispatchClass, Weight},
};
use frame_system::limits;
use pallet_transaction_payment::{Multiplier, TargetedFeeAdjustment};
use sp_runtime::{FixedPointNumber, Perbill, Perquintill};

use chainx_primitives::BlockNumber;

pub use frame_support::weights::constants::{
    BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight,
};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

pub type NegativeImbalance<T> = <pallet_balances::Pallet<T> as Currency<
    <T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

/// We assume that an on-initialize consumes 2.5% of the weight on average, hence a single extrinsic
/// will not be allowed to consume more than `AvailableBlockRatio - 2.5%`.
pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_perthousand(25);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2 seconds of compute with a 6 second average block time.
pub const MAXIMUM_BLOCK_WEIGHT: Weight = 2 * WEIGHT_PER_SECOND;

const_assert!(NORMAL_DISPATCH_RATIO.deconstruct() >= AVERAGE_ON_INITIALIZE_RATIO.deconstruct());

// Common constants used in all runtimes.
parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    /// The portion of the `NORMAL_DISPATCH_RATIO` that we adjust the fees with. Blocks filled less
    /// than this will decrease the weight and more will increase.
    pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
    /// The adjustment variable of the runtime. Higher values will cause `TargetBlockFullness` to
    /// change the fees more rapidly.
    pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(3, 100_000);
    /// Minimum amount of the multiplier. This value cannot be too low. A test case should ensure
    /// that combined with `AdjustmentVariable`, we can recover from the minimum.
    /// See `multiplier_can_grow_from_zero`.
    pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000u128);
    /// Maximum length of block. Up to 5MB.
    pub BlockLength: limits::BlockLength =
        limits::BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    /// Block weights base values and limits.
    pub BlockWeights: limits::BlockWeights = limits::BlockWeights::builder()
        .base_block(BlockExecutionWeight::get())
        .for_class(DispatchClass::all(), |weights| {
            weights.base_extrinsic = ExtrinsicBaseWeight::get();
        })
        .for_class(DispatchClass::Normal, |weights| {
            weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
        })
        .for_class(DispatchClass::Operational, |weights| {
            weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
            // Operational transactions have an extra reserved space, so that they
            // are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
            weights.reserved = Some(
                MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT,
            );
        })
        .avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
        .build_or_panic();
}

parameter_types! {
    /// A limit for off-chain phragmen unsigned solution submission.
    ///
    /// We want to keep it as high as possible, but can't risk having it reject,
    /// so we always subtract the base block execution weight.
    pub OffchainSolutionWeightLimit: Weight = BlockWeights::get()
        .get(DispatchClass::Normal)
        .max_extrinsic
        .expect("Normal extrinsics have weight limit configured by default; qed")
        .saturating_sub(BlockExecutionWeight::get());
}

/// Parameterized slow adjusting fee updated based on
/// https://w3f-research.readthedocs.io/en/latest/polkadot/Token%20Economics.html#-2.-slow-adjusting-mechanism
pub type SlowAdjustingFeeUpdate<R> =
    TargetedFeeAdjustment<R, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;

/// The type used for currency conversion.
///
/// This must only be used as long as the balance type is u128.
pub type CurrencyToVote = frame_support::traits::U128CurrencyToVote;

// EVM
pub const BASE_FEE: u128 = 50_000_000u128;

/*
这段代码是 ChainX 区块链的共同运行时代码的一部分,它定义了一些关键的参数类型和常量,这些参数和常量对于 ChainX 区块链的运行时配置至关重要.

### 常量定义

- `AVERAGE_ON_INITIALIZE_RATIO`: 假设 `on_initialize` 调用平均消耗区块权重的 0.25%.
- `NORMAL_DISPATCH_RATIO`: 允许 `Normal` 类型的交易填充至区块的 75%.
- `MAXIMUM_BLOCK_WEIGHT`: 最大区块权重,设置为每秒权重的两倍.

### 参数类型定义

- `BlockHashCount`: 区块链保留的区块哈希数量.
- `TargetBlockFullness`: 目标区块饱和度,用于调整交易费用.
- `AdjustmentVariable`: 运行时的调整变量,用于影响费用调整的速度.
- `MinimumMultiplier`: 调整乘数的最小值,确保费用可以恢复到最低值.
- `BlockLength`: 区块的最大长度,设置为 5MB.
- `BlockWeights`: 区块权重的基值和限制.

### 慢速调整费用更新

- `SlowAdjustingFeeUpdate`: 基于目标区块饱和度,调整变量和最小乘数的目标费用调整.

### 货币转换类型

- `CurrencyToVote`: 用于货币转换为投票的类型.

### EVM 相关

- `BASE_FEE`: EVM 交易的基础费用.

这些配置确保了 ChainX 区块链的交易费用和区块权重管理能够按照预定的规则和参数运行.
通过这些配置,ChainX 能够实现有效的区块生产,交易执行和费用调整机制,同时保持网络的稳定性和可靠性.

*/
