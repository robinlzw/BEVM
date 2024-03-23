// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use crate::{
    traits::BytesLike, Config, GenericTrusteeIntentionProps, GenericTrusteeSessionInfo,
    TrusteeIntentionPropertiesOf, TrusteeIntentionProps, TrusteeSessionInfo, TrusteeSessionInfoLen,
    TrusteeSessionInfoOf,
};
use chainx_primitives::Text;
use codec::{Decode, Encode};
use frame_support::{log::info, traits::Get, weights::Weight, RuntimeDebug};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;
use xp_assets_registrar::Chain;

/// The trustee session info.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
struct OldTrusteeSessionInfo<AccountId, TrusteeAddress: BytesLike> {
    /// Trustee account
    pub trustee_list: Vec<AccountId>,
    /// Threshold value
    pub threshold: u16,
    /// Hot address
    pub hot_address: TrusteeAddress,
    /// Cold address
    pub cold_address: TrusteeAddress,
}

/// The generic trustee session info.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
struct OldGenericTrusteeSessionInfo<AccountId>(pub OldTrusteeSessionInfo<AccountId, Vec<u8>>);

/// The trustee intention properties.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct OldTrusteeIntentionProps<TrusteeEntity: BytesLike> {
    #[cfg_attr(feature = "std", serde(with = "xp_rpc::serde_text"))]
    pub about: Text,
    pub hot_entity: TrusteeEntity,
    pub cold_entity: TrusteeEntity,
}
/// The generic trustee intention properties.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct OldGenericTrusteeIntentionProps(pub OldTrusteeIntentionProps<Vec<u8>>);

/// Apply all of the migrations due to taproot.
///
/// ### Warning
///
/// Use with care and run at your own risk.
pub fn apply<T: Config>() -> Weight {
    info!(
        target: "runtime::gateway::common",
        "Running migration for gateway common pallet"
    );

    migrate_trustee_session_info::<T>().saturating_add(migrate_trustee_intention_properties::<T>())
}

/// Migrate from the old trustee session info.
pub fn migrate_trustee_session_info<T: Config>() -> Weight {
    TrusteeSessionInfoLen::<T>::mutate(Chain::Bitcoin, |l| *l = l.saturating_sub(1));
    TrusteeSessionInfoOf::<T>::translate::<OldGenericTrusteeSessionInfo<T::AccountId>, _>(
        |_, _, trustee_info| {
            Some(GenericTrusteeSessionInfo(TrusteeSessionInfo {
                trustee_list: trustee_info
                    .0
                    .trustee_list
                    .iter()
                    .map(|n| (n.clone(), 0))
                    .collect::<Vec<_>>(),
                threshold: trustee_info.0.threshold,
                hot_address: trustee_info.0.hot_address,
                cold_address: trustee_info.0.cold_address,
                multi_account: None,
                start_height: None,
                end_height: None,
            }))
        },
    );
    let count = TrusteeSessionInfoOf::<T>::iter_values().count();
    info!(
        target: "runtime::gateway::common",
        "migrated {} trustee session infos.",
        count,
    );
    <T as frame_system::Config>::DbWeight::get()
        .reads_writes(count as Weight + 1, count as Weight + 1)
}

/// Migrate from the old trustee intention properties.
pub fn migrate_trustee_intention_properties<T: Config>() -> Weight {
    TrusteeIntentionPropertiesOf::<T>::translate::<OldGenericTrusteeIntentionProps, _>(
        |_, _, props| {
            Some(GenericTrusteeIntentionProps(TrusteeIntentionProps {
                proxy_account: None,
                about: props.0.about,
                hot_entity: props.0.hot_entity,
                cold_entity: props.0.cold_entity,
            }))
        },
    );
    let count = TrusteeIntentionPropertiesOf::<T>::iter_values().count();
    info!(
        target: "runtime::gateway::common",
        "migrated {} trustee_intention_properties.",
        count,
    );
    <T as frame_system::Config>::DbWeight::get()
        .reads_writes(count as Weight + 1, count as Weight + 1)
}

/*
这段代码是 ChainX 区块链中 `gateway_common`  pallet 的一部分,
主要负责处理与受托人(trustee)相关的会话信息和意图属性的迁移.
由于比特币 Taproot 升级,需要对受托人的信息结构进行更新,以适应新的地址格式和签名方案.
代码中定义了旧的受托人会话信息(`OldTrusteeSessionInfo`)和意图属性(`OldTrusteeIntentionProps`),
以及迁移函数,用于将旧数据转换为新的格式.

### 主要结构体和函数解释:

1. **OldTrusteeSessionInfo**:
   - 旧的受托人会话信息结构体,包含受托人列表,阈值,热钱包地址和冷钱包地址.

2. **OldGenericTrusteeSessionInfo**:
   - 泛型的旧受托人会话信息结构体,用于迁移时的类型转换.

3. **OldTrusteeIntentionProps**:
   - 旧的受托人意图属性结构体,包含关于受托人的描述信息和热钱包及冷钱包的实体.

4. **OldGenericTrusteeIntentionProps**:
   - 泛型的旧受托人意图属性结构体,同样用于迁移时的类型转换.

5. **apply**:
   - 应用所有迁移函数的入口点,它会调用 `migrate_trustee_session_info` 
   和 `migrate_trustee_intention_properties` 来执行迁移.

6. **migrate_trustee_session_info**:
   - 将旧的受托人会话信息迁移到新的格式.它会遍历所有旧的会话信息,并将它们转换为新的 `TrusteeSessionInfo` 格式.

7. **migrate_trustee_intention_properties**:
   - 将旧的受托人意图属性迁移到新的格式.它会遍历所有旧的意图属性,并将它们转换为新的 `TrusteeIntentionProps` 格式.

这些迁移函数在 Taproot 升级后非常重要,因为它们确保了旧的受托人信息与新的比特币地址格式兼容.
迁移过程中,旧的数据结构被新的数据结构所替代,同时保留了原有的数据内容.
这样的迁移对于保持区块链系统的连续性和安全性至关重要.

*/
