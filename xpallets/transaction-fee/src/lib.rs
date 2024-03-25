// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

//! # Transaction Fee Module

#![cfg_attr(not(feature = "std"), no_std)]

mod types;

pub use self::types::FeeDetails;
pub use pallet_transaction_payment::InclusionFee;

type BalanceOf<T> = <<T as pallet_transaction_payment::Config>::OnChargeTransaction as pallet_transaction_payment::OnChargeTransaction<T>>::Balance;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(crate) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_transaction_payment::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::event]
    pub enum Event<T: Config> {
        /// Transaction fee was paid to the block author and its reward pot in 1:9.
        /// [author, author_fee, reward_pot, reward_pot_fee]
        FeePaid(T::AccountId, BalanceOf<T>, T::AccountId, BalanceOf<T>),
        /// Transaction BTC fee
        BTCFeePaid(T::AccountId, u128),
    }
}

/*
这段代码是ChainX区块链项目中的交易费用模块的一部分,它定义了交易费用的计算和处理方式.
模块使用了Substrate框架的`pallet`宏来创建一个pallet,并且定义了相关的事件和存储逻辑.

1. **types**:
   - 一个模块,其中包含了`FeeDetails`类型的定义,这个类型用于描述交易费用的详细信息.

2. **BalanceOf**:
   - 一个类型别名,表示特定于`Config`的余额类型.这是通过`pallet_transaction_payment`模块的`OnChargeTransaction` trait来定义的.

3. **Pallet**:
   - 使用`frame_support::pallet`宏定义的pallet结构体,它包含了`Config` trait的定义和事件枚举`Event`.

4. **Config**:
   - `Config` trait定义了模块的配置,它要求实现`frame_system::Config`和`pallet_transaction_payment::Config`.
   这允许模块使用Substrate框架提供的系统功能和交易费用计算功能.

5. **Event**:
   - 定义了两个事件:`FeePaid`和`BTCFeePaid`.`FeePaid`事件在交易费用支付给区块作者和奖励池时发出,包含了作者的账户ID,
   作者费用,奖励池账户ID和奖励池费用.`BTCFeePaid`事件用于记录交易的BTC费用支付情况.

6. **无存储逻辑**:
   - 通过`#[pallet::without_storage_info]`属性,表明这个pallet没有定义任何存储项.这可能是因为所有的存储逻辑都已经
   在`types`模块中定义,或者这个pallet的设计是作为一个轻量级的模块,只负责处理事件和费用计算.

这个交易费用模块的设计允许ChainX区块链项目对交易费用进行管理和分配.通过定义事件,它可以通知其他模块或外部观察者有关交易费用支付的情况.
此外,通过使用`pallet_transaction_payment`模块,它能够集成Substrate框架的交易费用计算和支付机制.
*/
