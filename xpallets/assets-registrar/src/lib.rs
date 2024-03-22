// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

//! This crate provides the feature of managing the native and foreign assets' meta information.
//!
//! The foreign asset hereby means it's not the native token of the system(PCX for ChainX)
//! but derived from the other blockchain system, e.g., Bitcoin.

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(any(feature = "runtime-benchmarks", test))]
mod benchmarking;
#[cfg(test)]
mod tests;
mod types;
mod verifier;
pub mod weights;

use sp_std::prelude::*;

use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    ensure,
    log::info,
};

use chainx_primitives::{AssetId, Desc, Token};

pub use self::types::AssetInfo;
pub use self::weights::WeightInfo;
pub use xp_assets_registrar::{Chain, RegistrarHandler};

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    /// The pallet's config trait.
    ///
    /// `frame_system::Trait` should always be included in our implied traits.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Native asset Id.
        type NativeAssetId: Get<AssetId>;

        /// Handler for doing stuff after the asset is registered/deregistered.
        type RegistrarHandler: RegistrarHandler;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a new foreign asset.
        ///
        /// This is a root-only operation.
        #[pallet::weight(T::WeightInfo::register())]
        pub fn register(
            origin: OriginFor<T>,
            #[pallet::compact] asset_id: AssetId,
            asset: AssetInfo,
            is_online: bool,
            has_mining_rights: bool,
        ) -> DispatchResult {
            ensure_root(origin)?;

            asset.is_valid::<T>()?;
            ensure!(!Self::exists(&asset_id), Error::<T>::AssetAlreadyExists);

            info!(
                target: "runtime::assets-registrar",
                "[register_asset] id:{}, info:{:?}, is_online:{}, has_mining_rights:{}",
                asset_id, asset, is_online, has_mining_rights
            );

            Self::apply_register(asset_id, asset)?;

            Self::deposit_event(Event::Registered(asset_id, has_mining_rights));
            T::RegistrarHandler::on_register(&asset_id, has_mining_rights)?;

            if !is_online {
                let _ = Self::deregister(frame_system::RawOrigin::Root.into(), asset_id);
            }

            Ok(())
        }

        /// Deregister an asset with given `id`.
        ///
        /// This asset will be marked as invalid.
        ///
        /// This is a root-only operation.
        #[pallet::weight(T::WeightInfo::deregister())]
        pub fn deregister(origin: OriginFor<T>, #[pallet::compact] id: AssetId) -> DispatchResult {
            ensure_root(origin)?;

            ensure!(Self::is_valid(&id), Error::<T>::AssetIsInvalid);

            AssetOnline::<T>::remove(id);

            Self::deposit_event(Event::Deregistered(id));
            T::RegistrarHandler::on_deregister(&id)?;
            Ok(())
        }

        /// Recover a deregister asset to the valid state.
        ///
        /// `RegistrarHandler::on_register()` will be triggered again during the recover process.
        ///
        /// This is a root-only operation.
        #[pallet::weight(T::WeightInfo::recover())]
        pub fn recover(
            origin: OriginFor<T>,
            #[pallet::compact] id: AssetId,
            has_mining_rights: bool,
        ) -> DispatchResult {
            ensure_root(origin)?;

            ensure!(Self::exists(&id), Error::<T>::AssetDoesNotExist);
            ensure!(!Self::is_valid(&id), Error::<T>::AssetAlreadyValid);

            AssetOnline::<T>::insert(id, true);

            Self::deposit_event(Event::Recovered(id, has_mining_rights));
            T::RegistrarHandler::on_register(&id, has_mining_rights)?;
            Ok(())
        }

        /// Update the asset info, all the new fields are optional.
        ///
        /// This is a root-only operation.
        #[pallet::weight(T::WeightInfo::update_asset_info())]
        pub fn update_asset_info(
            origin: OriginFor<T>,
            #[pallet::compact] id: AssetId,
            token: Option<Token>,
            token_name: Option<Token>,
            desc: Option<Desc>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let mut info = Self::asset_info_of(&id).ok_or(Error::<T>::AssetDoesNotExist)?;
            if let Some(t) = token {
                info.set_token(t)
            }
            if let Some(name) = token_name {
                info.set_token_name(name);
            }
            if let Some(desc) = desc {
                info.set_desc(desc);
            }
            AssetInfoOf::<T>::insert(id, info);
            Ok(())
        }
    }

    /// Event for the XAssetRegistrar Pallet
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new asset was registered. [asset_id, has_mining_rights]
        Registered(AssetId, bool),
        /// A deregistered asset was recovered. [asset_id, has_mining_rights]
        Recovered(AssetId, bool),
        /// An asset was deregistered. [asset_id]
        Deregistered(AssetId),
    }

    /// Error for the XAssetRegistrar Pallet
    #[pallet::error]
    pub enum Error<T> {
        /// Token symbol length is zero or too long
        InvalidAssetTokenSymbolLength,
        /// Token symbol char is invalid, only allow ASCII alphanumeric character or '-', '.', '|', '~'
        InvalidAssetTokenSymbolChar,
        /// Token name length is zero or too long
        InvalidAssetTokenNameLength,
        /// Desc length is zero or too long
        InvalidAssetDescLength,
        /// Text is invalid ASCII, only allow ASCII visible character [0x20, 0x7E]
        InvalidAscii,
        /// The asset already exists.
        AssetAlreadyExists,
        /// The asset does not exist.
        AssetDoesNotExist,
        /// The asset is already valid (online), no need to recover.
        AssetAlreadyValid,
        /// The asset is invalid (not online).
        AssetIsInvalid,
    }

    /// Asset id list for each Chain.
    #[pallet::storage]
    #[pallet::getter(fn asset_ids_of)]
    pub(super) type AssetIdsOf<T: Config> =
        StorageMap<_, Twox64Concat, Chain, Vec<AssetId>, ValueQuery>;

    /// Asset info of each asset.
    #[pallet::storage]
    #[pallet::getter(fn asset_info_of)]
    pub(super) type AssetInfoOf<T: Config> = StorageMap<_, Twox64Concat, AssetId, AssetInfo>;

    /// The map of asset to the online state.
    #[pallet::storage]
    #[pallet::getter(fn asset_online)]
    pub(super) type AssetOnline<T: Config> = StorageMap<_, Twox64Concat, AssetId, bool, ValueQuery>;

    /// The map of asset to the block number at which the asset was registered.
    #[pallet::storage]
    #[pallet::getter(fn registered_at)]
    pub(super) type RegisteredAt<T: Config> =
        StorageMap<_, Twox64Concat, AssetId, T::BlockNumber, ValueQuery>;

    /// add_extra_genesis
    #[pallet::genesis_config]
    #[cfg_attr(feature = "std", derive(Default))]
    pub struct GenesisConfig {
        pub assets: Vec<(AssetId, AssetInfo, bool, bool)>,
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            let extra_genesis_builder: fn(&Self) = |config| {
                for (id, asset, is_online, has_mining_rights) in &config.assets {
                    Pallet::<T>::register(
                        frame_system::RawOrigin::Root.into(),
                        *id,
                        asset.clone(),
                        *is_online,
                        *has_mining_rights,
                    )
                    .expect("asset registeration during the genesis can not fail");
                }
            };
            extra_genesis_builder(self);
        }
    }
}

impl<T: Config> Pallet<T> {
    /// Returns an iterator of all the asset ids of all chains so far.
    #[inline]
    pub fn asset_ids() -> impl Iterator<Item = AssetId> {
        Chain::iter().map(Self::asset_ids_of).flatten()
    }

    /// Returns an iterator of all the valid asset ids of all chains so far.
    #[inline]
    pub fn valid_asset_ids() -> impl Iterator<Item = AssetId> {
        Self::asset_ids().filter(Self::is_valid)
    }

    /// Returns an iterator of tuple (AssetId, AssetInfo) of all assets.
    #[inline]
    pub fn asset_infos() -> impl Iterator<Item = (AssetId, AssetInfo)> {
        AssetInfoOf::<T>::iter()
    }

    /// Returns an iterator of tuple (AssetId, AssetInfo) of all valid assets.
    #[inline]
    pub fn valid_asset_infos() -> impl Iterator<Item = (AssetId, AssetInfo)> {
        Self::asset_infos().filter(|(id, _)| Self::is_valid(id))
    }

    /// Returns the chain of given asset `asset_id`.
    pub fn chain_of(asset_id: &AssetId) -> Result<Chain, DispatchError> {
        Self::asset_info_of(asset_id)
            .map(|info| info.chain())
            .ok_or_else(|| Error::<T>::AssetDoesNotExist.into())
    }

    /// Returns the asset info of given `id`.
    pub fn get_asset_info(id: &AssetId) -> Result<AssetInfo, DispatchError> {
        if let Some(asset) = Self::asset_info_of(id) {
            if Self::is_valid(id) {
                Ok(asset)
            } else {
                Err(Error::<T>::AssetIsInvalid.into())
            }
        } else {
            Err(Error::<T>::AssetDoesNotExist.into())
        }
    }

    /// Returns true if the given `asset_id` is an online asset.
    pub fn is_online(asset_id: &AssetId) -> bool {
        Self::asset_online(asset_id)
    }

    /// Returns true if the asset info record of given `asset_id` exists.
    pub fn exists(asset_id: &AssetId) -> bool {
        Self::asset_info_of(asset_id).is_some()
    }

    /// Returns true if the asset of given `asset_id` is valid (only check if still online currently).
    pub fn is_valid(asset_id: &AssetId) -> bool {
        Self::is_online(asset_id)
    }

    /// Helper function for checking the asset's existence.
    pub fn ensure_asset_exists(id: &AssetId) -> DispatchResult {
        ensure!(Self::exists(id), Error::<T>::AssetDoesNotExist);
        Ok(())
    }

    /// Helper function for checking the asset's validity.
    pub fn ensure_asset_is_valid(id: &AssetId) -> DispatchResult {
        ensure!(Self::is_valid(id), Error::<T>::AssetIsInvalid);
        Ok(())
    }

    /// Actually register an asset.
    fn apply_register(id: AssetId, asset: AssetInfo) -> DispatchResult {
        let chain = asset.chain();
        AssetIdsOf::<T>::mutate(chain, |ids| {
            if !ids.contains(&id) {
                ids.push(id);
            }
        });

        AssetInfoOf::<T>::insert(&id, asset);
        AssetOnline::<T>::insert(&id, true);

        RegisteredAt::<T>::insert(&id, frame_system::Pallet::<T>::block_number());

        Ok(())
    }
}

/*

这段代码是一个Rust库,它是为Substrate框架编写的,用于管理本地和外来资产的元信息.
本地资产指的是系统的原生代币(例如ChainX的PCX),而外来资产则是从其他区块链系统(如比特币)派生而来的代币.

这个库提供了一系列的功能,包括注册新的外来资产,注销资产,恢复已注销的资产以及更新资产信息.
这些功能通过定义在`pallet`模块中的一系列函数来实现,每个函数都有相应的权限检查和逻辑处理.

以下是代码中定义的主要组件和它们的功能:

1. **AssetInfo**:定义了资产信息的结构,包括代币符号,名称,链类型,精度,描述等.

2. **Config**:定义了Pallet配置 trait,其中包括事件类型,本地资产ID,注册处理器,权重信息等.

3. **Pallet**:定义了资产管理的核心逻辑,包括注册,注销,恢复和更新资产信息的函数.
这些函数使用了`frame_support`库中的`dispatch`和`ensure`宏来处理权限和逻辑验证.

4. **Event**:定义了Pallet可以发出的事件类型,包括资产注册,恢复和注销事件.

5. **Error**:定义了Pallet可能遇到的错误类型,包括无效的资产符号长度,资产已存在,资产不存在,资产已有效等.

6. **Storage**:定义了Pallet使用的存储结构,包括资产ID列表,资产信息,资产在线状态和注册时间戳.

7. **GenesisConfig**:定义了创世配置结构,允许在链的创世时注册一系列资产.

8. **Benchmarking**:定义了基准测试模块,用于测试Pallet的性能.

9. **Weights**:定义了Pallet中各种操作的权重信息,这对于调整交易费用和网络拥堵管理非常重要.

整个库的设计旨在为Substrate区块链提供资产管理功能,允许系统管理员注册和注销外来资产,
并管理这些资产的状态.通过这种方式,它可以支持多种不同区块链上的资产在Substrate链上的互操作性.

------------------------------------------------------------------------------------------------
`frame_support`库是Substrate框架中的一个核心组件,它提供了一系列用于构建区块链节点和智能合约的基础工具和宏.
这个库的目的是简化和标准化区块链智能合约和运行时逻辑的开发过程.以下是`frame_support`库中的一些关键特性和组件:

1. **Pallet**:Pallet是Substrate中用于构建特定功能模块的模板.`frame_support`提供了`pallet`宏,
它允许开发者定义自己的Pallet,并指定配置,存储,错误处理,事件和其他与Pallet相关的功能.

2. **DispatchError** 和 **DispatchResult**:这些类型用于处理交易的结果.`DispatchResult`是一个结果类型,
可以包含成功或失败的信息.`DispatchError`是一个枚举,定义了可能发生的错误类型,允许开发者在交易失败时提供清晰的错误信息.

3. **Ensure** 和 **EnsureOrigin**:这些宏用于验证交易的来源是否满足特定条件.
例如,`ensure_root`用于检查交易是否由系统的根账户发起,这是一种常见的权限检查.

4. **Storage**:`frame_support`提供了一系列的宏和工具,用于定义和管理区块链的状态(存储).这些工具允许开发者以类型安全的方式存储和检索数据.

5. **Event**:`Event`宏用于定义和发出区块链事件.事件是区块链上发生的重要状态变化的记录,可以被外部观察者监听和响应.

6. **Weight** 和 **WeightInfo**:这些组件用于定义和计算交易的权重,这是Substrate中用于衡量交易复杂性和资源消耗的机制.权重信息对于交易费用和区块生产者的奖励计算非常重要.

7. **GenesisConfig** 和 **GenesisBuild**:这些组件用于在区块链的创世区块中定义初始状态.开发者可以指定在区块链启动时应该包含的初始数据和配置.

8. **RuntimeDebug** 和 **RuntimeApplicable**:这些特性(traits)用于提供运行时调试信息和确保类型在运行时可以应用.

9. **Parameter** 和 **StorageValue**:这些宏和工具用于处理参数和存储值的序列化和反序列化.

`frame_support`库是Substrate框架中不可或缺的一部分,它为开发者提供了构建复杂区块链应用所需的基础设施.
通过使用`frame_support`,开发者可以专注于业务逻辑的实现,而不必担心底层的区块链逻辑和状态管理.

*/

