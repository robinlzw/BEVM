// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use sp_std::ops::Mul;
use sp_std::vec::Vec;

use super::*;

impl<T: Config> Pallet<T> {
    /// Returns the force chilled offenders if any after applying the slashings.
    ///
    /// The slashed balances will be moved to the treasury.
    pub(crate) fn slash_offenders_in_session(
        offenders: BTreeMap<T::AccountId, Perbill>,
        validator_rewards: Vec<(T::AccountId, BalanceOf<T>)>,
    ) -> Vec<T::AccountId> {
        let validator_rewards = validator_rewards.into_iter().collect::<BTreeMap<_, _>>();

        let treasury_account =
            T::TreasuryAccount::treasury_account().expect("TreasuryAccount is some; qed");
        let slasher = Slasher::<T>::new(treasury_account);

        let minimum_penalty = Self::minimum_penalty();
        let calc_base_slash = |offender: &T::AccountId, slash_fraction: Perbill| {
            // https://github.com/paritytech/substrate/blob/c60f00840034017d4b7e6d20bd4fcf9a3f5b529a/frame/im-online/src/lib.rs#L773
            // slash_fraction is zero when <10% offline, in which case we still apply a minimum_penalty.
            if slash_fraction.is_zero() {
                minimum_penalty
            } else {
                let pot = Self::reward_pot_for(offender);
                slash_fraction.mul(Self::free_balance(&pot))
            }
        };

        let minimum_validator_count = Self::reasonable_minimum_validator_count() as usize;
        let mut active_count = Self::active_validator_set().count();
        let mut chill_offender_safe = |offender: T::AccountId| {
            // The offender does not have enough balance for the slashing and has to be chilled,
            // but we must avoid the over-slashing, ensure have the minimum active validators.
            if active_count > minimum_validator_count {
                Self::apply_force_chilled(&offender);
                active_count -= 1;
                Some(offender)
            } else {
                None
            }
        };

        offenders
            .into_iter()
            .flat_map(|(offender, slash_fraction)| {
                let base_slash = calc_base_slash(&offender, slash_fraction);
                let penalty = validator_rewards
                    .get(&offender)
                    .copied()
                    .map(|reward| reward + base_slash)
                    .unwrap_or(base_slash)
                    .max(minimum_penalty);
                match slasher.try_slash(&offender, penalty) {
                    SlashOutcome::Slashed(_) => {
                        debug!(
                            target: "runtime::mining::staking",
                            "Slash the offender:{:?} for penalty {:?} by the given slash_fraction:{:?} successfully",
                            offender, penalty, slash_fraction
                        );
                        None
                    }
                    SlashOutcome::InsufficientSlash(actual_slashed) => {
                        debug!(
                            target: "runtime::mining::staking",
                            "Insufficient reward pot balance of {:?}, actual slashed:{:?}",
                            offender, actual_slashed
                        );
                        chill_offender_safe(offender)
                    }
                    SlashOutcome::SlashFailed(e) => {
                        debug!(
                            target: "runtime::mining::staking",
                            "Slash the offender {:?} for {:?} somehow failed: {:?}", offender, penalty, e,
                        );
                        // we still chill the offender even the slashing failed as currently
                        // the offender is only the authorties without running a node.
                        //
                        // TODO: Reconsider this once https://github.com/paritytech/substrate/pull/7127
                        // is merged.
                        chill_offender_safe(offender)
                    }
                }
            })
            .collect()
    }
}

/*
这段代码是ChainX区块链网络中负责处理离线验证者(offenders)的惩罚逻辑的一部分.它定义了一个名为`slash_offenders_in_session`的
函数,该函数用于在给定的惩罚分数(`slash_fraction`)和验证者奖励(`validator_rewards`)的情况下,对违规的验证者执行惩罚.

### 惩罚逻辑(Punishment Logic)
- 函数接收两个参数:`offenders`是一个映射,包含违规验证者的账户ID和对应的惩罚比例;`validator_rewards`是一个包含验证者账户ID和对应奖励余额的向量.
- 首先,函数将奖励向量转换为`BTreeMap`以便于查找.
- 接着,函数获取财政账户(`treasury_account`)和创建一个`Slasher`实例,用于执行实际的惩罚操作.
- 函数还定义了`minimum_penalty`,即使违规行为不严重,也会对验证者施加的最小惩罚.
- `calc_base_slash`函数用于计算基于惩罚比例和奖励池余额的基础惩罚金额.如果惩罚比例为零(即验证者离线时间少于10%),则只施加最小惩罚.

### 执行惩罚(Executing Punishment)
- 函数遍历`offenders`映射,并对于每个违规验证者,使用`Slasher`实例尝试执行惩罚.
- 如果惩罚执行成功,函数记录一条调试信息并继续处理下一个违规者.
- 如果惩罚执行失败,可能是因为奖励池余额不足或其它原因,函数将尝试将违规者标记为冷却状态(`chill`),以确保网络中至少有最小数量的活跃验证者.

### 冷却违规者(Chilling Offenders)
- `chill_offender_safe`函数用于安全地将违规者标记为冷却状态.它首先检查当前活跃验证者的数量,如果超过最小要求,它将执行冷却操作并从活跃验证者计数中减一.
- 如果执行冷却操作后,活跃验证者的数量低于最小要求,函数将不会执行冷却,以避免网络运行受到影响.

### 调试信息(Debug Information)
- 在执行惩罚的过程中,函数会在每个关键步骤记录调试信息,以便于跟踪和审计惩罚操作.

整体而言,这段代码确保了ChainX区块链网络能够对不活跃的验证者施加适当的惩罚,同时保持网络的稳定性和安全性.
通过对违规者施加惩罚和在必要时将其标记为冷却状态,网络激励验证者保持在线,从而维护区块链的去中心化和可靠性.
*/
