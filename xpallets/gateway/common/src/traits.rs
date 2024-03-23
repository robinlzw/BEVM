// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use frame_support::dispatch::DispatchError;
use sp_std::{convert::TryFrom, prelude::Vec};

use chainx_primitives::{AssetId, ReferralId};
use xpallet_assets::Chain;

use crate::types::{ScriptInfo, TrusteeInfoConfig, TrusteeIntentionProps, TrusteeSessionInfo};
use xp_gateway_bitcoin::{BtcDepositInfo, OpReturnAccount};
use xp_gateway_common::DstChain;

pub trait BytesLike: Into<Vec<u8>> + TryFrom<Vec<u8>> {}
impl<T: Into<Vec<u8>> + TryFrom<Vec<u8>>> BytesLike for T {}

pub trait ChainProvider {
    fn chain() -> Chain;
}

pub trait ProposalProvider {
    type WithdrawalProposal;

    fn get_withdrawal_proposal() -> Option<Self::WithdrawalProposal>;
}

impl ProposalProvider for () {
    type WithdrawalProposal = ();

    fn get_withdrawal_proposal() -> Option<Self::WithdrawalProposal> {
        None
    }
}

pub trait TotalSupply<Balance> {
    fn total_supply() -> Balance;
}

pub trait TrusteeForChain<
    AccountId,
    BlockNumber,
    TrusteeEntity: BytesLike,
    TrusteeAddress: BytesLike,
>
{
    fn check_trustee_entity(raw_addr: &[u8]) -> Result<TrusteeEntity, DispatchError>;

    fn generate_trustee_session_info(
        props: Vec<(AccountId, TrusteeIntentionProps<AccountId, TrusteeEntity>)>,
        config: TrusteeInfoConfig,
    ) -> Result<
        (
            TrusteeSessionInfo<AccountId, BlockNumber, TrusteeAddress>,
            ScriptInfo<AccountId>,
        ),
        DispatchError,
    >;
}

pub trait TrusteeSession<AccountId, BlockNumber, TrusteeAddress: BytesLike> {
    fn trustee_session(
        number: u32,
    ) -> Result<TrusteeSessionInfo<AccountId, BlockNumber, TrusteeAddress>, DispatchError>;

    fn current_trustee_session(
    ) -> Result<TrusteeSessionInfo<AccountId, BlockNumber, TrusteeAddress>, DispatchError>;

    fn current_proxy_account() -> Result<Vec<AccountId>, DispatchError>;

    fn last_trustee_session(
    ) -> Result<TrusteeSessionInfo<AccountId, BlockNumber, TrusteeAddress>, DispatchError>;

    fn trustee_transition_state() -> bool;

    #[cfg(feature = "std")]
    fn genesis_trustee(chain: Chain, init: &[AccountId]);
}

impl<AccountId, BlockNumber, TrusteeAddress: BytesLike>
    TrusteeSession<AccountId, BlockNumber, TrusteeAddress> for ()
{
    fn trustee_session(
        _: u32,
    ) -> Result<TrusteeSessionInfo<AccountId, BlockNumber, TrusteeAddress>, DispatchError> {
        Err("NoTrustee".into())
    }

    fn current_trustee_session(
    ) -> Result<TrusteeSessionInfo<AccountId, BlockNumber, TrusteeAddress>, DispatchError> {
        Err("NoTrustee".into())
    }

    fn current_proxy_account() -> Result<Vec<AccountId>, DispatchError> {
        Err("NoTrustee".into())
    }

    fn last_trustee_session(
    ) -> Result<TrusteeSessionInfo<AccountId, BlockNumber, TrusteeAddress>, DispatchError> {
        Err("NoTrustee".into())
    }

    fn trustee_transition_state() -> bool {
        false
    }

    #[cfg(feature = "std")]
    fn genesis_trustee(_: Chain, _: &[AccountId]) {}
}

pub trait TrusteeInfoUpdate {
    /// Update the trustee trasition status when the renewal of the trustee is completed
    fn update_transition_status(chain: Chain, status: bool, trans_amount: Option<u64>);
    /// Each withdrawal is completed to record the weight of the signer
    fn update_trustee_sig_record(chain: Chain, script: &[u8], withdraw_amout: u64);
}

impl TrusteeInfoUpdate for () {
    fn update_transition_status(_: Chain, _: bool, _: Option<u64>) {}

    fn update_trustee_sig_record(_: Chain, _: &[u8], _: u64) {}
}

pub trait ReferralBinding<AccountId> {
    fn update_binding(asset_id: &AssetId, who: &AccountId, referral_name: Option<ReferralId>);
    fn referral(asset_id: &AssetId, who: &AccountId) -> Option<AccountId>;
}

impl<AccountId> ReferralBinding<AccountId> for () {
    fn update_binding(_: &AssetId, _: &AccountId, _: Option<ReferralId>) {}
    fn referral(_: &AssetId, _: &AccountId) -> Option<AccountId> {
        None
    }
}

pub trait AddressBinding<AccountId, Address: Into<Vec<u8>>> {
    fn update_binding(chain: Chain, address: Address, who: OpReturnAccount<AccountId>);
    fn check_allowed_binding(info: BtcDepositInfo<AccountId>) -> BtcDepositInfo<AccountId>;
    fn dst_chain_proxy_address(dst_chain: DstChain) -> Option<AccountId>;
    fn address(chain: Chain, address: Address) -> Option<OpReturnAccount<AccountId>>;
}

impl<AccountId, Address: Into<Vec<u8>>> AddressBinding<AccountId, Address> for () {
    fn update_binding(_: Chain, _: Address, _: OpReturnAccount<AccountId>) {}
    fn check_allowed_binding(info: BtcDepositInfo<AccountId>) -> BtcDepositInfo<AccountId> {
        info
    }
    fn dst_chain_proxy_address(_: DstChain) -> Option<AccountId> {
        None
    }
    fn address(_: Chain, _: Address) -> Option<OpReturnAccount<AccountId>> {
        None
    }
}

/*
这段代码定义了一系列的 trait 和它们的默认实现,这些 trait 被用于 ChainX 区块链项目中,
主要用于处理受托人(trustee)管理,提案提供,总供应量,推荐人绑定和地址绑定等功能.

### 主要 Trait 及其功能:

1. **BytesLike**:
   - 一个 trait,用于类型可以转换成 `Vec<u8>` 并从 `Vec<u8>` 尝试重建的类型.

2. **ChainProvider**:
   - 一个 trait,用于获取特定类型关联的链(Chain).

3. **ProposalProvider**:
   - 一个 trait,用于获取提款提案(withdrawal proposal)的接口.

4. **TotalSupply**:
   - 一个 trait,用于获取特定资产的总供应量.

5. **TrusteeForChain**:
   - 一个 trait,定义了与受托人实体相关的操作,如检查受托人实体和生成受托人会话信息.

6. **TrusteeSession**:
   - 一个 trait,定义了获取受托人会话信息的接口,包括当前会话,代理账户,最后会话和受托人过渡状态.

7. **TrusteeInfoUpdate**:
   - 一个 trait,用于更新受托人过渡状态和签名记录.

8. **ReferralBinding**:
   - 一个 trait,用于更新和获取推荐人绑定.

9. **AddressBinding**:
   - 一个 trait,用于更新和检查地址绑定,以及获取目标链的代理地址和解析地址.

### 默认实现:

每个 trait 都有一个空实现(使用单元结构体 `()` 作为默认实现),这些实现返回错误或 `None`,
表示没有受托人或不进行任何操作.这些默认实现可以被具体的 pallet 或模块覆盖,以提供实际的逻辑.

### 总结:

这段代码为 ChainX 区块链项目提供了一套标准的接口和默认行为,使得受托人管理和地址绑定等功能
可以在不同的模块中以统一的方式实现和扩展.
*/
