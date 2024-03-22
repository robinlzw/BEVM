// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use frame_support::dispatch::{DispatchError, DispatchResult};

use chainx_primitives::AssetId;
use xpallet_assets_registrar::Chain;

use crate::types::{AssetErr, AssetType, WithdrawalLimit};

pub trait ChainT<Balance: Default> {
    /// ASSET should be the native Asset for this chain.
    /// e.g.
    ///     if ChainT for Bitcoin, then ASSET is X_BTC
    ///     if ChainT for Ethereum, then ASSET is X_ETH
    ///     if ChainT for Polkadot, then ASSET is X_DOT
    const ASSET_ID: AssetId;
    fn chain() -> Chain;
    fn check_addr(_addr: &[u8], _ext: &[u8]) -> DispatchResult {
        Ok(())
    }
    fn withdrawal_limit(_asset_id: &AssetId) -> Result<WithdrawalLimit<Balance>, DispatchError> {
        Ok(WithdrawalLimit::default())
    }
}

/// Hooks for doing stuff when the assets are minted/moved/destroyed.
pub trait OnAssetChanged<AccountId, Balance> {
    /// Triggered before issuing the fresh assets.
    fn on_issue_pre(_id: &AssetId, _who: &AccountId) {}

    /// Triggered after issuing the fresh assets.
    fn on_issue_post(_id: &AssetId, _who: &AccountId, _value: Balance) -> DispatchResult {
        Ok(())
    }

    /// Triggered before moving the assets.
    fn on_move_pre(
        _id: &AssetId,
        _from: &AccountId,
        _from_type: AssetType,
        _to: &AccountId,
        _to_type: AssetType,
        _value: Balance,
    ) {
    }

    /// Triggered after moving the assets.
    fn on_move_post(
        _id: &AssetId,
        _from: &AccountId,
        _from_type: AssetType,
        _to: &AccountId,
        _to_type: AssetType,
        _value: Balance,
    ) -> Result<(), AssetErr> {
        Ok(())
    }

    /// Triggered before destroying the assets.
    fn on_destroy_pre(_id: &AssetId, _who: &AccountId) {}

    /// Triggered after the assets has been destroyed.
    fn on_destroy_post(_id: &AssetId, _who: &AccountId, _value: Balance) -> DispatchResult {
        Ok(())
    }

    /// Triggered after the balance has been set to a new value.
    fn on_set_balance(
        _id: &AssetId,
        _who: &AccountId,
        _type: AssetType,
        _value: Balance,
    ) -> DispatchResult {
        Ok(())
    }
}

/*

这段代码定义了两个主要的trait(特征):`ChainT`和`OnAssetChanged`,它们是用于区块链资产模块的一部分.
这些trait为资产的发行,移动和销毁等操作提供了钩子(hooks),允许在这些事件发生前后执行额外的逻辑.

### `ChainT` Trait(特征)

`ChainT`是一个泛型trait,它定义了一个区块链应该具备的属性.对于每个区块链,都会有一个对应的资产ID(例如,
    对于比特币链,资产ID可能是`X_BTC`).这个trait要求实现以下方法:

- `ASSET_ID`:一个常量,表示该链的原生资产ID.
- `chain()`:返回表示链类型的`Chain`枚举.
- `check_addr()`:一个方法,用于检查地址的有效性.这里的实现是一个空实现,因为在trait中没有提供具体的逻辑.
- `withdrawal_limit()`:返回一个关于资产提款限制的结构体.默认实现返回一个空的`WithdrawalLimit`结构体.

### `OnAssetChanged` Trait(特征)

`OnAssetChanged` trait定义了一系列在资产发生变化时触发的钩子.这些钩子允许在资产被发行,移动或销毁前后执行特定的逻辑.这个trait要求实现以下方法:

- `on_issue_pre()`和`on_issue_post()`:在发行新资产之前和之后触发.
- `on_move_pre()`和`on_move_post()`:在资产转移之前和之后触发.`on_move_post()`返回`Result<(), AssetErr>`,这意味着它可能会返回一个错误.
- `on_destroy_pre()`和`on_destroy_post()`:在销毁资产之前和之后触发.
- `on_set_balance()`:在余额被设置为新值之后触发.这个方法返回`DispatchResult`,允许它返回更广泛的错误类型.

这些trait为区块链资产模块提供了灵活性,允许根据不同的区块链和资产类型来自定义行为.例如,
一个实现`ChainT`的比特币链可能会有一个特定的资产ID和提款限制,而以太坊链则会有另一个.通过实现`OnAssetChanged`,
模块可以在资产生命周期的关键点执行额外的检查或更新,例如记录事件,更新状态或执行其他链上逻辑.
*/
