// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use sp_runtime::RuntimeDebug;

use chainx_primitives::ReferralId;

use light_bitcoin::keys::Address;
pub use xp_gateway_common::OpReturnAccount;

/// (hot trustee address, cold trustee address)
pub type TrusteePair = (Address, Address);

/// The bitcoin transaction type.
#[doc(hidden)]
#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum BtcTxType {
    Withdrawal,
    Deposit,
    HotAndCold,
    TrusteeTransition,
    Irrelevance,
}

impl Default for BtcTxType {
    fn default() -> Self {
        BtcTxType::Irrelevance
    }
}

/// The transaction type with deposit info.
#[doc(hidden)]
#[derive(PartialEq, Eq, Clone, RuntimeDebug, TypeInfo)]
pub enum BtcTxMetaType<AccountId> {
    Withdrawal,
    Deposit(BtcDepositInfo<AccountId>),
    HotAndCold,
    TrusteeTransition,
    Irrelevance,
}

impl<AccountId> BtcTxMetaType<AccountId> {
    /// Convert the MetaTxType as BtcTxType.
    pub fn ref_into(&self) -> BtcTxType {
        match self {
            BtcTxMetaType::Withdrawal => BtcTxType::Withdrawal,
            BtcTxMetaType::Deposit(_) => BtcTxType::Deposit,
            BtcTxMetaType::HotAndCold => BtcTxType::HotAndCold,
            BtcTxMetaType::TrusteeTransition => BtcTxType::TrusteeTransition,
            BtcTxMetaType::Irrelevance => BtcTxType::Irrelevance,
        }
    }
}

/// The info of deposit transaction.
#[derive(PartialEq, Eq, Clone, RuntimeDebug, TypeInfo)]
pub struct BtcDepositInfo<AccountId> {
    /// The deposit value.
    pub deposit_value: u64,
    /// The parsed op_return data.
    pub op_return: Option<(OpReturnAccount<AccountId>, Option<ReferralId>)>,
    /// The input address of deposit transaction.
    pub input_addr: Option<Address>,
}

/*
这段代码定义了ChainX项目中与比特币交易类型相关的几个枚举类型和结构体,用于处理和区分不同类型的比特币交易.以下是代码中各个部分的详细解释:

1. **导入依赖**:代码开始部分导入了所需的模块,包括序列化和反序列化库(`codec`),类型信息库(`scale_info`),以及Substrate运行时环境的相关模块.

2. **TrusteePair类型**:这是一个元组类型,表示(热钱包受托人地址,冷钱包受托人地址)对.在ChainX项目中,热钱包和冷钱包通常用于管理比特币资产.

3. **BtcTxType枚举**:定义了比特币交易的几种类型,包括提款(`Withdrawal`),存款(`Deposit`),热钱包和冷钱包之间的转换(`HotAndCold`),
受托人转换(`TrusteeTransition`)以及不相关的交易(`Irrelevance`).这个枚举用于内部逻辑判断和交易处理.

4. **BtcTxMetaType枚举**:这是一个泛型枚举,用于表示包含账户信息的比特币交易类型.
它包含了提款,存款信息(`BtcDepositInfo`),热钱包和冷钱包之间的转换,受托人转换以及不相关的交易.

5. **BtcDepositInfo结构体**:表示存款交易的信息,包括存款金额,解析后的OP_RETURN数据以及存款交易的输入地址.
这个结构体用于存储和传递与存款交易相关的详细信息.

6. **转换方法**:`BtcTxMetaType`枚举实现了一个方法`ref_into`,它将`BtcTxMetaType`转换为`BtcTxType`.
这个转换方法用于将包含账户信息的交易类型转换为不包含账户信息的交易类型,以便在不需要账户信息的情况下使用.

整体而言,这段代码为ChainX区块链提供了一种机制,使其能够有效地处理和区分不同类型的比特币交易.
*/