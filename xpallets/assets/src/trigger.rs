// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use frame_support::dispatch::DispatchResult;

use chainx_primitives::AssetId;

use crate::traits::OnAssetChanged;
use crate::types::{AssetErr, AssetType};
use crate::{BalanceOf, Config, Event, Pallet};

impl<AccountId, Balance> OnAssetChanged<AccountId, Balance> for () {}

pub struct AssetChangedTrigger<T: Config>(sp_std::marker::PhantomData<T>);

impl<T: Config> AssetChangedTrigger<T> {
    pub fn on_move_pre(
        id: &AssetId,
        from: &T::AccountId,
        from_type: AssetType,
        to: &T::AccountId,
        to_type: AssetType,
        value: BalanceOf<T>,
    ) {
        T::OnAssetChanged::on_move_pre(id, from, from_type, to, to_type, value);
    }

    pub fn on_move_post(
        id: &AssetId,
        from: &T::AccountId,
        from_type: AssetType,
        to: &T::AccountId,
        to_type: AssetType,
        value: BalanceOf<T>,
    ) -> Result<(), AssetErr> {
        Pallet::<T>::deposit_event(Event::<T>::Moved(
            *id,
            from.clone(),
            from_type,
            to.clone(),
            to_type,
            value,
        ));
        T::OnAssetChanged::on_move_post(id, from, from_type, to, to_type, value)?;
        Ok(())
    }

    pub fn on_issue_pre(id: &AssetId, who: &T::AccountId) {
        T::OnAssetChanged::on_issue_pre(id, who);
    }

    pub fn on_issue_post(
        id: &AssetId,
        who: &T::AccountId,
        value: BalanceOf<T>,
        reward_pcx: bool,
    ) -> DispatchResult {
        if reward_pcx {
            Pallet::<T>::deposit_event(Event::<T>::Issued(*id, who.clone(), value));
            T::OnAssetChanged::on_issue_post(id, who, value)?;
        }

        Ok(())
    }

    pub fn on_destroy_pre(id: &AssetId, who: &T::AccountId) {
        T::OnAssetChanged::on_destroy_pre(id, who);
    }

    pub fn on_destroy_post(
        id: &AssetId,
        who: &T::AccountId,
        value: BalanceOf<T>,
    ) -> DispatchResult {
        Pallet::<T>::deposit_event(Event::<T>::Destroyed(*id, who.clone(), value));
        T::OnAssetChanged::on_destroy_post(id, who, value)?;
        Ok(())
    }

    pub fn on_set_balance(
        id: &AssetId,
        who: &T::AccountId,
        type_: AssetType,
        value: BalanceOf<T>,
    ) -> DispatchResult {
        Pallet::<T>::deposit_event(Event::<T>::BalanceSet(*id, who.clone(), type_, value));
        T::OnAssetChanged::on_set_balance(id, who, type_, value)?;
        Ok(())
    }
}

/*
这段代码定义了一个名为 `AssetChangedTrigger` 的结构体,它用于在区块链资产模块中处理资产变动事件.
`AssetChangedTrigger` 结构体提供了一系列的方法,用于在资产被移动,发行,销毁或余额被设置时触发相应的事件.
这些方法与之前提到的 `OnAssetChanged` trait 中定义的钩子相对应.

### `AssetChangedTrigger` 结构体

`AssetChangedTrigger` 结构体包含一个类型参数 `T`,它依赖于配置 trait `Config`.这意味着 `AssetChangedTrigger` 
可以为任何实现了 `Config` trait 的类型生成实例.结构体内部使用了一个 `PhantomData<T>` 标记类型,
这表明 `AssetChangedTrigger` 并不持有任何 `T` 类型的值,但仍然依赖于 `T` 类型的配置.

### 方法

- `on_move_pre` 和 `on_move_post`:在资产转移之前和之后调用,用于检查和确认资产转移的操作.
`on_move_post` 方法还负责记录事件并返回一个 `Result` 类型,以便在操作失败时返回错误.
- `on_issue_pre` 和 `on_issue_post`:在资产发行之前和之后调用,用于处理新发行资产的逻辑.
如果 `reward_pcx` 参数为 `true`,则记录发行事件.
- `on_destroy_pre` 和 `on_destroy_post`:在资产销毁之前和之后调用,用于处理资产销毁的逻辑,并记录销毁事件.
- `on_set_balance`:在余额被设置时调用,用于处理余额更新的逻辑,并记录余额设置事件.

### 事件记录

`AssetChangedTrigger` 结构体中的每个方法都调用了 `Pallet::<T>::deposit_event`,
这是一个用于记录区块链事件的方法.这些事件可以被外部系统监听,以便根据资产变动执行相应的逻辑.

### 空实现

代码中还提供了一个空实现 `impl<AccountId, Balance> OnAssetChanged<AccountId, Balance> for () {}`.
这意味着如果某个区块链配置没有提供 `OnAssetChanged` trait 的实现,这个空实现将作为默认行为.
这是一种常见的模式,用于提供默认行为,以防配置 trait 没有被具体实现.

总的来说,这段代码通过 `AssetChangedTrigger` 结构体提供了一种机制,允许在资产变动时执行特定的逻辑,
并通过事件系统与区块链的其他部分进行交互.

----------------------------------------------------------------------------------------------------
`AssetChangedTrigger` 和 `OnAssetChanged` 之间的关联体现在它们都是为了处理资产变动事件而设计的,但它们的角色和使用方式有所不同.

### `OnAssetChanged` Trait(特征)

`OnAssetChanged` 是一个trait,定义了一系列在资产生命周期关键点(如发行,移动,销毁)时可以被调用的钩子函数.
这些钩子函数允许在这些事件发生前后执行特定的逻辑,例如验证条件,更新状态或记录日志.
`OnAssetChanged` trait 通常由区块链的资产模块实现,以便在资产变动时触发相应的处理逻辑.

### `AssetChangedTrigger` Struct(结构体)

`AssetChangedTrigger` 是一个结构体,它提供了一组方法来实际触发和处理由 `OnAssetChanged` trait 定义的事件.
这个结构体通常由资产模块的实现部分使用,以确保在资产变动时调用正确的钩子函数,并执行相关的逻辑,如记录事件或更新状态.

### 关联性

`AssetChangedTrigger` 结构体的方法直接调用了 `OnAssetChanged` trait 中定义的钩子函数.当资产变动发生时,
`AssetChangedTrigger` 的方法会被调用来执行预定义的逻辑,这些逻辑是通过 `OnAssetChanged` trait 的实现来提供的.
换句话说,`AssetChangedTrigger` 是一个执行器,它使用 `OnAssetChanged` trait 来定义资产变动时应采取的具体行动.

例如,当资产被转移时,`AssetChangedTrigger` 的 `on_move_pre` 和 `on_move_post` 方法会被调用,
这些方法内部会调用 `OnAssetChanged` trait 的 `on_move_pre` 和 `on_move_post` 钩子函数,以便在转移发生前后执行逻辑.

总结来说,`OnAssetChanged` trait 定义了资产变动事件的接口,而 `AssetChangedTrigger` 结构体提供了这些接口的具体实现和触发机制.
两者共同协作,确保资产变动事件得到妥善处理.
*/
