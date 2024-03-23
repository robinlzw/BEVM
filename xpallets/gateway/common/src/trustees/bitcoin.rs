// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use codec::{Decode, Encode, Error as CodecError};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use sp_runtime::RuntimeDebug;
use sp_std::{convert::TryFrom, fmt, prelude::Vec};

use super::TrusteeSessionManager;
use crate::{
    traits::ChainProvider,
    types::{TrusteeIntentionProps, TrusteeSessionInfo},
};
use xp_assets_registrar::Chain;

pub type BtcAddress = Vec<u8>;
pub type BtcTrusteeSessionInfo<AccountId, BlockNumber> =
    TrusteeSessionInfo<AccountId, BlockNumber, BtcTrusteeAddrInfo>;
pub type BtcTrusteeIntentionProps<AccountId> = TrusteeIntentionProps<AccountId, BtcTrusteeType>;
pub type BtcTrusteeSessionManager<T> = TrusteeSessionManager<T, BtcTrusteeAddrInfo>;

#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct BtcTrusteeAddrInfo {
    #[cfg_attr(feature = "std", serde(with = "xp_rpc::serde_text"))]
    pub addr: BtcAddress,
    #[cfg_attr(feature = "std", serde(with = "xp_rpc::serde_hex"))]
    pub redeem_script: Vec<u8>,
}

impl fmt::Debug for BtcTrusteeAddrInfo {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redeem_script_in_hex = hex::encode(&self.redeem_script);
        if redeem_script_in_hex.len() > 16 {
            write!(
                f,
                "BtcTrusteeAddrInfo {{ addr: {}, redeem_script: 0x{}...{} }}",
                String::from_utf8_lossy(&self.addr),
                &redeem_script_in_hex[..8],
                &redeem_script_in_hex[redeem_script_in_hex.len() - 8..]
            )
        } else {
            write!(
                f,
                "BtcTrusteeAddrInfo {{ addr: {}, redeem_script: 0x{} }}",
                String::from_utf8_lossy(&self.addr),
                redeem_script_in_hex,
            )
        }
    }

    #[cfg(not(feature = "std"))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BtcTrusteeAddrInfo {{ addr: {:?}, redeem_script: {:?} }}",
            self.addr, self.redeem_script
        )
    }
}

impl From<BtcTrusteeAddrInfo> for Vec<u8> {
    fn from(value: BtcTrusteeAddrInfo) -> Self {
        value.encode()
    }
}

impl TryFrom<Vec<u8>> for BtcTrusteeAddrInfo {
    type Error = CodecError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Decode::decode(&mut &value[..])
    }
}

impl ChainProvider for BtcTrusteeAddrInfo {
    fn chain() -> Chain {
        Chain::Bitcoin
    }
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct BtcTrusteeType(pub light_bitcoin::keys::Public);

impl From<BtcTrusteeType> for Vec<u8> {
    fn from(value: BtcTrusteeType) -> Self {
        value.0.to_vec()
    }
}

impl TryFrom<Vec<u8>> for BtcTrusteeType {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        light_bitcoin::keys::Public::from_slice(&value)
            .map(BtcTrusteeType)
            .map_err(|_| ())
    }
}

impl ChainProvider for BtcTrusteeType {
    fn chain() -> Chain {
        Chain::Bitcoin
    }
}

#[test]
fn test_serde_btc_trustee_type() {
    let pubkey = BtcTrusteeType(light_bitcoin::keys::Public::Compressed(Default::default()));
    let ser = serde_json::to_string(&pubkey).unwrap();
    assert_eq!(
        ser,
        "\"0x000000000000000000000000000000000000000000000000000000000000000000\""
    );
    let de = serde_json::from_str::<BtcTrusteeType>(&ser).unwrap();
    assert_eq!(de, pubkey);
}

/*
这段代码是 ChainX 区块链项目中处理 比特币受托人(trustee) 信息的相关类型和实现.
它定义了与比特币受托人相关的结构体,trait 实现以及序列化/反序列化(serde)的支持.
以下是代码中定义的主要类型和它们的用途:

### BtcTrusteeAddrInfo
- 这是一个结构体,用于表示比特币受托人的地址信息,包括比特币地址(`addr`)和赎回脚本(`redeem_script`).
- 实现了 `Debug` trait,用于在标准库中提供调试信息.
- 实现了 `From` 和 `TryFrom` trait,允许在 `BtcTrusteeAddrInfo` 和 `Vec<u8>` 之间进行转换.
- 实现了 `ChainProvider` trait,用于指定受托人地址信息所属的链(比特币链).

### BtcTrusteeType
- 表示比特币受托人类型的结构体,包含一个比特币公钥.
- 同样实现了 `From` 和 `TryFrom` trait,允许在 `BtcTrusteeType` 和 `Vec<u8>` 之间进行转换.
- 也实现了 `ChainProvider` trait,指定受托人类型所属的链.

### 序列化/反序列化支持
- 通过 `#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]` 条件编译属性,
当启用标准库时,启用 serde 的序列化和反序列化特性.
- `serde_text` 和 `serde_hex` 属性用于控制文本和十六进制格式的序列化输出.

### 测试函数
- `test_serde_btc_trustee_type` 是一个测试函数,用于验证 `BtcTrusteeType` 的序列化和反序列化是否按预期工作.

这段代码的主要作用是为 ChainX 区块链项目中的比特币受托人信息提供数据结构和序列化支持,确保与比特币网络的兼容性,
并允许在 ChainX 区块链中安全有效地处理比特币受托人相关的操作.
通过这些类型和实现,ChainX 能够管理比特币资产的跨链转移和受托人会话信息.
*/
