// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use frame_support::log::{debug, error, info, warn};
use sp_std::{collections::btree_map::BTreeMap, prelude::*};

use chainx_primitives::{AssetId, ChainAddress, ReferralId};
use xp_gateway_bitcoin::{BtcDepositInfo, OpReturnAccount};
use xp_gateway_common::{transfer_aptos_uncheck, transfer_evm_uncheck, DstChain, DstChainConfig};
use xpallet_assets::Chain;
use xpallet_support::{traits::Validator, try_addr, try_str};

use crate::traits::{AddressBinding, ReferralBinding};
use crate::{
    AddressBindingOf, AddressBindingOfDstChain, BoundAddressOf, BoundAddressOfDstChain, Config,
    DefaultDstChain, DstChainProxyAddress, NamedDstChainConfig, Pallet,
};

/// Update the referrer's binding
impl<T: Config> ReferralBinding<T::AccountId> for Pallet<T> {
    fn update_binding(asset_id: &AssetId, who: &T::AccountId, referral_name: Option<ReferralId>) {
        let chain = match xpallet_assets_registrar::Pallet::<T>::chain_of(asset_id) {
            Ok(chain) => chain,
            Err(err) => {
                error!(
                    target: "runtime::gateway::common",
                    "[update_referral_binding] Unexpected asset_id:{:?}, error:{:?}",
                    asset_id, err
                );
                return;
            }
        };

        if let Some(name) = referral_name {
            if let Some(referral) = T::Validator::validator_for(&name) {
                match Self::referral_binding_of(who, chain) {
                    None => {
                        // set to storage
                        Self::set_referral_binding(chain, who.clone(), referral);
                    }
                    Some(channel) => {
                        debug!(
                            target: "runtime::gateway::common",
                            "[update_referral_binding] Already has referral binding:[assert id:{}, chain:{:?}, who:{:?}, referral:{:?}]",
                            asset_id, chain, who, channel
                        );
                    }
                }
            } else {
                warn!(
                    target: "runtime::gateway::common",
                    "[update_referral_binding] {:?} has no referral, cannot update binding",
                    try_str(name)
                );
            };
        };
    }

    fn referral(asset_id: &AssetId, who: &T::AccountId) -> Option<T::AccountId> {
        let chain = xpallet_assets_registrar::Pallet::<T>::chain_of(asset_id).ok()?;
        Self::referral_binding_of(who, chain)
    }
}

/// Update the binding of user deposit address
impl<T: Config, Address: Into<Vec<u8>>> AddressBinding<T::AccountId, Address> for Pallet<T> {
    fn update_binding(chain: Chain, address: Address, who: OpReturnAccount<T::AccountId>) {
        match who {
            OpReturnAccount::Evm(w) => Pallet::<T>::update_dst_chain_binding(
                chain,
                DstChain::ChainXEvm,
                address,
                w.as_bytes().to_vec(),
            ),
            OpReturnAccount::Wasm(w) => Pallet::<T>::update_wasm_binding(chain, address, w),
            OpReturnAccount::Aptos(w) => Pallet::<T>::update_dst_chain_binding(
                chain,
                DstChain::Aptos,
                address,
                w.as_bytes().to_vec(),
            ),
            OpReturnAccount::Named(prefix, w) => {
                Pallet::<T>::update_dst_chain_binding(chain, DstChain::Named(prefix), address, w)
            }
        }
    }

    fn check_allowed_binding(info: BtcDepositInfo<T::AccountId>) -> BtcDepositInfo<T::AccountId> {
        let op_return = if let Some((account, refererid)) = info.op_return {
            match &account {
                OpReturnAccount::Named(prefix, addr) => {
                    let deposit_config = DstChainConfig::new(prefix, addr.len() as u32);
                    let config = NamedDstChainConfig::<T>::get();
                    if config.contains(&deposit_config)
                        && DstChainProxyAddress::<T>::get(DstChain::Named(prefix.clone())).is_some()
                    {
                        Some((account, refererid))
                    } else {
                        None
                    }
                }
                OpReturnAccount::Aptos(_) => {
                    if DstChainProxyAddress::<T>::get(DstChain::Aptos).is_some() {
                        Some((account, refererid))
                    } else {
                        None
                    }
                }
                _ => Some((account, refererid)),
            }
        } else {
            None
        };
        BtcDepositInfo { op_return, ..info }
    }

    fn dst_chain_proxy_address(dst_chain: DstChain) -> Option<T::AccountId> {
        DstChainProxyAddress::<T>::get(&dst_chain)
    }

    fn address(chain: Chain, address: Address) -> Option<OpReturnAccount<T::AccountId>> {
        let addr_bytes: ChainAddress = address.into();
        let default_dst_chain = DefaultDstChain::<T>::get(&addr_bytes)?;

        match default_dst_chain {
            DstChain::ChainX => {
                let wasm_addr = AddressBindingOf::<T>::get(chain, &addr_bytes)?;
                Some(OpReturnAccount::Wasm(wasm_addr))
            }
            DstChain::ChainXEvm => {
                let evm_raw_addr =
                    AddressBindingOfDstChain::<T>::get((chain, DstChain::ChainXEvm, &addr_bytes))?;
                let evm_addr = transfer_evm_uncheck(&evm_raw_addr)?;
                Some(OpReturnAccount::Evm(evm_addr))
            }
            DstChain::Aptos => {
                let aptos_raw_addr =
                    AddressBindingOfDstChain::<T>::get((chain, DstChain::Aptos, &addr_bytes))?;
                let aptos_addr = transfer_aptos_uncheck(&aptos_raw_addr)?;
                Some(OpReturnAccount::Aptos(aptos_addr))
            }
            DstChain::Named(prefix) => {
                let named_addr = AddressBindingOfDstChain::<T>::get((
                    chain,
                    DstChain::Named(prefix.clone()),
                    &addr_bytes,
                ))?;
                Some(OpReturnAccount::Named(prefix, named_addr))
            }
        }
    }
}

// export for runtime-api
impl<T: Config> Pallet<T> {
    // todo! Add find of evm address
    pub fn bound_addrs(who: &T::AccountId) -> BTreeMap<Chain, Vec<ChainAddress>> {
        BoundAddressOf::<T>::iter_prefix(&who).collect()
    }

    fn update_wasm_binding<Address>(chain: Chain, address: Address, who: T::AccountId)
    where
        Address: Into<Vec<u8>>,
    {
        let address = address.into();
        if let Some(accountid) = AddressBindingOf::<T>::get(chain, &address) {
            if accountid != who {
                debug!(
                    target: "runtime::gateway::common",
                    "[update_address_binding] Current address binding need to changed (old:{:?} => new:{:?})",
                    accountid, who
                );
                // old accountid is not equal to new accountid, means should change this addr bind to new account
                // remove this addr for old accounid's CrossChainBindOf
                BoundAddressOf::<T>::mutate(accountid, chain, |addr_list| {
                    addr_list.retain(|addr| addr != &address);
                });
            }
        }
        // insert or override binding relationship
        BoundAddressOf::<T>::mutate(&who, chain, |addr_list| {
            if !addr_list.contains(&address) {
                addr_list.push(address.clone());
            }
        });

        info!(
            target: "runtime::gateway::common",
            "[update_address_binding] Update address binding:[chain:{:?}, addr:{:?}, who:{:?}]",
            chain,
            try_addr(&address),
            who,
        );
        AddressBindingOf::<T>::insert(chain, address.clone(), who);
        DefaultDstChain::<T>::insert(address, DstChain::ChainX);
    }

    fn update_dst_chain_binding<Address>(
        chain: Chain,
        dst_chain: DstChain,
        address: Address,
        who: ChainAddress,
    ) where
        Address: Into<Vec<u8>>,
    {
        let address = address.into();
        if let Some(accountid) = AddressBindingOfDstChain::<T>::get((chain, &dst_chain, &address)) {
            if accountid != who {
                debug!(
                    target: "runtime::gateway::common",
                    "[update_address_binding] Current address binding need to changed (old:{:?} => new:{:?})",
                    accountid, who
                );
                // old accountid is not equal to new accountid, means should change this addr bind to new account
                // remove this addr for old accounid's CrossChainBindOf
                BoundAddressOfDstChain::<T>::mutate((accountid, chain, &dst_chain), |addr_list| {
                    addr_list.retain(|addr| addr != &address);
                });
            }
        }
        // insert or override binding relationship
        BoundAddressOfDstChain::<T>::mutate((&who, chain, &dst_chain), |addr_list| {
            if !addr_list.contains(&address) {
                addr_list.push(address.clone());
            }
        });

        info!(
            target: "runtime::gateway::common",
            "[update_address_binding] Update address binding:[chain:{:?}, addr:{:?}, who:{:?}]",
            chain,
            try_addr(&address),
            who,
        );
        AddressBindingOfDstChain::<T>::insert((chain, dst_chain.clone(), address.clone()), who);
        DefaultDstChain::<T>::insert(address, dst_chain);
    }
}

/*
这段代码是 ChainX 区块链项目中的一个 pallet,它实现了 `ReferralBinding` 和 `AddressBinding` trait,
用于管理用户存款地址的绑定以及推荐人的绑定.这些绑定对于跨链资产转移和追踪推荐关系至关重要.

### 主要功能:

1. **更新推荐人绑定 (`update_binding`)**:
   - `ReferralBinding` trait 的实现,用于更新或设置特定资产的推荐人绑定.如果提供了推荐人名称,
   它会检查该推荐人是否存在,并设置或更新绑定.

2. **获取推荐人 (`referral`)**:
   - `ReferralBinding` trait 的实现,用于获取特定用户和资产的推荐人.

3. **更新用户存款地址绑定 (`update_binding`)**:
   - `AddressBinding` trait 的实现,用于更新或设置用户的存款地址绑定.
   这包括对不同链(如 EVM,Aptos,Named)的地址进行处理.

4. **检查允许的绑定 (`check_allowed_binding`)**:
   - `AddressBinding` trait 的实现,用于检查 BTC 存款信息中的操作返回地址是否允许绑定.

5. **获取目标链的代理地址 (`dst_chain_proxy_address`)**:
   - `AddressBinding` trait 的实现,用于获取目标链的代理地址.

6. **根据地址获取用户 (`address`)**:
   - `AddressBinding` trait 的实现,用于根据给定的地址和链类型解析出用户的 `OpReturnAccount`.

7. **导出运行时 API (`bound_addrs`)**:
   - 提供一个函数,允许外部查询给定用户的绑定地址.

### 辅助函数:

- **`update_wasm_binding`**:
  - 用于更新 Wasm 链上的地址绑定.

- **`update_dst_chain_binding`**:
  - 用于更新目标链上的地址绑定.

### 存储修改:

- 这些函数通过修改 `BoundAddressOf` 和 `AddressBindingOf` 等存储项来更新绑定信息.

### 日志记录:

- 使用 `debug`, `error`, `info`, `warn` 等日志记录函数来跟踪绑定操作的状态和潜在的错误.

这段代码通过提供一套完整的绑定管理机制,确保了 ChainX 区块链项目能够正确处理跨链资产转移和推荐关系追踪.
这对于维护用户资产的安全和推荐系统的有效性至关重要.
*/
