// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.
#![allow(clippy::type_complexity)]
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use sp_runtime::RuntimeDebug;

use xp_mining_common::RewardPotAccountFor;

use crate::{
    types::*, BalanceOf, Config, LastRebondOf, Nominations, Pallet, SessionInterface,
    ValidatorLedgers, Validators,
};

/// Total information about a validator.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct ValidatorInfo<AccountId, Balance, VoteWeight, BlockNumber> {
    /// AccountId of this (potential) validator.
    pub account: AccountId,
    #[cfg_attr(feature = "std", serde(flatten))]
    pub profile: ValidatorProfile<BlockNumber>,
    #[cfg_attr(feature = "std", serde(flatten))]
    pub ledger: ValidatorLedger<Balance, VoteWeight, BlockNumber>,
    /// Being a validator, responsible for authoring the new blocks.
    pub is_validating: bool,
    /// How much balances the validator has bonded itself.
    pub self_bonded: Balance,
    /// AccountId of the reward pot of this validator.
    pub reward_pot_account: AccountId,
    /// Balance of the reward pot account.
    pub reward_pot_balance: Balance,
}

/// Profile of staking nominator.
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct NominatorInfo<BlockNumber> {
    /// Block number of last `rebond` operation.
    pub last_rebond: Option<BlockNumber>,
}

impl<T: Config> Pallet<T> {
    pub fn validators_info(
    ) -> Vec<ValidatorInfo<T::AccountId, BalanceOf<T>, VoteWeight, T::BlockNumber>> {
        Self::validator_set().map(Self::validator_info_of).collect()
    }

    pub fn validator_info_of(
        who: T::AccountId,
    ) -> ValidatorInfo<T::AccountId, BalanceOf<T>, VoteWeight, T::BlockNumber> {
        let profile = Validators::<T>::get(&who);
        let ledger: ValidatorLedger<BalanceOf<T>, VoteWeight, T::BlockNumber> =
            ValidatorLedgers::<T>::get(&who);
        let self_bonded: BalanceOf<T> = Nominations::<T>::get(&who, &who).nomination;
        let is_validating = T::SessionInterface::validators().contains(&who);
        let reward_pot_account = T::DetermineRewardPotAccount::reward_pot_account_for(&who);
        let reward_pot_balance: BalanceOf<T> = Self::free_balance(&reward_pot_account);
        ValidatorInfo {
            account: who,
            profile,
            ledger,
            is_validating,
            self_bonded,
            reward_pot_account,
            reward_pot_balance,
        }
    }

    pub fn staking_dividend_of(who: T::AccountId) -> BTreeMap<T::AccountId, BalanceOf<T>> {
        let current_block = <frame_system::Pallet<T>>::block_number();
        Nominations::<T>::iter_prefix(&who)
            .filter_map(|(validator, _)| {
                match Self::compute_dividend_at(&who, &validator, current_block) {
                    Ok(dividend) => Some((validator, dividend)),
                    Err(_) => None,
                }
            })
            .collect()
    }

    pub fn nomination_details_of(
        who: T::AccountId,
    ) -> BTreeMap<T::AccountId, NominatorLedger<BalanceOf<T>, VoteWeight, T::BlockNumber>> {
        Nominations::<T>::iter_prefix(&who)
            .map(|(validator, ledger)| (validator, ledger))
            .collect()
    }

    pub fn nominator_info_of(who: T::AccountId) -> NominatorInfo<T::BlockNumber> {
        let last_rebond = LastRebondOf::<T>::get(&who);
        NominatorInfo { last_rebond }
    }
}

/*
这段代码是ChainX区块链网络中负责质押(Staking)模块的一部分,它定义了与验证者(Validators)和
提名者(Nominators)相关的信息结构和辅助函数.以下是对代码中关键部分的详细解释:

### 验证者信息结构(ValidatorInfo)
- `ValidatorInfo`结构体包含了一个验证者的完整信息,包括其账户ID,质押档案,质押账本,是否正在验证,自我质押金额,奖励金库账户ID和奖励金库余额.

### 提名者信息结构(NominatorInfo)
- `NominatorInfo`结构体包含了一个提名者的质押档案信息,主要是最后一次`rebond`操作的区块号.

### 辅助函数(Helper Functions)
- `validators_info`:返回当前所有验证者的详细信息列表.
- `validator_info_of`:返回特定验证者的详细信息.
- `staking_dividend_of`:返回一个提名者从所有验证者那里获得的股息信息,以`AccountId`为键,`Balance`为值的映射.
- `nomination_details_of`:返回一个提名者对所有验证者的提名详细信息,以`AccountId`为键,`NominatorLedger`为值的映射.
- `nominator_info_of`:返回一个提名者的详细信息.

这些结构和函数为ChainX区块链网络中的质押模块提供了必要的数据结构和查询功能,使得质押和提名的相关信息可以
被有效地管理和访问.通过这些工具,区块链的参与者可以跟踪自己的质押状态,验证者的信誉和潜在的奖励,从而做出更明智的决策.
*/
