// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

//! Runtime API definition required by ChainX RPC extensions.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments, clippy::unnecessary_mut_passed)]

use sp_std::{collections::btree_map::BTreeMap, prelude::*};

use sp_runtime::DispatchError;

pub use chainx_primitives::{AddrStr, AssetId, ChainAddress};
pub use xp_assets_registrar::Chain;
pub use xp_runtime::Memo;

pub use xpallet_assets::WithdrawalLimit;
pub use xpallet_gateway_common::{
    trustees,
    types::{GenericTrusteeIntentionProps, GenericTrusteeSessionInfo, ScriptInfo},
};
pub use xpallet_gateway_records::{Withdrawal, WithdrawalRecordId, WithdrawalState};
sp_api::decl_runtime_apis! {
    /// The API to query account nonce (aka transaction index).
    pub trait XGatewayCommonApi<AccountId, Balance, BlockNumber>
    where
        AccountId: codec::Codec,
        Balance: codec::Codec,
        BlockNumber: codec::Codec,
    {
        fn bound_addrs(who: AccountId) -> BTreeMap<Chain, Vec<ChainAddress>>;

        fn withdrawal_limit(asset_id: AssetId) -> Result<WithdrawalLimit<Balance>, DispatchError>;

        #[allow(clippy::type_complexity)]
        fn withdrawal_list_with_fee_info(asset_id: AssetId) -> Result<
        BTreeMap<
            WithdrawalRecordId,
            (
                Withdrawal<AccountId, Balance, BlockNumber>,
                WithdrawalLimit<Balance>,
            ),
        >,
        DispatchError,
    >;

        fn verify_withdrawal(asset_id: AssetId, value: Balance, addr: AddrStr, memo: Memo) -> Result<(), DispatchError>;

        /// Get all trustee multisig.
        fn trustee_multisigs() -> BTreeMap<Chain, AccountId>;

        fn trustee_properties(chain: Chain, who: AccountId) -> Option<GenericTrusteeIntentionProps<AccountId>>;

        fn trustee_session_info(chain: Chain, session_number: i32) -> Option<GenericTrusteeSessionInfo<AccountId, BlockNumber>>;

        fn generate_trustee_session_info(chain: Chain, Vec<AccountId>) -> Result<(GenericTrusteeSessionInfo<AccountId, BlockNumber>, ScriptInfo<AccountId>), DispatchError>;
    }
}

/*
这段代码定义了 ChainX 区块链的 RPC 扩展所需的运行时 API(Application Programming Interface).
RPC 扩展允许外部客户端与区块链节点进行交互,执行查询和操作.这个 API 专注于 `xpallet_gateway_common` 模块,
该模块提供了与网关和资产跨链转移相关的功能.

以下是定义的 API 函数及其用途:

1. **bound_addrs**:
   - 功能:获取指定账户在不同链上的绑定地址.
   - 参数:`AccountId`(账户ID).
   - 返回值:一个 `BTreeMap`,键是链的类型(`Chain`),值是该账户在该链上的地址列表(`Vec<ChainAddress>`).

2. **withdrawal_limit**:
   - 功能:获取指定资产的提款限额.
   - 参数:`AssetId`(资产ID).
   - 返回值:`WithdrawalLimit<Balance>` 结构体,包含提款限额信息.

3. **withdrawal_list_with_fee_info**:
   - 功能:获取带有提款费用信息的提款列表.
   - 参数:`AssetId`(资产ID).
   - 返回值:一个 `BTreeMap`,键是提款记录ID(`WithdrawalRecordId`),值是一个元组,
   包含提款信息(`Withdrawal<AccountId, Balance, BlockNumber>`)和提款限额(`WithdrawalLimit<Balance>`).

4. **verify_withdrawal**:
   - 功能:验证提款请求的有效性,包括资产数量,接收地址和备忘录.
   - 参数:`AssetId`(资产ID),`Balance`(提款金额),`AddrStr`(接收地址),`Memo`(备忘录).
   - 返回值:无(`Result<(), DispatchError>`),成功时返回空元组,失败时返回错误.

5. **trustee_multisigs**:
   - 功能:获取所有受托人的多重签名地址.
   - 返回值:一个 `BTreeMap`,键是链的类型(`Chain`),值是受托人账户ID列表(`AccountId`).

6. **trustee_properties**:
   - 功能:获取指定账户在特定链上的受托人属性.
   - 参数:`Chain`(链类型),`AccountId`(账户ID).
   - 返回值:一个 `Option<GenericTrusteeIntentionProps<AccountId>>`,如果存在则包含受托人属性.

7. **trustee_session_info**:
   - 功能:获取指定链和会话编号的受托人会话信息.
   - 参数:`Chain`(链类型),`session_number`(会话编号).
   - 返回值:一个 `Option<GenericTrusteeSessionInfo<AccountId, BlockNumber>>`,如果存在则包含受托人会话信息.

8. **generate_trustee_session_info**:
   - 功能:生成新的受托人会话信息.
   - 参数:`Chain`(链类型),`Vec<AccountId>`(账户ID列表).
   - 返回值:一个元组,包含受托人会话信息(`GenericTrusteeSessionInfo<AccountId, BlockNumber>`)
   和脚本信息(`ScriptInfo<AccountId>`).

这些 API 函数为 ChainX 区块链的用户提供了一系列工具,以查询和管理他们的资产跨链转移操作.
通过这些函数,用户可以检查自己的地址绑定情况,提款限额,提款列表,以及验证和生成受托人会话信息.
*/
