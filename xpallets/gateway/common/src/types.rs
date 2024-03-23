// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use sp_runtime::RuntimeDebug;
use sp_std::{convert::TryFrom, prelude::Vec};

use chainx_primitives::Text;

use crate::traits::BytesLike;

/// The config of trustee info.
#[derive(PartialEq, Clone, Encode, Decode, Default, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct TrusteeInfoConfig {
    pub min_trustee_count: u32,
    pub max_trustee_count: u32,
}

/// The trustee session info.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct TrusteeSessionInfo<AccountId, BlockNumber, TrusteeAddress: BytesLike> {
    /// Trustee account
    pub trustee_list: Vec<(AccountId, u64)>,
    /// Threshold value
    pub threshold: u16,
    /// Hot address
    pub hot_address: TrusteeAddress,
    /// Cold address
    pub cold_address: TrusteeAddress,
    /// Trustee multi account to receive congressional multi-signature rewards
    pub multi_account: Option<AccountId>,
    /// The height of trustee start
    pub start_height: Option<BlockNumber>,
    /// The height of trustee end
    pub end_height: Option<BlockNumber>,
}

/// Aggregate public key script and corresponding personal public key index.
///
/// Each aggregate public key corresponds to multiple accounts.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct ScriptInfo<AccountId> {
    pub agg_pubkeys: Vec<Vec<u8>>,
    pub personal_accounts: Vec<Vec<AccountId>>,
}

/// Used to record the rewards distributed by the trustee.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct RewardInfo<AccountId, Balance> {
    pub rewards: Vec<(AccountId, Balance)>,
}

/// The generic trustee session info.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct GenericTrusteeSessionInfo<AccountId, BlockNumber>(
    pub TrusteeSessionInfo<AccountId, BlockNumber, Vec<u8>>,
);

impl<AccountId, BlockNumber, TrusteeAddress: BytesLike>
    From<TrusteeSessionInfo<AccountId, BlockNumber, TrusteeAddress>>
    for GenericTrusteeSessionInfo<AccountId, BlockNumber>
{
    fn from(info: TrusteeSessionInfo<AccountId, BlockNumber, TrusteeAddress>) -> Self {
        GenericTrusteeSessionInfo(TrusteeSessionInfo {
            trustee_list: info.trustee_list,
            multi_account: info.multi_account,
            start_height: info.start_height,
            threshold: info.threshold,
            hot_address: info.hot_address.into(),
            cold_address: info.cold_address.into(),
            end_height: info.end_height,
        })
    }
}

impl<AccountId, BlockNumber, TrusteeAddress: BytesLike>
    TryFrom<GenericTrusteeSessionInfo<AccountId, BlockNumber>>
    for TrusteeSessionInfo<AccountId, BlockNumber, TrusteeAddress>
{
    // TODO, may use a better error
    type Error = ();

    fn try_from(
        info: GenericTrusteeSessionInfo<AccountId, BlockNumber>,
    ) -> Result<Self, Self::Error> {
        Ok(
            TrusteeSessionInfo::<AccountId, BlockNumber, TrusteeAddress> {
                trustee_list: info.0.trustee_list,
                multi_account: info.0.multi_account,
                start_height: info.0.start_height,
                threshold: info.0.threshold,
                hot_address: TrusteeAddress::try_from(info.0.hot_address).map_err(|_| ())?,
                cold_address: TrusteeAddress::try_from(info.0.cold_address).map_err(|_| ())?,
                end_height: info.0.end_height,
            },
        )
    }
}

/// The trustee intention properties.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct TrusteeIntentionProps<AccountId, TrusteeEntity: BytesLike> {
    pub proxy_account: Option<AccountId>,
    #[cfg_attr(feature = "std", serde(with = "xp_rpc::serde_text"))]
    pub about: Text,
    pub hot_entity: TrusteeEntity,
    pub cold_entity: TrusteeEntity,
}

/// The generic trustee intention properties.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct GenericTrusteeIntentionProps<AccountId>(pub TrusteeIntentionProps<AccountId, Vec<u8>>);

impl<AccountId, TrusteeEntity: BytesLike> From<TrusteeIntentionProps<AccountId, TrusteeEntity>>
    for GenericTrusteeIntentionProps<AccountId>
{
    fn from(props: TrusteeIntentionProps<AccountId, TrusteeEntity>) -> Self {
        GenericTrusteeIntentionProps(TrusteeIntentionProps {
            proxy_account: props.proxy_account,
            about: props.about,
            hot_entity: props.hot_entity.into(),
            cold_entity: props.cold_entity.into(),
        })
    }
}

impl<AccountId, TrusteeEntity: BytesLike> TryFrom<GenericTrusteeIntentionProps<AccountId>>
    for TrusteeIntentionProps<AccountId, TrusteeEntity>
{
    // TODO, may use a better error
    type Error = ();

    fn try_from(value: GenericTrusteeIntentionProps<AccountId>) -> Result<Self, Self::Error> {
        Ok(TrusteeIntentionProps::<AccountId, TrusteeEntity> {
            proxy_account: value.0.proxy_account,
            about: value.0.about,
            hot_entity: TrusteeEntity::try_from(value.0.hot_entity).map_err(|_| ())?,
            cold_entity: TrusteeEntity::try_from(value.0.cold_entity).map_err(|_| ())?,
        })
    }
}

/*
这段代码定义了 ChainX 区块链项目中与受托人(trustee)相关的数据结构和类型转换.
这些结构体用于存储和管理受托人的信息,包括会话信息,意图属性,脚本信息和奖励信息.
代码还提供了序列化/反序列化的支持,使得这些数据结构可以在网络中传输或存储.

### 主要结构体和类型:

1. **TrusteeInfoConfig**:
   - 受托人信息配置结构体,包含最小和最大受托人数量.

2. **TrusteeSessionInfo**:
   - 受托人会话信息结构体,包含受托人列表,阈值,热钱包地址,冷钱包地址,多重签名账户,开始和结束高度.

3. **ScriptInfo**:
   - 脚本信息结构体,包含聚合公钥和对应的个人账户列表.

4. **RewardInfo**:
   - 奖励信息结构体,用于记录受托人分配的奖励.

5. **GenericTrusteeSessionInfo**:
   - 泛型受托人会话信息结构体,用于类型转换.

6. **TrusteeIntentionProps**:
   - 受托人意图属性结构体,包含代理账户,描述,热钱包实体和冷钱包实体.

7. **GenericTrusteeIntentionProps**:
   - 泛型受托人意图属性结构体,用于类型转换.

### 类型转换实现:

- **From 和 TryFrom**:
  - 为 `TrusteeSessionInfo` 和 `GenericTrusteeSessionInfo` 之间,`TrusteeIntentionProps` 
  和 `GenericTrusteeIntentionProps` 之间提供了类型转换的实现.这些转换用于在不同的上下文中传递和处理受托人信息.

### 序列化/反序列化支持:

- **Encode 和 Decode**:
  - 通过实现 `Encode` 和 `Decode` trait,这些结构体可以被编码和解码为字节序列,便于在区块链网络中传输.

- **Serialize 和 Deserialize**:
  - 当启用标准库特性时,通过 `serde` 库提供序列化和反序列化的支持,使得这些结构体可以被转换为 JSON 格式或其他序列化格式.

### 总结:

这段代码为 ChainX 区块链项目提供了一套完整的受托人信息管理工具,包括数据结构的定义,类型转换和序列化/反序列化的支持.
这些工具对于实现受托人机制和确保区块链网络的安全性和透明度至关重要.
*/
