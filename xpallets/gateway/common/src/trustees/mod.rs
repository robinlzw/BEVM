// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

pub mod bitcoin;

use frame_support::{
    dispatch::DispatchError,
    log::{error, warn},
};
use sp_std::{convert::TryFrom, marker::PhantomData, prelude::*};

use xp_assets_registrar::Chain;

use crate::{
    traits::{BytesLike, ChainProvider, TrusteeInfoUpdate, TrusteeSession},
    types::TrusteeSessionInfo,
    Config, Error, Pallet, TrusteeSessionInfoOf, TrusteeSigRecord, TrusteeTransitionStatus,
};

pub struct TrusteeSessionManager<T: Config, TrusteeAddress>(
    PhantomData<T>,
    PhantomData<TrusteeAddress>,
);

impl<T: Config, TrusteeAddress: BytesLike + ChainProvider>
    TrusteeSession<T::AccountId, T::BlockNumber, TrusteeAddress>
    for TrusteeSessionManager<T, TrusteeAddress>
{
    fn trustee_session(
        number: u32,
    ) -> Result<TrusteeSessionInfo<T::AccountId, T::BlockNumber, TrusteeAddress>, DispatchError>
    {
        let chain = TrusteeAddress::chain();
        let generic_info =
            Pallet::<T>::trustee_session_info_of(chain, number).ok_or_else(|| {
                error!(
                    target: "runtime::gateway::common",
                    "[trustee_session] Can not find session info, chain:{:?}, number:{}",
                    chain,
                    number
                );
                Error::<T>::InvalidTrusteeSession
            })?;
        let info = TrusteeSessionInfo::<T::AccountId, T::BlockNumber, TrusteeAddress>::try_from(
            generic_info,
        )
        .map_err(|_| Error::<T>::InvalidGenericData)?;
        Ok(info)
    }

    fn current_trustee_session(
    ) -> Result<TrusteeSessionInfo<T::AccountId, T::BlockNumber, TrusteeAddress>, DispatchError>
    {
        let chain = TrusteeAddress::chain();
        let number = Pallet::<T>::trustee_session_info_len(chain);
        Self::trustee_session(number)
    }

    fn current_proxy_account() -> Result<Vec<T::AccountId>, DispatchError> {
        Ok(Self::current_trustee_session()?
            .trustee_list
            .iter()
            .filter_map(|info| {
                match Pallet::<T>::trustee_intention_props_of(&info.0, TrusteeAddress::chain()) {
                    None => None,
                    Some(n) => n.0.proxy_account,
                }
            })
            .collect::<Vec<T::AccountId>>())
    }

    fn last_trustee_session(
    ) -> Result<TrusteeSessionInfo<T::AccountId, T::BlockNumber, TrusteeAddress>, DispatchError>
    {
        let chain = TrusteeAddress::chain();
        let number = match Pallet::<T>::trustee_session_info_len(chain).checked_sub(1) {
            Some(r) => r,
            None => u32::MAX,
        };
        Self::trustee_session(number).map_err(|err| {
            warn!(
                target: "runtime::gateway::common",
                "[last_trustee_session] Last trustee session not exist yet for chain:{:?}",
                chain
            );
            err
        })
    }

    fn trustee_transition_state() -> bool {
        Pallet::<T>::trustee_transition_status(TrusteeAddress::chain())
    }

    #[cfg(feature = "std")]
    fn genesis_trustee(chain: Chain, trustees: &[T::AccountId]) {
        Pallet::<T>::transition_trustee_session_impl(chain, trustees.to_vec())
            .expect("trustee session transition can not fail; qed");
    }
}

impl<T: Config> TrusteeInfoUpdate for Pallet<T> {
    fn update_transition_status(chain: Chain, status: bool, _: Option<u64>) {
        // The renewal of the trustee is completed, the current trustee information is replaced
        // and the number of multiple signings is archived. Currently only supports bitcoin
        if chain == Chain::Bitcoin && Self::trustee_transition_status(chain) && !status {
            let last_session_num = Self::trustee_session_info_len(chain).saturating_sub(1);
            TrusteeSessionInfoOf::<T>::mutate(chain, last_session_num, |info| match info {
                None => {
                    warn!(
                        target: "runtime::gateway::common",
                        "[last_trustee_session] Last trustee session not exist for chain:{:?}, session_num:{}",
                        chain, last_session_num
                    );
                }
                Some(trustee) => {
                    for i in 0..trustee.0.trustee_list.len() {
                        trustee.0.trustee_list[i].1 =
                            Self::trustee_sig_record(chain, &trustee.0.trustee_list[i].0);
                    }

                    let end_height = frame_system::Pallet::<T>::block_number();
                    trustee.0.end_height = Some(end_height);
                }
            });
            TrusteeSigRecord::<T>::remove_prefix(chain, None);
        }

        TrusteeTransitionStatus::<T>::insert(chain, status);
    }

    fn update_trustee_sig_record(chain: Chain, script: &[u8], withdraw_amount: u64) {
        let signed_trustees = Self::agg_pubkey_info(chain, script);
        signed_trustees.into_iter().for_each(|trustee| {
            let amount = if Some(trustee.clone()) == Self::trustee_admin(chain) {
                withdraw_amount
                    .saturating_mul(Self::trustee_admin_multiply(chain))
                    .checked_div(10)
                    .unwrap_or(withdraw_amount)
            } else {
                withdraw_amount
            };
            if TrusteeSigRecord::<T>::contains_key(chain, &trustee) {
                TrusteeSigRecord::<T>::mutate(chain, &trustee, |record| *record += amount);
            } else {
                TrusteeSigRecord::<T>::insert(chain, trustee, amount);
            }
        });
    }
}

/*
这段代码是 ChainX 区块链项目中负责管理受托人会话信息的模块.`TrusteeSessionManager` 是一个泛型结构体,
用于处理特定于比特币的受托人会话信息.它实现了 `TrusteeSession` trait,提供了一系列的函数来查询和管理受托人会话.

### 主要功能和结构体:

1. **TrusteeSessionManager**:
   - 一个泛型结构体,使用 `PhantomData` 来表示它依赖于特定的配置 `T`(即 ChainX 区块链的配置)和受托人地址类型 `TrusteeAddress`.
   - 实现了 `TrusteeSession` trait,该 trait 定义了获取受托人会话信息,当前会话,代理账户列表,最后会话以及转换状态的方法.

2. **TrusteeSessionInfo**:
   - 受托人会话信息的结构体,包含受托人列表,阈值,热钱包地址,冷钱包地址,多账户信息,开始和结束高度等字段.

3. **TrusteeSigRecord**:
   - 受托人签名记录的结构体,用于记录每个受托人签名的金额.

4. **TrusteeTransitionStatus**:
   - 一个记录受托人转换状态的结构体,用于标记受托人会话是否正在转换.

5. **TrusteeInfoUpdate**:
   - 一个 trait,定义了更新受托人信息的方法,如更新转换状态和签名记录.

### 主要方法:

- **trustee_session**:
  - 根据会话编号获取受托人会话信息.

- **current_trustee_session**:
  - 获取当前活跃的受托人会话信息.

- **current_proxy_account**:
  - 获取当前会话中所有受托人的代理账户列表.

- **last_trustee_session**:
  - 获取最后一个受托人会话信息.

- **trustee_transition_state**:
  - 获取受托人转换状态.

- **update_transition_status**:
  - 更新受托人转换状态,并在转换完成后更新最后会话的信息.

- **update_trustee_sig_record**:
  - 更新受托人的签名记录.

### 用途:

`TrusteeSessionManager` 用于 ChainX 区块链中管理比特币受托人的会话信息.
它允许区块链系统跟踪受托人的当前状态,历史记录以及他们在跨链交易中的角色.
通过这些功能,ChainX 能够确保比特币资产的安全转移和管理.
*/
