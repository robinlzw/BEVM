use crate::*;

impl<T: Config> Pallet<T> {
    /// Initializes the genesis validators.
    ///
    /// Set the weight to 0.
    pub fn initialize_validators(
        validators: &[xp_genesis_builder::ValidatorInfo<T::AccountId, BalanceOf<T>>],
        initialize_validators: &[Vec<u8>],
    ) -> DispatchResult {
        for xp_genesis_builder::ValidatorInfo {
            who,
            referral_id,
            total_nomination,
        } in validators
        {
            Self::check_referral_id(referral_id)?;
            Self::apply_register(who, referral_id.to_vec());
            // These validators will be chilled on the network startup.
            if !initialize_validators.contains(referral_id) {
                Self::apply_force_chilled(who);
            }

            ValidatorLedgers::<T>::mutate(who, |validator| {
                validator.total_nomination = *total_nomination;
                validator.last_total_vote_weight = Default::default();
            });
        }
        Ok(())
    }

    pub fn force_bond(
        sender: &T::AccountId,
        target: &T::AccountId,
        value: BalanceOf<T>,
    ) -> DispatchResult {
        if !value.is_zero() {
            Self::bond_reserve(sender, value);
            Nominations::<T>::mutate(sender, target, |nominator| {
                nominator.nomination = value;
            });
        }
        Ok(())
    }

    /// Mock the `unbond` operation but lock the funds until the specific height `locked_until`.
    pub fn force_unbond(
        sender: &T::AccountId,
        target: &T::AccountId,
        value: BalanceOf<T>,
        locked_until: T::BlockNumber,
    ) -> DispatchResult {
        // We can not reuse can_unbond() as the target can has no bond but has unbonds.
        // Self::can_unbond(sender, target, value)?;
        ensure!(Self::is_validator(target), Error::<T>::NotValidator);
        ensure!(
            Self::unbonded_chunks_of(sender, target).len()
                < Self::maximum_unbonded_chunk_size() as usize,
            Error::<T>::NoMoreUnbondChunks
        );
        Self::unbond_reserve(sender, value)?;
        Self::mutate_unbonded_chunks(sender, target, value, locked_until);
        Ok(())
    }

    pub fn force_set_nominator_vote_weight(
        nominator: &T::AccountId,
        validator: &T::AccountId,
        new_weight: VoteWeight,
    ) {
        Nominations::<T>::mutate(nominator, validator, |nominator| {
            nominator.last_vote_weight = new_weight;
        });
    }

    pub fn force_set_validator_vote_weight(who: &T::AccountId, new_weight: VoteWeight) {
        ValidatorLedgers::<T>::mutate(who, |validator| {
            validator.last_total_vote_weight = new_weight;
        });
    }
}

/*
这段代码是ChainX区块链网络中关于验证者管理的智能合约的一部分,使用Rust语言编写.
它定义了几个与验证者初始化,质押,解质押和投票权重相关的函数.下面是对这些函数的详细解释:

1. `initialize_validators`: 此函数用于在区块链创世时初始化验证者.它接受一个验证者信息数组和一个初始化验证者数组.
对于每个验证者,函数会检查推荐ID,应用注册逻辑,设置验证者的总提名和最后总投票权重.如果验证者的推荐ID不在初始化验证者数组中,
则会将其设置为不活跃(chilled)状态.这个函数确保了在网络启动时,验证者的状态被正确设置.

2. `force_bond`: 此函数允许强制对指定的账户进行质押.它首先检查质押值是否为零,如果不是,则从发送者账户中扣除相应的金额,
并更新提名者的提名值.这个函数可以用于在特定情况下,如测试或模拟操作时,强制进行质押.

3. `force_unbond`: 此函数模拟了解质押操作,但是将资金锁定直到特定的区块高度.函数首先确保目标账户是一个验证者,
并且发送者的解质押块数没有达到最大限制.然后,它会尝试解质押并更新解质押块的列表.这个函数可以用于在特定情况下强制执行解质押操作.

4. `force_set_nominator_vote_weight`: 此函数允许强制设置提名者的投票权重.它通过变异(mutate)提名者的记录来更新其最后投票权重.
这个函数可以用于在测试或模拟操作时,强制更新提名者的投票权重.

5. `force_set_validator_vote_weight`: 类似地,此函数允许强制设置验证者的投票权重.它通过变异验证者的记录来更新其最后总投票权重.
这个函数同样适用于在特定情况下调整验证者的投票权重.

这些函数为ChainX区块链网络提供了一套工具,以便在需要时能够强制执行验证者的质押,解质押和投票权重的更改.
这些操作通常在正常的区块链操作中由用户通过交易自行发起,但在这里提供了一种机制,使得在特定情况下(如测试或紧急情况)能够由智能合约直接进行这些操作.
*/
