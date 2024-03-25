// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use sp_std::vec::Vec;

use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use sp_runtime::{
    traits::{SaturatedConversion, Saturating},
    DispatchError, DispatchResult, RuntimeDebug,
};

use chainx_primitives::{AssetId, ReferralId};
use frame_support::log::debug;
use xp_mining_common::{RewardPotAccountFor, WeightType};
use xp_mining_staking::MiningPower;

use crate::{AssetMining, BalanceOf, Config, EraIndex, Event, Pallet};

pub type VoteWeight = WeightType;

/// Detailed types of reserved balances in Staking.
#[derive(PartialEq, PartialOrd, Ord, Eq, Clone, Copy, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum LockedType {
    /// Locked balances when nominator calls `bond`.
    Bonded,
    /// The locked balances transition from `Bonded` into `BondedWithdrawal` state
    /// when nominator calls `unbond`.
    BondedWithdrawal,
}

/// Destination for minted fresh PCX on each new session.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum MintedDestination<AccountId> {
    Validator(AccountId),
    Asset(AssetId),
}

/// The requirement of a qualified staking candidate.
///
/// If the (potential) validator failed to meet this requirement, force it to be chilled on new election round.
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct BondRequirement<Balance> {
    /// The minimal amount of self-bonded balance to be a qualified validator candidate.
    pub self_bonded: Balance,
    /// The minimal amount of total-bonded balance to be a qualified validator candidate.
    ///
    /// total_bonded = self_bonded + all the other nominators' nominations.
    pub total: Balance,
}

/// Type for noting when the unbonded fund can be withdrawn.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct Unbonded<Balance, BlockNumber> {
    /// Amount of funds to be unlocked.
    pub value: Balance,
    /// Block number at which point it'll be unlocked.
    pub locked_until: BlockNumber,
}

/// Vote weight properties of validator.
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct ValidatorLedger<Balance, VoteWeight, BlockNumber> {
    /// The total amount of all the nominators' vote balances.
    pub total_nomination: Balance,
    /// Last calculated total vote weight of current validator.
    pub last_total_vote_weight: VoteWeight,
    /// Block number at which point `last_total_vote_weight` just updated.
    pub last_total_vote_weight_update: BlockNumber,
}

/// Vote weight properties of nominator.
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct NominatorLedger<Balance, VoteWeight, BlockNumber> {
    /// The amount of vote.
    pub nomination: Balance,
    /// Last calculated total vote weight of current nominator.
    pub last_vote_weight: VoteWeight,
    /// Block number at which point `last_vote_weight` just updated.
    pub last_vote_weight_update: BlockNumber,
    /// Unbonded entries.
    pub unbonded_chunks: Vec<Unbonded<Balance, BlockNumber>>,
}

/// Profile of staking validator.
///
/// These fields are static or updated less frequently.
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct ValidatorProfile<BlockNumber> {
    /// Block number at which point it's registered on chain.
    pub registered_at: BlockNumber,
    /// Validator is chilled right now.
    ///
    /// Declared no desire to be a validator or forced to be chilled due to `MinimumCandidateThreshold`.
    pub is_chilled: bool,
    /// Block number of last performed `chill` operation.
    pub last_chilled: Option<BlockNumber>,
    /// Referral identity that belongs to the validator.
    #[cfg_attr(feature = "std", serde(with = "xp_rpc::serde_text"))]
    pub referral_id: ReferralId,
}

/// Information regarding the active era (era in used in session).
#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ActiveEraInfo {
    /// Index of era.
    pub index: EraIndex,
    /// Moment of start expressed as millisecond from `$UNIX_EPOCH`.
    ///
    /// Start can be none if start hasn't been set for the era yet,
    /// Start is set on the first on_finalize of the era to guarantee usage of `Time`.
    pub start: Option<u64>,
}

/// Mode of era-forcing.
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Forcing {
    /// Not forcing anything - just let whatever happen.
    NotForcing,
    /// Force a new era, then reset to `NotForcing` as soon as it is done.
    ForceNew,
    /// Avoid a new era indefinitely.
    ForceNone,
    /// Force a new era at the end of all sessions indefinitely.
    ForceAlways,
}

impl Default for Forcing {
    fn default() -> Self {
        Forcing::NotForcing
    }
}

/// Top level shares of various reward destinations.
#[derive(Copy, Clone, PartialEq, Eq, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct GlobalDistribution {
    pub treasury: u32,
    pub mining: u32,
}

impl GlobalDistribution {
    /// Calculates the rewards for treasury and mining accordingly.
    pub fn calc_rewards<T: Config>(&self, reward: BalanceOf<T>) -> (BalanceOf<T>, BalanceOf<T>) {
        assert!(self.treasury + self.mining > 0);
        let treasury_reward = reward * self.treasury.saturated_into()
            / (self.treasury + self.mining).saturated_into();
        (treasury_reward, reward - treasury_reward)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct MiningDistribution {
    pub asset: u32,
    pub staking: u32,
}

impl MiningDistribution {
    /// Returns the reward for Staking given the total reward according to the Staking proportion.
    pub fn calc_staking_reward<T: Config>(&self, reward: BalanceOf<T>) -> BalanceOf<T> {
        reward.saturating_mul(self.staking.saturated_into())
            / (self.asset + self.staking).saturated_into()
    }

    /// Return a tuple (m1, m2) for comparing whether asset_mining_power are reaching the upper limit.
    ///
    /// If m1 >= m2, the asset mining cap has reached, all the reward calculated by the shares go to
    /// the mining assets, but its unit mining power starts to decrease compared to the inital FixedPower.
    fn asset_mining_vs_staking<T: Config>(&self) -> (u128, u128) {
        let total_staking_power =
            crate::Pallet::<T>::total_staked().saturated_into::<MiningPower>();
        let total_asset_mining_power = T::AssetMining::total_asset_mining_power();

        // When:
        //
        //  total_asset_mining_power     1(asset_mining_shares)
        //  ------------------------ >= -----------------------
        //     total_staking_power         9(staking_shares)
        //
        //  i.e., m1 >= m2,
        //
        // there is no extra treasury split, otherwise the difference will
        // be distruted to the treasury account again.
        let m1 = total_asset_mining_power * u128::from(self.staking);
        let m2 = total_staking_power * u128::from(self.asset);

        (m1, m2)
    }

    pub fn has_treasury_extra<T: Config>(
        &self,
        asset_mining_reward_cap: BalanceOf<T>,
    ) -> Option<BalanceOf<T>> {
        let (m1, m2) = self.asset_mining_vs_staking::<T>();
        if m1 >= m2 {
            debug!(
                target: "runtime::mining::staking",
                "[has_treasury_extra] m1({}) >= m2({}), no extra treasury split.",
                m1, m2
            );
            None
        } else {
            assert!(
                m2 > 0,
                "asset_mining_shares is ensured to be positive in set_distribution_ratio()"
            );
            // There could be some computation loss here, but it's ok.
            let treasury_extra = (m2 - m1) * asset_mining_reward_cap.saturated_into::<u128>() / m2;
            Some(treasury_extra.saturated_into::<BalanceOf<T>>())
        }
    }
}

/// Result of performing a slash operation.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum SlashOutcome<Balance> {
    /// Succeeded in slashing the reward pot given the slash value.
    Slashed(Balance),
    /// The reward pot does not have enough balances to pay the slash,
    /// the whole reward pot will just be slashed.
    InsufficientSlash(Balance),
    /// Somehow can not transfer from the reward pot to the treasury account.
    SlashFailed(DispatchError),
}

/// Struct for performing the slash.
///
/// Abstracted for caching the treasury account.
#[derive(Copy, Clone, PartialEq, Eq, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Slasher<T: Config>(T::AccountId);

impl<T: Config> Slasher<T> {
    pub fn new(treasury_account: T::AccountId) -> Self {
        Self(treasury_account)
    }

    /// Try to slash the reward pot of the offender.
    ///
    /// If the reward pot of offender has enough balance to cover the slashing,
    /// otherwise slash the reward pot as much as possible and returns the value actually slashed.
    pub fn try_slash(
        &self,
        offender: &T::AccountId,
        expected_slash: BalanceOf<T>,
    ) -> SlashOutcome<BalanceOf<T>> {
        let reward_pot = T::DetermineRewardPotAccount::reward_pot_account_for(offender);
        let reward_pot_balance = Pallet::<T>::free_balance(&reward_pot);

        debug!(
            target: "runtime::mining::staking",
            "[try_slash] reward_pot_balance:{:?}, expected_slash:{:?}",
            reward_pot_balance, expected_slash
        );

        let (actual_slash, is_insufficient_slash) = if expected_slash <= reward_pot_balance {
            (expected_slash, false)
        } else {
            (reward_pot_balance, true)
        };

        if let Err(e) = self.do_slash(&reward_pot, actual_slash) {
            SlashOutcome::SlashFailed(e)
        } else {
            Pallet::<T>::deposit_event(Event::<T>::Slashed(offender.clone(), actual_slash));
            if is_insufficient_slash {
                SlashOutcome::InsufficientSlash(actual_slash)
            } else {
                SlashOutcome::Slashed(actual_slash)
            }
        }
    }

    /// Actually slash the account being punished, all slashed balance will go to the treasury.
    fn do_slash(&self, reward_pot: &T::AccountId, value: BalanceOf<T>) -> DispatchResult {
        Pallet::<T>::transfer(reward_pot, &self.0, value)
    }
}

/*
这段代码是ChainX区块链项目中质押(Staking)模块的一部分,它定义了与质押相关的多种数据结构和类型.
这些类型用于管理质押的资产,奖励分配,惩罚机制等.下面是对这些类型的详细解释:

1. **LockedType**: 表示质押系统中锁定余额的类型,包括`Bonded`(质押中)和`BondedWithdrawal`(解质押过渡状态).

2. **MintedDestination**: 定义了每个新会话中新铸造的PCX的目的地,可以是验证者或资产.

3. **BondRequirement**: 定义了成为合格质押候选人所需的最低自质押余额和总质押余额要求.

4. **Unbonded**: 表示解质押资金的结构,包含解锁金额,锁定直到的区块号.

5. **ValidatorLedger**: 记录验证者的投票权重和相关信息,如总提名余额,最后计算的总投票权重等.

6. **NominatorLedger**: 记录提名者的投票权重和相关信息,如提名金额,最后投票权重等.

7. **ValidatorProfile**: 包含验证者的静态或更新频率较低的信息,如注册区块号,是否处于冷却状态等.

8. **ActiveEraInfo**: 包含活动纪元的索引和开始时间.

9. **Forcing**: 表示纪元强制模式的枚举,包括不强制,强制新纪元,无限期避免新纪元和始终强制新纪元.

10. **GlobalDistribution**: 定义了奖励分配给国库和挖矿的全局比例.

11. **MiningDistribution**: 定义了挖矿奖励在资产挖矿和质押挖矿之间的分配比例.

12. **SlashOutcome**: 表示执行惩罚(Slash)操作的结果,包括成功惩罚,惩罚不足和惩罚失败.

13. **Slasher**: 用于执行惩罚操作的结构,包含尝试惩罚和实际执行惩罚的方法.

这些类型是ChainX质押模块的核心组成部分,它们确保了质押系统的正常运行,包括资产的锁定和解锁,奖励的分配以及对
不当行为的惩罚.代码中还包含了一些默认实现和辅助函数,用于简化和处理与这些类型相关的操作.
*/
