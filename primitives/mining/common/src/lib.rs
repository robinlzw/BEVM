// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

#![cfg_attr(not(feature = "std"), no_std)]

//! Common concepts with regard to the ChainX Mining system, particularly the user-level ones.
//!
//! There are two approaches of mining in ChainX:
//!
//! 1. As a PoS-based blockchain, **Staking** is inherently the fundamental way of mining.
//! In this way, users(stakers) nominate some validators with some balances locked, earning
//! the staking reward.
//!
//! 2. One goal of ChainX is to embrace more the eixsting cryptocurrencies into one ecosystem,
//! therefore **Asset Mining** is introduced for winning more external assets like BTC, ETH, etc.
//! For example, Bitcoin users can deposit their BTC into ChainX, then they'll get the X_BTC
//! in 1:1 and the mining rights in ChainX system accordingly, earning the new minted PCX
//! like the stakers in Staking.
//!
//! Both of these two approaches share one same rule when calculating the individual reward, i.e.,
//! **time-sensitive weight calculation**.
//!
//! ```no_compile
//! Amount(Balance) * Duration(BlockNumber) = Weight
//! ```
//!
//! For Staking:
//!
//! ```no_compile
//! staked_balance(Balance) * time(BlockNumber) = vote_weight
//! ```
//!
//! All the nominators split the reward of the validator's reward pot according to the proportion of vote weight.
//!
//! For Asset Mining:
//!
//! ```no_compile
//! ext_asset_balance(Balance) * time(BlockNumber) = ext_mining_weight
//! ```
//!
//! All asset miners split the reward of asset's reward pot according to the proportion of asset mining weight.
//!

use sp_arithmetic::traits::{BaseArithmetic, SaturatedConversion};
use sp_runtime::RuntimeDebug;

/// Type for calculating the mining weight.
pub type WeightType = u128;

/// The getter and setter methods for the further mining weight processing.
pub trait BaseMiningWeight<Balance, BlockNumber> {
    fn amount(&self) -> Balance;
    /// Set the new amount.
    ///
    /// Amount management of asset miners is handled by assets module,
    /// hence the default implementation is provided here.
    fn set_amount(&mut self, _new: Balance) {}

    fn last_acum_weight(&self) -> WeightType;
    fn set_last_acum_weight(&mut self, s: WeightType);

    fn last_acum_weight_update(&self) -> BlockNumber;
    fn set_last_acum_weight_update(&mut self, num: BlockNumber);
}

/// Amount changes of miner's state.
///
/// `Zero` happens:
/// 1. stakers performs the `rebond` operation.
/// 2. claim the reward.
#[derive(Clone, Copy, RuntimeDebug)]
pub enum Delta<Balance> {
    Add(Balance),
    Sub(Balance),
    Zero,
}

impl<Balance: BaseArithmetic> Delta<Balance> {
    /// Calculates `value` + `self` and returns the calculated value.
    pub fn calculate(self, value: Balance) -> Balance {
        match self {
            Delta::Add(v) => value + v,
            Delta::Sub(v) => value - v,
            Delta::Zero => value,
        }
    }
}

/// General logic for state changes of the mining weight operations.
pub trait MiningWeight<Balance: BaseArithmetic + Copy, BlockNumber>:
    BaseMiningWeight<Balance, BlockNumber>
{
    /// Set the new amount after settling the change of nomination.
    fn settle_and_set_amount(&mut self, delta: &Delta<Balance>) {
        let new = match *delta {
            Delta::Add(x) => self.amount() + x,
            Delta::Sub(x) => self.amount() - x,
            Delta::Zero => return,
        };
        self.set_amount(new);
    }

    /// This action doesn't involve in the change of amount.
    ///
    /// Used for asset mining module.
    fn set_state_weight(&mut self, latest_acum_weight: WeightType, current_block: BlockNumber) {
        self.set_last_acum_weight(latest_acum_weight);
        self.set_last_acum_weight_update(current_block);
    }

    /// Set new state on bond, unbond and rebond in Staking.
    fn set_state(
        &mut self,
        latest_acum_weight: WeightType,
        current_block: BlockNumber,
        delta: &Delta<Balance>,
    ) {
        self.set_state_weight(latest_acum_weight, current_block);
        self.settle_and_set_amount(delta);
    }
}

impl<Balance: BaseArithmetic + Copy, BlockNumber, T: BaseMiningWeight<Balance, BlockNumber>>
    MiningWeight<Balance, BlockNumber> for T
{
}

/// Skips the next processing when the latest mining weight is zero.
pub struct ZeroMiningWeightError;

/// General logic for calculating the latest mining weight.
pub trait ComputeMiningWeight<AccountId, BlockNumber: Copy> {
    /// The entity that holds the funds of claimers.
    type Claimee;
    type Error: From<ZeroMiningWeightError>;

    fn claimer_weight_factors(_: &AccountId, _: &Self::Claimee, _: BlockNumber) -> WeightFactors;
    fn claimee_weight_factors(_: &Self::Claimee, _: BlockNumber) -> WeightFactors;

    fn settle_claimer_weight(
        who: &AccountId,
        target: &Self::Claimee,
        current_block: BlockNumber,
    ) -> WeightType {
        Self::_calc_latest_vote_weight(Self::claimer_weight_factors(who, target, current_block))
    }

    fn settle_claimee_weight(target: &Self::Claimee, current_block: BlockNumber) -> WeightType {
        Self::_calc_latest_vote_weight(Self::claimee_weight_factors(target, current_block))
    }

    fn settle_weight_on_claim(
        who: &AccountId,
        target: &Self::Claimee,
        current_block: BlockNumber,
    ) -> Result<(WeightType, WeightType), Self::Error> {
        let claimer_weight = Self::settle_claimer_weight(who, target, current_block);

        if claimer_weight == 0 {
            return Err(ZeroMiningWeightError.into());
        }

        let claimee_weight = Self::settle_claimee_weight(target, current_block);

        Ok((claimer_weight, claimee_weight))
    }

    fn _calc_latest_vote_weight(weight_factors: WeightFactors) -> WeightType {
        let (last_acum_weight, amount, duration) = weight_factors;
        last_acum_weight + amount * duration
    }

    /// Computes the dividend according to the latest mining weight proportion.
    fn compute_dividend<Balance: BaseArithmetic>(
        claimer: &AccountId,
        claimee: &Self::Claimee,
        current_block: BlockNumber,
        reward_pot_balance: Balance,
    ) -> Result<(Balance, WeightType, WeightType), Self::Error> {
        // 1. calculates the latest mining weight.
        let (source_weight, target_weight) =
            Self::settle_weight_on_claim(claimer, claimee, current_block)?;

        // 2. calculates the dividend by the mining weight proportion.
        let dividend = compute_dividend::<AccountId, Balance>(
            source_weight,
            target_weight,
            reward_pot_balance,
        );

        Ok((dividend, source_weight, target_weight))
    }
}

/// Weight Formula:
///
/// LatestVoteWeight(WeightType) = last_acum_weight(WeightType) + amount(Balance) * duration(BlockNumber)
///
/// Using u128 for calculating the weights won't run into the overflow issue practically.
pub type WeightFactors = (WeightType, u128, u128);

/// Prepares the factors for calculating the latest mining weight.
pub fn generic_weight_factors<
    Balance: BaseArithmetic,
    BlockNumber: BaseArithmetic,
    W: BaseMiningWeight<Balance, BlockNumber>,
>(
    w: W,
    current_block: BlockNumber,
) -> WeightFactors {
    (
        w.last_acum_weight(),
        w.amount().saturated_into(),
        (current_block - w.last_acum_weight_update()).saturated_into(),
    )
}

/// Computes the dividend according to the ratio of source_vote_weight/target_vote_weight.
///
/// dividend = source_vote_weight/target_vote_weight * balance_of(claimee_reward_pot)
pub fn compute_dividend<AccountId, Balance: BaseArithmetic>(
    source_vote_weight: WeightType,
    target_vote_weight: WeightType,
    reward_pot_balance: Balance,
) -> Balance {
    match source_vote_weight.checked_mul(reward_pot_balance.saturated_into()) {
        Some(x) => (x / target_vote_weight).saturated_into(),
        None => panic!("source_vote_weight * total_reward_pot overflow, this should not happen"),
    }
}

/// Claims the reward for participating in the mining.
pub trait Claim<AccountId> {
    /// Entity of holder of individual miners.
    ///
    /// Validator for Staking, Asset for Asset Mining.
    type Claimee;
    /// Claim error type.
    type Error;

    fn claim(claimer: &AccountId, claimee: &Self::Claimee) -> Result<(), Self::Error>;
}

/// A function that generates an `AccountId` for the reward pot of a mining entity.
///
/// The reward of all individual miners will be staged in the reward pot, the individual
/// reward can be claimed manually at any time.
pub trait RewardPotAccountFor<AccountId, MiningEntity> {
    /// Entity of the mining participant.
    ///
    /// The entity can be a Staking Validator or a Mining Asset.
    fn reward_pot_account_for(_entity: &MiningEntity) -> AccountId;
}

impl<AccountId: Default, MiningEntity> RewardPotAccountFor<AccountId, MiningEntity> for () {
    fn reward_pot_account_for(_entity: &MiningEntity) -> AccountId {
        Default::default()
    }
}

/*
这段代码是ChainX区块链项目中与挖矿系统相关的一些通用概念和逻辑的实现.ChainX的挖矿系统包括两种主要的挖矿方式:
基于权益证明(PoS)的质押(Staking)和资产挖矿(Asset Mining).

这段注释是ChainX区块链项目中关于其挖矿系统用户层面的通用概念的描述.ChainX的挖矿系统有两种主要的方法:

1. **质押(Staking)**:
   - 作为一种基于权益证明(Proof of Stake, PoS)的区块链,质押是ChainX挖矿的基本方式.
   - 在质押中,用户(质押者)通过锁定一部分余额来提名一些验证者,从而赚取质押奖励.

2. **资产挖矿(Asset Mining)**:
   - ChainX的目标是将更多的现有加密货币纳入一个生态系统,因此引入了资产挖矿,以赢得如BTC,ETH等外部资产.
   - 例如,比特币用户可以将他们的BTC存入ChainX,然后他们将按1:1的比例获得X_BTC,
   并根据ChainX系统中的挖矿权利赚取新铸造的PCX,就像在质押中的质押者一样.

这两种方法在计算个人奖励时都遵循相同的规则,即**时间敏感的权重计算**.具体的计算公式如下:

- 对于质押:`质押余额(Balance) * 时间(BlockNumber) = 投票权重(Weight)`
- 对于资产挖矿:`外部资产余额(Balance) * 时间(BlockNumber) = 外部挖矿权重(Weight)`

在质押中,所有提名者根据投票权重的比例来分配验证者的奖励池.在资产挖矿中,所有资产矿工根据资产挖矿权重的比例来分配资产的奖励池.

这种设计允许ChainX在保持其PoS挖矿机制的同时,还能够吸引和整合其他区块链资产,为用户提供更多样化的挖矿机会和收益来源.
通过这种方式,ChainX旨在创建一个更加包容和互联的加密货币生态系统.


以下是对代码中各个部分的详细解释:

1. **挖矿权重(WeightType)**:
   - 描述:用于计算挖矿权重的类型,这里定义为`u128`.

2. **BaseMiningWeight trait**:
   - 描述:提供了挖矿权重处理的基本getter和setter方法.
   - 方法:包括获取和设置挖矿金额,获取和设置最后累积权重,获取和设置最后累积权重更新的区块号.

3. **Delta枚举**:
   - 描述:表示矿工状态金额变化的类型,包括增加(Add),减少(Sub)和归零(Zero).

4. **MiningWeight trait**:
   - 描述:提供了挖矿权重操作的通用逻辑,包括处理提名变更后的金额更新,资产挖矿模块的状态权重设置,
   以及在质押中的邦定,解邦和重新邦定状态设置.

5. **ComputeMiningWeight trait**:
   - 描述:定义了计算挖矿权重的逻辑,包括计算索赔者和索赔对象的权重因子,解决索赔时的权重,
   计算最新挖矿权重,以及根据挖矿权重比例计算分红.

6. **WeightFactors类型**:
   - 描述:表示计算最新挖矿权重的公式中的因子,包括最后累积权重,金额和持续时间.

7. **generic_weight_factors函数**:
   - 描述:根据给定的挖矿权重实例,当前区块号准备计算最新挖矿权重所需的因子.

8. **compute_dividend函数**:
   - 描述:根据源投票权重和目标投票权重的比例计算分红.

9. **Claim trait**:
   - 描述:定义了索赔挖矿奖励的逻辑,包括索赔者和索赔对象的类型以及索赔错误类型.

10. **RewardPotAccountFor trait**:
    - 描述:提供了为挖矿实体生成奖励池账户ID的逻辑.

整体而言,这段代码为ChainX区块链提供了挖矿权重计算和管理的机制,确保了挖矿奖励能够根据参与者的权重公平分配.
通过这些trait和函数,ChainX能够实现复杂的挖矿和奖励逻辑,为用户和验证者提供灵活的挖矿参与方式.
*/