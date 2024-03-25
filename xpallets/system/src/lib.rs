// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{collections::btree_map::BTreeMap, prelude::*};

use sp_runtime::traits::StaticLookup;

use frame_support::{
    dispatch::{CallMetadata, DispatchResult},
    traits::Currency,
};

use frame_system::ensure_root;
use xp_protocol::NetworkType;

pub use pallet::*;

const PALLET_MARK: &[u8; 1] = b"#";
const ALWAYS_ALLOW: [&str; 1] = ["Sudo"];

/// The pallet's config trait.
///
/// `frame_system::Config` should always be included in our implied traits.
#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    /// The pallet's config trait.
    ///
    /// `frame_system::Config` should always be included in our implied traits.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The currency mechanism.
        type Currency: Currency<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(crate) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Modify the paused status of the given pallet call.
        ///
        /// This is a root-only operation.
        #[pallet::weight(0)]
        pub fn modify_paused(
            origin: OriginFor<T>,
            pallet: Vec<u8>,
            call: Option<Vec<u8>>,
            should_paused: bool,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let mut paused = Self::paused(&pallet);

            if should_paused {
                if let Some(c) = call {
                    // pause the call of the pallet
                    paused.insert(c, ());
                } else {
                    // pause the whole calls of the pallet
                    paused.insert(PALLET_MARK.to_vec(), ());
                }
            } else if let Some(c) = call {
                // revoke the paused status of the call in the pallet
                paused.remove(&c[..]);
            } else {
                // revoke the paused status of the whole calls in the pallet.
                paused.remove(&PALLET_MARK[..]);
            }

            if paused.is_empty() {
                Paused::<T>::remove(&pallet);
            } else {
                Paused::<T>::insert(pallet, paused);
            }
            Ok(())
        }

        /// Toggle the blacklist status of the given account id.
        ///
        /// This is a root-only operation.
        #[pallet::weight(0)]
        pub fn toggle_blacklist(
            origin: OriginFor<T>,
            who: <T::Lookup as StaticLookup>::Source,
            should_blacklist: bool,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let who = T::Lookup::lookup(who)?;
            if should_blacklist {
                Blacklist::<T>::insert(who.clone(), true);
                Self::deposit_event(Event::<T>::Blacklisted(who))
            } else {
                Blacklist::<T>::remove(&who);
                Self::deposit_event(Event::<T>::Unblacklisted(who));
            }
            Ok(())
        }
    }

    /// Event for the XSystem Pallet
    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// An account was added to the blacklist. [who]
        Blacklisted(T::AccountId),
        /// An account was removed from the blacklist. [who]
        Unblacklisted(T::AccountId),
    }

    /// Network property (Mainnet / Testnet).
    #[pallet::storage]
    #[pallet::getter(fn network_props)]
    pub type NetworkProps<T> = StorageValue<_, NetworkType, ValueQuery>;

    /// Paused pallet call
    #[pallet::storage]
    #[pallet::getter(fn paused)]
    pub type Paused<T> = StorageMap<_, Twox64Concat, Vec<u8>, BTreeMap<Vec<u8>, ()>, ValueQuery>;

    /// The accounts that are blocked
    #[pallet::storage]
    #[pallet::getter(fn blacklist)]
    pub type Blacklist<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

    #[pallet::genesis_config]
    #[cfg_attr(feature = "std", derive(Default))]
    pub struct GenesisConfig {
        pub network_props: NetworkType,
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            NetworkProps::<T>::put(self.network_props);
        }
    }
}

impl<T: Config> Pallet<T> {
    /// Returns true if the given pallet call has been paused.
    pub fn is_paused(metadata: CallMetadata) -> bool {
        if ALWAYS_ALLOW.contains(&metadata.pallet_name) {
            return false;
        }

        let p = Self::paused(metadata.pallet_name.as_bytes());
        // check whether this pallet has been paused
        if p.get(&PALLET_MARK[..]).is_some() {
            return true;
        }
        // check whether this pallet call has been paused
        if p.get(metadata.function_name.as_bytes()).is_some() {
            return true;
        }
        // no pause
        false
    }

    /// Returns the blocked account id list.
    pub fn get_blacklist() -> Vec<T::AccountId> {
        Blacklist::<T>::iter()
            .filter_map(|(account, blocked)| if blocked { Some(account) } else { None })
            .collect()
    }
}

/*
这段代码是一个Substrate框架的区块链项目中的一个名为`xsystem`的pallet(模块).
这个pallet提供了一些系统级别的功能,包括暂停特定pallet的调用,管理黑名单账户等.
以下是对代码中定义的trait,结构体,函数和存储项的详细解释:

1. **MultisigAddressFor**:
   - 一个trait,用于计算多签名地址.它接受一个账户ID的切片和一个阈值作为参数,并返回一个多签名地址.

2. **MultiSig**:
   - 一个trait,定义了一个`multisig`函数,用于获取多签名地址.

3. **Validator**:
   - 一个trait,定义了`is_validator`和`validator_for`函数,用于检查账户是否是验证者以及根据名称获取验证者账户ID.

4. **TreasuryAccount**:
   - 一个trait,定义了`treasury_account`函数,用于获取国库账户ID.

5. **pallet**:
   - 使用`frame_support::pallet`宏定义的pallet模块,包含了`Config` trait,`Pallet`结构体,`Call`接口,`Event`枚举和存储项.

6. **Config**:
   - `xsystem` pallet的配置trait,要求实现`frame_system::Config`,并添加了`Event`和`Currency`类型.

7. **Pallet**:
   - `Pallet`结构体,使用`PhantomData<T>`来表示与`Config`相关的类型.

8. **Call**:
   - 定义了`modify_paused`和`toggle_blacklist`函数,用于修改pallet调用的暂停状态和切换账户的黑名单状态.

9. **Event**:
   - 定义了`Blacklisted`和`Unblacklisted`事件,用于通知账户被添加到黑名单或从黑名单中移除.

10. **Storage**:
    - 定义了`NetworkProps`,`Paused`和`Blacklist`存储项,用于存储网络属性,暂停的pallet调用和黑名单账户.

11. **GenesisConfig**:
    - 定义了`GenesisConfig`结构体,用于在创世区块时设置网络属性.

12. **GenesisBuild**:
    - 实现了`GenesisBuild` trait,用于在创世区块时初始化`NetworkProps`.

13. **辅助函数**:
    - `is_paused`函数用于检查给定的pallet调用是否已暂停.
    - `get_blacklist`函数用于获取当前的黑名单账户列表.

这个pallet提供了一些基本的系统管理功能,允许区块链的管理员(通常是root账户或sudo权限的账户)管理pallet的调用和账户的黑名单状态.
*/
