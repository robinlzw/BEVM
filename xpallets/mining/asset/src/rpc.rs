// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use sp_runtime::{RuntimeDebug, SaturatedConversion};

use chainx_primitives::AssetId;
use xp_mining_common::RewardPotAccountFor;

use crate::{
    types::*, AssetLedgers, BalanceOf, ClaimRestrictionOf, Config, FixedAssetPowerOf, MinerLedgers,
    MiningPrevilegedAssets, Pallet,
};

/// Mining asset info.
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct MiningAssetInfo<AccountId, Balance, MiningWeight, BlockNumber> {
    pub asset_id: AssetId,
    pub mining_power: FixedAssetPower,
    pub reward_pot: AccountId,
    pub reward_pot_balance: Balance,
    #[cfg_attr(feature = "std", serde(flatten))]
    pub ledger_info: AssetLedger<MiningWeight, BlockNumber>,
}

/// Detailed dividend info of asset miner.
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct MiningDividendInfo<Balance> {
    /// Actual dividend balance that belongs to the claimer.
    pub own: Balance,
    /// Dividend cut(10% of total) for the referral of claimer or treasury.
    pub other: Balance,
    /// Required more stake to claim the mining dividend.
    pub insufficient_stake: Balance,
}

impl<T: Config> Pallet<T> {
    /// Get overall information about all mining assets.
    pub fn mining_assets(
    ) -> Vec<MiningAssetInfo<T::AccountId, BalanceOf<T>, MiningWeight, T::BlockNumber>> {
        MiningPrevilegedAssets::<T>::get()
            .into_iter()
            .map(|asset_id| {
                let mining_power = FixedAssetPowerOf::<T>::get(asset_id);
                let reward_pot = T::DetermineRewardPotAccount::reward_pot_account_for(&asset_id);
                let reward_pot_balance: BalanceOf<T> = Self::free_balance(&reward_pot);
                let ledger_info: AssetLedger<MiningWeight, T::BlockNumber> =
                    AssetLedgers::<T>::get(asset_id);
                MiningAssetInfo {
                    asset_id,
                    mining_power,
                    reward_pot,
                    reward_pot_balance,
                    ledger_info,
                }
            })
            .collect()
    }

    /// Get the asset mining dividends info given the miner AccountId.
    pub fn mining_dividend(
        who: T::AccountId,
    ) -> BTreeMap<AssetId, MiningDividendInfo<BalanceOf<T>>> {
        let current_block = <frame_system::Pallet<T>>::block_number();
        MinerLedgers::<T>::iter_prefix(&who)
            .filter_map(|(asset_id, _)| {
                match Self::compute_dividend_at(&who, &asset_id, current_block) {
                    Ok(dividend) => {
                        let ClaimRestriction {
                            staking_requirement,
                            ..
                        } = ClaimRestrictionOf::<T>::get(&asset_id);
                        let insufficient_stake =
                            Self::need_more_stake(&who, dividend, staking_requirement)
                                .unwrap_or_default();
                        let other = dividend / 10u32.saturated_into::<BalanceOf<T>>();
                        let own = dividend - other;
                        Some((
                            asset_id,
                            MiningDividendInfo {
                                own,
                                other,
                                insufficient_stake,
                            },
                        ))
                    }
                    Err(_) => None,
                }
            })
            .collect()
    }

    /// Get the nomination details given the staker AccountId.
    pub fn miner_ledger(
        who: T::AccountId,
    ) -> BTreeMap<AssetId, MinerLedger<MiningWeight, T::BlockNumber>> {
        MinerLedgers::<T>::iter_prefix(&who).collect()
    }
}

/*
这段代码是ChainX区块链项目的一部分,它定义了挖矿资产信息和挖矿分红信息的结构体,并提供了获取这些信息的函数.
这些函数对于ChainX区块链的挖矿模块来说是核心功能,因为它们允许用户和智能合约查询有关挖矿资产的状态和挖矿奖励的详细信息.

以下是代码的主要组成部分和它们的功能:

1. **`MiningAssetInfo`结构体**:
   - 这个结构体包含了有关挖矿资产的详细信息,包括资产ID,挖矿权重,奖励池账户,奖励池余额和账本信息.它实现了
   `PartialEq`,`Eq`,`Clone`,`Default`,`Encode`,`Decode`和`RuntimeDebug`特质,
   以及可选的`Serialize`和`Deserialize`特质(当启用`std`特性时).

2. **`MiningDividendInfo`结构体**:
   - 这个结构体包含了有关矿工挖矿分红的详细信息,包括矿工自己的分红余额,推荐人或国库的分红余额以及由于质押余额不足而
   无法认领的额外质押需求.它同样实现了`PartialEq`,`Eq`,`Clone`,`Default`,`Encode`,`Decode`和`RuntimeDebug`特质,
   以及可选的`Serialize`和`Deserialize`特质.

3. **`Pallet`实现**:
   - 为`Pallet`提供了三个公共函数:
     - `mining_assets`: 返回所有特权挖矿资产的列表,每个资产的详细信息包括其挖矿权重,奖励池账户和余额,以及当前的账本信息.
     - `mining_dividend`: 根据矿工的账户ID返回该矿工所有挖矿资产的分红信息.它计算当前区块号下的分红,并检查矿工是否有足够的质押余额来认领这些分红.
     - `miner_ledger`: 返回给定矿工账户的所有挖矿资产的账本详细信息.

这些函数对于ChainX区块链的挖矿模块至关重要,因为它们提供了挖矿资产和分红的透明度,允许矿工和用户了解他们的挖矿状态和潜在奖励.
这对于矿工来说尤其重要,因为他们可以根据这些信息做出是否继续挖矿或调整质押策略的决策.
*/
