// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

//! This crate provides the feature of initializing the genesis state from ChainX 1.0.

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use xp_genesis_builder::AllParams;
#[cfg(feature = "std")]
use xpallet_assets::BalanceOf as AssetBalanceOf;
#[cfg(feature = "std")]
use xpallet_mining_staking::BalanceOf as StakingBalanceOf;

#[cfg(feature = "std")]
mod regenesis;

pub use self::pallet::*;

#[frame_support::pallet]
pub mod pallet {
    #[cfg(feature = "std")]
    use frame_support::traits::GenesisBuild;
    use sp_std::marker::PhantomData;

    use super::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(crate) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + pallet_balances::Config
        + xpallet_mining_asset::Config
        + xpallet_mining_staking::Config
    {
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub params: AllParams<T::AccountId, T::Balance, AssetBalanceOf<T>, StakingBalanceOf<T>>,
        pub initial_authorities: Vec<Vec<u8>>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                params: Default::default(),
                initial_authorities: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    #[cfg(feature = "std")]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            regenesis::initialize(self)
        }
    }
}

/*
这段代码是一个Rust库的一部分,它提供了从ChainX 1.0初始化创世状态(genesis state)的功能.
这个库是为Substrate框架设计的,Substrate是一个用于构建区块链应用的Rust库.

以下是代码的主要组成部分和它们的功能:

1. **`#![cfg_attr(not(feature = "std"), no_std)]`**:
   - 这是一个条件编译指令,它指定当不启用`std`特性时,应该编译为`no_std`环境.在WebAssembly (Wasm) 环境中,通常需要禁用Rust的标准库.

2. **`#[cfg(feature = "std")]`**:
   - 这个条件编译块只在启用了`std`特性时编译.在这个块中,它引入了一些依赖项,这些依赖项只在标准环境中可用.

3. **`mod regenesis;`**:
   - 这是一个模块声明,`regenesis.rs`文件应该包含与初始化创世状态相关的函数和逻辑.

4. **`pub use self::pallet::*;`**:
   - 这行代码将`pallet`模块中的所有公开项重新导出,使得外部代码可以直接使用它们.

5. **`#[frame_support::pallet]`**:
   - 这是一个属性宏,用于定义一个Substrate框架的Pallet.Pallet是Substrate中的一种模块化组件.

6. **`pub struct Pallet<T>(PhantomData<T>);`**:
   - 这是Pallet的结构体定义,它使用`PhantomData`来表示泛型参数`T`的存在,但并不实际存储它.
   这通常用于泛型类型中,以指示编译器关于类型的信息,而不实际使用该类型.

7. **`#[pallet::config]`**:
   - 这是一个属性宏,用于定义Pallet的配置特征.它指定了Pallet所需的几种其他配置.

8. **`#[pallet::genesis_config]`**:
   - 这个宏定义了一个用于创世配置的结构体,它包含了初始化创世状态所需的所有参数.

9. **`#[cfg(feature = "std")]`**:
   - 这个条件编译块定义了`GenesisConfig`的默认实现,它提供了默认的参数值.

10. **`#[pallet::genesis_build]`**:
    - 这个宏定义了一个实现了`GenesisBuild`特征的函数,这个函数在创世时被调用,用于构建创世状态.

11. **`fn build(&self)`**:
    - 这个函数是`GenesisBuild`特征的一部分,它定义了如何构建创世状态.在这个例子中,它调用了`regenesis::initialize`函数.

整体来看,这段代码提供了一个框架,用于在Substrate区块链中初始化创世状态.
它定义了所需的配置和参数,并指定了在区块链启动时执行的初始化逻辑.这对于升级或迁移区块链的创世状态非常有用.
*/
