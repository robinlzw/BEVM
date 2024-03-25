// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use super::*;
#[allow(unused_imports)]
use micromath::F32Ext;
use sp_std::vec::Vec;

mod proposal09;

impl<T: Config> Pallet<T> {
    /// Simple u32 power of 2 function - simply uses a bit shift
    #[inline]
    fn pow2(n: u32) -> BalanceOf<T> {
        (1_u32 << n).saturated_into()
    }

    /// (1/2)^(n+1) < (2100 - x) / 2100 <= (1/2)^n
    /// Returns the total reward for the session, assuming it ends with this block.
    pub(crate) fn this_session_reward() -> BalanceOf<T> {
        let total_issuance = T::Currency::total_issuance().saturated_into::<u64>(); // x
        let tt = (FIXED_TOTAL / (FIXED_TOTAL - total_issuance)) as f32;
        let halving_epoch = tt.log2().trunc() as u32; // n

        INITIAL_REWARD.saturated_into::<BalanceOf<T>>() / Self::pow2(halving_epoch)
    }

    /// Issue new fresh PCX.
    #[inline]
    pub(crate) fn mint(receiver: &T::AccountId, value: BalanceOf<T>) {
        T::Currency::deposit_creating(receiver, value);
        Self::deposit_event(Event::<T>::Minted(receiver.clone(), value));
    }

    /// Issue new fresh PCX.
    #[inline]
    pub(crate) fn mint_for_validator(
        validator: &T::AccountId,
        reward: BalanceOf<T>,
        validator_pot: &T::AccountId,
        reward_pot: BalanceOf<T>,
    ) {
        T::Currency::deposit_creating(validator, reward);
        T::Currency::deposit_creating(validator_pot, reward_pot);

        Self::deposit_event(Event::<T>::MintedForValidator(
            validator.clone(),
            reward,
            validator_pot.clone(),
            reward_pot,
        ));
    }

    /// Reward a (potential) validator by a specific amount.
    ///
    /// Add the reward to their balance, and their reward pot, pro-rata.
    fn apply_reward_validator(who: &T::AccountId, reward: BalanceOf<T>) {
        // Validator themselves can only directly gain 20%, the rest 80% is for the reward pot.
        let off_the_table = reward.saturated_into::<BalanceOf<T>>() / 5u32.saturated_into();

        // Issue the rest 80% to validator's reward pot.
        let to_reward_pot = (reward - off_the_table).saturated_into();
        let reward_pot = T::DetermineRewardPotAccount::reward_pot_account_for(who);

        Self::mint_for_validator(who, off_the_table, &reward_pot, to_reward_pot);

        frame_support::log::debug!(
            target: "runtime::mining::staking",
            "� Mint validator({:?}):{:?}, reward_pot({:?}):{:?}",
            who,
            off_the_table,
            reward_pot,
            to_reward_pot
        );
    }

    /// Reward the intention and slash the validators that went offline in last session.
    ///
    /// If the slashed validator can't afford that penalty, it will be
    /// removed from the validator list.
    #[inline]
    fn reward_active_validator(validator: &T::AccountId, reward: BalanceOf<T>) {
        Self::apply_reward_validator(validator, reward);
    }

    /// Distribute the session reward to all the receivers, returns the total reward for validators.
    pub(crate) fn distribute_session_reward() -> Vec<(T::AccountId, BalanceOf<T>)> {
        let session_reward = Self::this_session_reward();

        Self::distribute_session_reward_impl_09(session_reward)
    }
}

/*
这段代码是ChainX区块链项目中的一个模块,它提供了与发行新代币(PCX)和分配挖矿奖励相关的功能.
代码中定义了一系列函数,用于计算奖励,发行新代币以及记录事件.以下是代码的主要组成部分和它们的功能:

1. **`pow2`函数**:
   - 一个简单的函数,用于计算2的幂.它通过位左移操作来实现,并返回`BalanceOf<T>`类型的结果.

2. **`this_session_reward`函数**:
   - 计算当前会话的总奖励.它首先获取当前发行的总代币量,然后根据这个量计算出减半周期,最后根据减半周期计算出当前会话的奖励.

3. **`mint`函数**:
   - 用于发行新代币给指定的接收者.它将代币直接存入接收者的账户,并通过事件记录这一操作.

4. **`mint_for_validator`函数**:
   - 用于给验证者发放奖励.它将一部分奖励直接发放给验证者,并将剩余的奖励存入验证者的奖励池.

5. **`apply_reward_validator`函数**:
   - 应用奖励给验证者.它根据奖励规则,将奖励的一部分直接发放给验证者,其余部分存入奖励池.

6. **`reward_active_validator`函数**:
   - 奖励活跃的验证者.这个函数调用`apply_reward_validator`来给验证者发放奖励.

7. **`distribute_session_reward`函数**:
   - 分发会话奖励给所有接收者.它调用`distribute_session_reward_impl_09`函数来实现奖励的分配,并返回一个包含所有接收者和他们获得的奖励的向量.

整体来看,这段代码为ChainX区块链提供了一个完整的挖矿奖励分配系统.它确保了挖矿奖励的分配是公平和透明的,并且与验证者的参与度成正比.
这对于激励用户参与挖矿和维护区块链的安全性至关重要.此外,代码中的事件记录功能允许区块链的其他部分跟踪和审计挖矿奖励的分配情况.
*/
