// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

//! A set of constant values used in chainx runtime.

/// Money matters.
pub mod currency {
    use chainx_primitives::Balance;

    pub const PCXS: Balance = 100_000_000; // 8 decimals
    pub const DOLLARS: Balance = PCXS / 100; // 1000_000
    pub const CENTS: Balance = DOLLARS / 100; // 10_000
    pub const MILLICENTS: Balance = CENTS / 1_000; // 10

    pub const fn deposit(items: u32, bytes: u32) -> Balance {
        items as Balance * 20 * DOLLARS + (bytes as Balance) * 100 * MILLICENTS
    }
}

/// Time.
pub mod time {
    use chainx_primitives::{BlockNumber, Moment};

    pub const MILLISECS_PER_BLOCK: Moment = 6000;
    pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;
    pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 5 * MINUTES;

    // These time units are defined in number of blocks.
    pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
    pub const HOURS: BlockNumber = MINUTES * 60;
    pub const DAYS: BlockNumber = HOURS * 24;

    // 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
    pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);
}

/// Fee-related.
pub mod fee {
    use frame_support::weights::{
        constants::ExtrinsicBaseWeight, WeightToFeeCoefficient, WeightToFeeCoefficients,
        WeightToFeePolynomial,
    };
    use smallvec::smallvec;
    pub use sp_runtime::Perbill;

    use chainx_primitives::Balance;

    /// The block saturation level. Fees will be updates based on this value.
    pub const TARGET_BLOCK_FULLNESS: Perbill = Perbill::from_percent(25);

    /// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
    /// node's balance type.
    ///
    /// This should typically create a mapping between the following ranges:
    ///   - [0, frame_system::MaximumBlockWeight]
    ///   - [Balance::min, Balance::max]
    ///
    /// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
    ///   - Setting it to `0` will essentially disable the weight fee.
    ///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
    pub struct WeightToFee;
    impl WeightToFeePolynomial for WeightToFee {
        type Balance = Balance;
        fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
            // in Polkadot, extrinsic base weight (smallest non-zero weight) is mapped to 1/10 CENT:
            let p = super::currency::CENTS;
            let q = 10 * Balance::from(ExtrinsicBaseWeight::get());
            smallvec![WeightToFeeCoefficient {
                degree: 1,
                negative: false,
                coeff_frac: Perbill::from_rational(p % q, q),
                coeff_integer: p / q,
            }]
        }
    }
}

/*
这段代码定义了 ChainX 区块链运行时中使用的一组常量值,这些常量分布在三个模块中:货币(currency),时间(time)和费用(fee).

### 货币(currency)模块

- `PCXS`:ChainX 区块链的最小货币单位,设置为 100,000,000,具有 8 位小数.
- `DOLLARS`:1 美元在 ChainX 货币体系中的等价值,等于 `PCXS / 100`.
- `CENTS`:1 美分在 ChainX 货币体系中的等价值,等于 `DOLLARS / 100`.
- `MILLICENTS`:1 毫美分在 ChainX 货币体系中的等价值,等于 `CENTS / 1,000`.
- `deposit` 函数:计算存入 ChainX 网络的费用,基于项目的数量和字节大小.

### 时间(time)模块

- `MILLISECS_PER_BLOCK`:每个区块的预期毫秒数,设置为 6,000.
- `SLOT_DURATION`:插槽持续时间,与 `MILLISECS_PER_BLOCK` 相同.
- `EPOCH_DURATION_IN_BLOCKS`:一个时代(epoch)包含的区块数量,设置为 5 分钟.
- `MINUTES`,`HOURS`,`DAYS`:分别代表分钟,小时和天数对应的区块数量,基于 `MILLISECS_PER_BLOCK` 计算得出.
- `PRIMARY_PROBABILITY`:Babe 共识算法中,一个区块成为主区块的概率,设置为 1/4.

### 费用(fee)模块

- `TARGET_BLOCK_FULLNESS`:区块饱和度的目标百分比,设置为 25%.
- `WeightToFee` 结构:实现了 `WeightToFeePolynomial` trait,用于将权重转换为费用.
- `polynomial` 方法:定义了一个一次多项式,将权重映射到费用上.在 Polkadot 中,最小的非零权重(即 extrinsic base weight)映射到 1/10 美分.

这些常量和模块为 ChainX 区块链提供了货币单位,时间单位和费用计算的基础,它们对于区块链的运行和交易费用系统至关重要.
通过这些定义,ChainX 能够确保其经济模型的稳定性和可预测性.
*/
