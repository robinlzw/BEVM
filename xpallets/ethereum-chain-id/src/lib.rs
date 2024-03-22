// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

//! Minimal Pallet that stores the numeric Ethereum-style chain id in the runtime.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{pallet, traits::Get, weights::Weight};

pub use pallet::*;

#[pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    /// The Ethereum Chain Id Pallet
    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    /// Configuration trait of this pallet.
    #[pallet::config]
    pub trait Config: frame_system::Config {}

    impl<T: Config> Get<u64> for Pallet<T> {
        fn get() -> u64 {
            Self::chain_id()
        }
    }

    #[pallet::storage]
    #[pallet::getter(fn chain_id)]
    pub type ChainId<T> = StorageValue<_, u64, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig {
        pub chain_id: u64,
    }

    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            Self { chain_id: 1501u64 }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            ChainId::<T>::put(self.chain_id);
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(100_000_000u64)]
        pub fn set_chain_id(
            origin: OriginFor<T>,
            #[pallet::compact] new_chain_id: u64,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let _ = Self::set_chain_id_inner(new_chain_id);

            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn set_chain_id_inner(new_chain_id: u64) -> Weight {
        ChainId::<T>::mutate(|chain_id| *chain_id = new_chain_id);
        T::DbWeight::get().write
    }
}

/*

这段代码是一个 Substrate 框架的 Pallet,名为 `Ethereum Chain Id Pallet`,
它提供了在运行时存储以太坊风格的链 ID 的功能.链 ID 是一个数字,用于区分不同的以太坊网络,例如主网,测试网和其他私有网络.

### Pallet 结构

- `Pallet<T>`: Pallet 的结构体,使用 `PhantomData<T>` 来表示它与 Substrate 框架的特定配置类型 `T` 的关联.

### 配置和存储

- `Config` trait: 定义了 Pallet 配置的 trait,它继承自 `frame_system::Config`,但没有额外的字段.

- `ChainId`: 使用 `StorageValue` 类型定义了一个存储项,用于存储当前的链 ID.

- `GenesisConfig`: 定义了创世配置结构体,它包含一个 `chain_id` 字段,用于在区块链初始化时设置链 ID.

### 创世构建

- `GenesisBuild` trait: 实现了 `GenesisBuild` trait,允许在链的创世区块中设置初始链 ID.

### 调用函数

- `set_chain_id`: 一个调用来允许根账户更改链 ID.它使用 `set_chain_id_inner` 函数来实际更新存储中的链 ID.

### 辅助函数

- `set_chain_id_inner`: 一个内部函数,用于更新 `ChainId` 存储项的值,并返回操作的权重.

### 总结

这个 Pallet 为 Substrate 区块链提供了一个简单的机制来存储和更新链 ID,这对于与以太坊兼容的跨链操作非常重要.
通过在创世区块中设置链 ID,区块链可以明确其身份,从而确保与以太坊生态系统中的其他项目和服务兼容.
此外,`set_chain_id` 函数允许在区块链运行时更改链 ID,这在需要迁移到不同网络或进行测试时非常有用.
*/
