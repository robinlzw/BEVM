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
这段代码是一个Rust模块,它定义了ChainX项目运行时使用的一组常量值.ChainX是一个基于Substrate框架的区块链项目.
这个模块被分为三个子模块:`currency`,`time`和`fee`,每个子模块都包含了与其名称相关的不同类型的常量和函数.

### currency 模块

`currency`模块定义了一些与ChainX项目中的货币单位相关的常量.这些常量包括:

- `PCXS`:项目的最小货币单位,设置为100,000,000,具有8位小数.
- `DOLLARS`:1美元在ChainX货币体系中的等价值,等于`PCXS / 100`.
- `CENTS`:1美分在ChainX货币体系中的等价值,等于`DOLLARS / 100`.
- `MILLICENTS`:1毫美分在ChainX货币体系中的等价值,等于`CENTS / 1,000`.

此外,还有一个`deposit`函数,用于计算存入ChainX网络的费用.该函数接受两个参数:`items`和`bytes`,
分别代表项目的数量和字节大小,并返回一个`Balance`类型的值,表示总费用.

### time 模块

`time`模块定义了一些与时间相关的常量,用于ChainX项目的区块链时间管理:

- `MILLISECS_PER_BLOCK`:每个区块的预期毫秒数,设置为6,000.
- `SLOT_DURATION`:插槽持续时间,与`MILLISECS_PER_BLOCK`相同.
- `EPOCH_DURATION_IN_BLOCKS`:一个时代(epoch)包含的区块数量,设置为5分钟.
- `MINUTES`,`HOURS`,`DAYS`:分别代表分钟,小时和天数对应的区块数量,基于`MILLISECS_PER_BLOCK`计算得出.
- `PRIMARY_PROBABILITY`:Babe共识算法中,一个区块成为主区块的概率,设置为1/4.

### fee 模块

`fee`模块定义了一些与交易费用相关的常量和类型.这些常量和类型用于计算和转换交易费用:

- `TARGET_BLOCK_FULLNESS`:区块饱和度的目标百分比,设置为25%.
- `WeightToFee`:一个结构体,实现了`WeightToFeePolynomial`特征,用于将交易的权重转换为费用.
它定义了一个多项式,将权重映射到费用上.在`polynomial`方法中,定义了一个一次多项式,其中`CENTS`作为常数项,
`ExtrinsicBaseWeight::get()`返回的权重值乘以10后作为一次项的系数.

这个模块的目的是为ChainX区块链提供一个标准化的费用计算方法,确保交易费用的合理性和一致性.
通过定义这些常量和函数,ChainX项目能够确保其经济模型的稳定性和可预测性.

*/