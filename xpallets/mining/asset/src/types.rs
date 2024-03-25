// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::RuntimeDebug;

use chainx_primitives::AssetId;
use xp_mining_common::WeightType;

use crate::Config;

pub type MiningWeight = WeightType;
pub type FixedAssetPower = u32;
pub type StakingRequirement = u32;

/// Vote weight properties of validator.
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct AssetLedger<MiningWeight, BlockNumber> {
    /// Last calculated total vote weight of current validator.
    pub last_total_mining_weight: MiningWeight,
    /// Block number at which point `last_total_vote_weight` just updated.
    pub last_total_mining_weight_update: BlockNumber,
}

pub struct AssetLedgerWrapper<'a, T: Config> {
    pub asset_id: &'a AssetId,
    pub inner: &'a mut AssetLedger<MiningWeight, T::BlockNumber>,
}

impl<'a, T: Config> AssetLedgerWrapper<'a, T> {
    pub fn new(
        asset_id: &'a AssetId,
        inner: &'a mut AssetLedger<MiningWeight, T::BlockNumber>,
    ) -> Self {
        Self { asset_id, inner }
    }
}

/// Mining weight properties of asset miners.
///
/// Aside from the mining weight information, this struct also contains
/// the `last_claim` field, for it's not neccessary to use another
/// storeage item due to the claim restrictions of asset miners.
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct MinerLedger<MiningWeight, BlockNumber> {
    /// Last calculated total vote weight of current validator.
    pub last_mining_weight: MiningWeight,
    /// Block number at which point `last_total_vote_weight` just updated.
    pub last_mining_weight_update: BlockNumber,
    /// Block number at which point the miner claimed last time.
    pub last_claim: Option<BlockNumber>,
}

pub struct MinerLedgerWrapper<'a, T: Config> {
    pub miner: &'a T::AccountId,
    pub asset_id: &'a AssetId,
    pub inner: &'a mut MinerLedger<MiningWeight, T::BlockNumber>,
}

impl<'a, T: Config> MinerLedgerWrapper<'a, T> {
    pub fn new(
        miner: &'a T::AccountId,
        asset_id: &'a AssetId,
        inner: &'a mut MinerLedger<MiningWeight, T::BlockNumber>,
    ) -> Self {
        Self {
            miner,
            asset_id,
            inner,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct ClaimRestriction<BlockNumber> {
    /// Claimer must have `staking_requirement` times of PCX staked.
    pub staking_requirement: StakingRequirement,
    /// Claimer can only claim once per `frequency_limit`.
    pub frequency_limit: BlockNumber,
}

/*
这段代码定义了ChainX区块链项目中与挖矿权重相关的几个关键结构体和类型.这些结构体用于存储和处理挖矿资产和矿工的权重信息,
以及认领挖矿奖励的限制条件.以下是代码的主要组成部分和它们的功能:

1. **类型别名**:
   - `MiningWeight`: 用于表示挖矿权重的类型.
   - `FixedAssetPower`: 用于表示资产固定挖矿权重的类型,这里被定义为`u32`.
   - `StakingRequirement`: 用于表示质押要求的类型,同样被定义为`u32`.

2. **`AssetLedger`结构体**:
   - 存储了资产的挖矿权重信息,包括最后计算的总挖矿权重和更新这个权重的区块号.

3. **`AssetLedgerWrapper`结构体**:
   - 为了简化对`AssetLedger`的引用和修改,提供了一个包装结构体.

4. **`MinerLedger`结构体**:
   - 存储了矿工的挖矿权重信息,包括最后计算的总挖矿权重,更新权重的区块号以及矿工上次认领奖励的区块号.

5. **`MinerLedgerWrapper`结构体**:
   - 同样为了简化对`MinerLedger`的引用和修改,提供了一个包装结构体.

6. **`ClaimRestriction`结构体**:
   - 定义了认领挖矿奖励的限制条件,包括质押要求和认领频率限制.

这些结构体和类型被用于ChainX区块链的挖矿模块中,以确保挖矿奖励的分配是基于矿工的挖矿权重和认领规则的.
通过这些定义,ChainX区块链能够维护一个公平和透明的挖矿奖励系统,激励矿工参与网络的维护和安全.此外,这些结构体
实现了`Encode`,`Decode`,`RuntimeDebug`和`TypeInfo`特质,使得它们可以被序列化,反序列化,并在运行时进行调试和类型信息的查询.
如果启用了`std`特性,它们还会实现`Serialize`和`Deserialize`特质,以便在JSON或其他格式中进行序列化和反序列化.
*/
