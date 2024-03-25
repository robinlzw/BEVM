// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

//! New minted PCX distribution logic for ChainX Proposal 09.

use super::*;
use sp_std::vec::Vec;

impl<T: Config> Pallet<T> {
    fn generic_calculate_by_proportion<S: Into<u128>>(
        total_reward: BalanceOf<T>,
        mine: S,
        total: S,
    ) -> BalanceOf<T> {
        let mine: u128 = mine.saturated_into();
        let total: u128 = total.saturated_into();

        match mine.checked_mul(total_reward.saturated_into()) {
            Some(x) => {
                let r = x / total;
                assert!(
                    r < u128::from(u64::max_value()),
                    "reward of per validator definitely less than u64::MAX"
                );
                r.saturated_into::<BalanceOf<T>>()
            }
            None => panic!("stake * session_reward overflow!"),
        }
    }

    /// Calculates the individual reward according to the proportion and total reward.
    fn calc_individual_staking_reward(
        total_reward: BalanceOf<T>,
        my_stake: BalanceOf<T>,
        total_stake: BalanceOf<T>,
    ) -> BalanceOf<T> {
        let mine = my_stake.saturated_into::<u128>();
        let total = total_stake.saturated_into::<u128>();
        Self::generic_calculate_by_proportion(total_reward, mine, total)
    }

    fn calc_invididual_asset_mining_reward(
        total_reward: BalanceOf<T>,
        my_power: u128,
        total_power: u128,
    ) -> BalanceOf<T> {
        Self::generic_calculate_by_proportion(total_reward, my_power, total_power)
    }

    /// Distributes the invididual asset mining reward, returns the unpaid asset mining rewards.
    fn distribute_to_mining_assets(total_reward: BalanceOf<T>) -> BalanceOf<T> {
        let asset_mining_info = T::AssetMining::asset_mining_power();

        // [PASS*] No risk of sum overflow practically.
        //        u128::MAX / u128::from(u64::max_value()) / u128::from(u32::max_value())
        //      = 4294967297 > u32::MAX = 4294967295
        //        which means even we have u32::MAX mining assets and each power of them
        //        is u32::MAX, the computation of sum() here won't overflow.
        let mut total_power: u128 = asset_mining_info.iter().map(|(_, power)| power).sum();

        let mut total_reward = total_reward;

        for (asset_id, power) in asset_mining_info.into_iter() {
            if !total_power.is_zero() {
                let reward =
                    Self::calc_invididual_asset_mining_reward(total_reward, power, total_power);
                T::AssetMining::reward(asset_id, reward);
                total_power -= power;
                total_reward -= reward;
            }
        }

        total_reward
    }

    /// Reward to all the active validators pro rata.
    fn distribute_to_active_validators(
        session_reward: BalanceOf<T>,
    ) -> Vec<(T::AccountId, BalanceOf<T>)> {
        let current_validators: Vec<(T::AccountId, BalanceOf<T>)> =
            T::SessionInterface::validators()
                .into_iter()
                .filter(|v| Self::is_active(v))
                .map(|v| {
                    let total_votes = Self::total_votes_of(&v);
                    (v, total_votes)
                })
                .collect();

        let mut total_stake = current_validators
            .iter()
            .fold(Zero::zero(), |acc: BalanceOf<T>, (_, x)| acc + *x);
        let mut total_reward = session_reward;
        current_validators
            .into_iter()
            .filter_map(|(validator, stake)| {
                // May become zero after meeting the last one.
                if !total_stake.is_zero() {
                    let reward =
                        Self::calc_individual_staking_reward(total_reward, stake, total_stake);
                    Self::reward_active_validator(&validator, reward);
                    total_stake -= stake;
                    total_reward -= reward;
                    Some((validator, reward))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Issue new PCX to the action intentions and cross mining asset entities
    /// accroding to DistributionRatio.
    fn distribute_mining_rewards(
        total: BalanceOf<T>,
        treasury_account: &T::AccountId,
    ) -> Vec<(T::AccountId, BalanceOf<T>)> {
        let mining_distribution = Self::mining_distribution_ratio();
        let staking_reward = mining_distribution.calc_staking_reward::<T>(total);
        let max_asset_mining_reward = total - staking_reward;

        let validator_rewards = Self::distribute_to_active_validators(staking_reward);

        let real_asset_mining_reward = if let Some(treasury_extra) =
            mining_distribution.has_treasury_extra::<T>(max_asset_mining_reward)
        {
            Self::mint(treasury_account, treasury_extra);
            max_asset_mining_reward - treasury_extra
        } else {
            max_asset_mining_reward
        };

        let unpaid_asset_mining_reward =
            Self::distribute_to_mining_assets(real_asset_mining_reward);
        if !unpaid_asset_mining_reward.is_zero() {
            debug!(
                target: "runtime::mining::staking",
                "[distribute_mining_rewards] unpaid_asset_mining_reward:{:?}",
                unpaid_asset_mining_reward
            );
            Self::mint(treasury_account, unpaid_asset_mining_reward);
        }

        validator_rewards
    }

    /// Main minting logic.
    ///
    /// Returns the reward balance minted specifically for Staking.
    pub(super) fn distribute_session_reward_impl_09(
        session_reward: BalanceOf<T>,
    ) -> Vec<(T::AccountId, BalanceOf<T>)> {
        let global_distribution = Self::global_distribution_ratio();
        let (treasury_reward, mining_reward) =
            global_distribution.calc_rewards::<T>(session_reward);

        // -> Treasury
        let treasury_account =
            T::TreasuryAccount::treasury_account().expect("TreasuryAccount is some; qed");
        if !treasury_reward.is_zero() {
            Self::mint(&treasury_account, treasury_reward);
        }

        // -> Mining
        //      |-> XBTC(Asset Mining)
        //      |-> PCX(Staking)
        if !mining_reward.is_zero() {
            return Self::distribute_mining_rewards(mining_reward, &treasury_account);
        }

        Default::default()
    }
}

/*
这段代码是ChainX区块链项目中实现新发行的PCX(ChainX的原生代币)分配逻辑的一部分,特别是针对ChainX提案09的实现.
代码中定义了一系列函数,用于计算和分配挖矿奖励,包括质押奖励和资产挖矿奖励.以下是代码的主要组成部分和它们的功能:

1. **`generic_calculate_by_proportion`函数**:
   - 一个通用函数,用于根据比例和总奖励计算个体奖励.

2. **`calc_individual_staking_reward`函数**:
   - 计算个体质押奖励,基于用户的质押量和总质押量.

3. **`calc_invididual_asset_mining_reward`函数**:
   - 计算个体资产挖矿奖励,基于用户的挖矿权重和总挖矿权重.

4. **`distribute_to_mining_assets`函数**:
   - 将挖矿奖励分配给所有挖矿资产,返回未分配的资产挖矿奖励.

5. **`distribute_to_active_validators`函数**:
   - 将质押奖励分配给所有活跃的验证者,按其质押量比例分配.

6. **`distribute_mining_rewards`函数**:
   - 根据分配比例,将新发行的PCX分配给行动意图和跨链挖矿资产实体.

7. **`distribute_session_reward_impl_09`函数**:
   - 主要的挖矿逻辑函数,用于在一个会话结束后分配奖励.它首先将奖励分配给国库,然后将剩余的奖励分配给挖矿实体,包括质押和资产挖矿.

整体来看,这段代码为ChainX区块链提供了一个完整的奖励分配机制,确保了新发行的代币能够根据预设的分配比例公平地分配给质押者和挖矿资产.
这有助于激励用户参与到ChainX的质押和挖矿活动中,从而增强网络的安全性和去中心化程度.此外,代码中的事件记录功能允许区块链的其他部分跟踪和审计奖励的分配情况.
*/
