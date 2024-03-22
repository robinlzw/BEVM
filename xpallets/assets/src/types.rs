// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use bitflags::bitflags;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

// Substrate
use sp_runtime::RuntimeDebug;
use sp_std::{collections::btree_map::BTreeMap, prelude::*, slice::Iter};

// ChainX
pub use chainx_primitives::{Decimals, Desc, Token};
use xpallet_assets_registrar::AssetInfo;

use frame_support::traits::LockIdentifier;

use crate::{Config, Error};

const ASSET_TYPES: [AssetType; 5] = [
    AssetType::Usable,
    AssetType::Locked,
    AssetType::Reserved,
    AssetType::ReservedWithdrawal,
    AssetType::ReservedDexSpot,
];

/// Concrete type of non-native asset balance.
///
/// NOTE: The native token also reserves an AssetId in this module, but it's
/// handle by Balances runtime module in fact.
#[derive(PartialEq, PartialOrd, Ord, Eq, Clone, Copy, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum AssetType {
    /// Free balance.
    Usable,
    /// Placeholder for the future use.
    ///
    /// Unused for now.
    Locked,
    /// General reserved balance.
    ///
    /// Unused for now.
    Reserved,
    /// Reserved balance when an account redeems its bridged asset.
    ReservedWithdrawal,
    /// Reserved balance for creating order in DEX.
    ReservedDexSpot,
}

impl AssetType {
    /// Returns an iterator of all asset types.
    pub fn iter() -> Iter<'static, AssetType> {
        ASSET_TYPES.iter()
    }
}

impl Default for AssetType {
    fn default() -> Self {
        Self::Usable
    }
}

bitflags! {
    /// Restrictions for asset operations.
    #[derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    pub struct AssetRestrictions: u32 {
        const MOVE                = 1 << 0;
        const TRANSFER            = 1 << 1;
        const DEPOSIT             = 1 << 2;
        const WITHDRAW            = 1 << 3;
        const DESTROY_WITHDRAWAL  = 1 << 4;
        const DESTROY_USABLE      = 1 << 5;
    }
}

impl Default for AssetRestrictions {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct TotalAssetInfo<Balance> {
    pub info: AssetInfo,
    pub balance: BTreeMap<AssetType, Balance>,
    pub is_online: bool,
    pub restrictions: AssetRestrictions,
}

#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum AssetErr {
    NotEnough,
    OverFlow,
    TotalAssetNotEnough,
    TotalAssetOverFlow,
    InvalidAsset,
    NotAllow,
}

impl<T: Config> From<AssetErr> for Error<T> {
    fn from(err: AssetErr) -> Self {
        match err {
            AssetErr::NotEnough => Error::<T>::InsufficientBalance,
            AssetErr::OverFlow => Error::<T>::Overflow,
            AssetErr::TotalAssetNotEnough => Error::<T>::TotalAssetInsufficientBalance,
            AssetErr::TotalAssetOverFlow => Error::<T>::TotalAssetOverflow,
            AssetErr::InvalidAsset => Error::<T>::InvalidAsset,
            AssetErr::NotAllow => Error::<T>::ActionNotAllowed,
        }
    }
}

/// A single lock on a balance. There can be many of these on an account and
/// they "overlap", so the same balance is frozen by multiple locks.
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct BalanceLock<Balance> {
    /// An identifier for this lock. Only one lock may be in existence for each
    /// identifier.
    pub id: LockIdentifier,
    /// The amount which the free balance may not drop below when this lock is
    /// in effect.
    pub amount: Balance,
}

#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct WithdrawalLimit<Balance> {
    pub minimal_withdrawal: Balance,
    pub fee: Balance,
}

/*
这段代码是区块链资产模块的一部分,它定义了资产类型,资产限制,资产信息结构以及与资产相关的错误类型.下面是对这些组件的详细解释:

### `AssetType` 枚举

`AssetType` 枚举定义了不同类型的资产余额,包括可用余额(Usable),锁定余额(Locked),预留余额(Reserved),
用于赎回桥接资产的预留余额(ReservedWithdrawal)和用于DEX订单的预留余额(ReservedDexSpot).这些类型用于区分资产的不同用途和状态.

### `AssetRestrictions` 位标志

`AssetRestrictions` 是一个位标志结构,用于定义资产操作的限制.它包括移动(MOVE),转移(TRANSFER),存款(DEPOSIT),提款(WITHDRAW),
销毁赎回余额(DESTROY_WITHDRAWAL)和销毁可用余额(DESTROY_USABLE)等操作的限制.位标志允许同时对多个限制进行测试和设置.

### `TotalAssetInfo` 结构体

`TotalAssetInfo` 结构体包含了资产的详细信息(`AssetInfo`),每种类型的资产余额(`balance`),资产是否在线(`is_online`)
以及资产的限制(`restrictions`).这个结构体提供了一个全面的视图,展示了某个资产的所有相关信息.

### `AssetErr` 枚举

`AssetErr` 枚举定义了与资产操作相关的错误类型,包括余额不足(NotEnough),溢出(OverFlow),总资产不足(TotalAssetNotEnough),
总资产溢出(TotalAssetOverFlow),无效资产(InvalidAsset)和不允许的操作(NotAllow).这些错误类型用于在资产操作失败时提供具体的错误信息.

### `Error` trait 实现

代码中还提供了一个将 `AssetErr` 转换为 `Error<T>` 的实现,其中 `T` 是配置 trait.这允许将资产相关的错误集成到框架的错误处理系统中.

### `BalanceLock` 结构体

`BalanceLock` 结构体表示对账户余额的锁定.它包含一个锁定标识符(`id`)和一个金额(`amount`),表示在锁定生效时可用余额不得低于该金额.
多个锁定可以同时存在于一个账户上,并且它们可以重叠.

### `WithdrawalLimit` 结构体

`WithdrawalLimit` 结构体定义了最小提款金额(`minimal_withdrawal`)和提款费用(`fee`).这可以用于实施提款策略,例如设置最小提款限额或收取提款手续费.

总的来说,这段代码为区块链资产模块提供了基础的数据结构和错误处理机制,以便在资产操作中进行有效的管理和控制.
*/
