// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::RuntimeDebug;

/// The network type of ChainX.
#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum NetworkType {
    /// Main network type
    Mainnet,
    /// Test network type
    Testnet,
}

impl Default for NetworkType {
    fn default() -> Self {
        NetworkType::Testnet
    }
}

impl NetworkType {
    /// Return the ss58 address format identifier of the network type.
    pub fn ss58_addr_format_id(&self) -> Ss58AddressFormatId {
        match self {
            NetworkType::Mainnet => MAINNET_ADDRESS_FORMAT_ID,
            NetworkType::Testnet => TESTNET_ADDRESS_FORMAT_ID,
        }
    }
}

/// Ss58AddressFormat identifier
pub type Ss58AddressFormatId = u8;
/// ChainX main network ss58 address format identifier
pub const MAINNET_ADDRESS_FORMAT_ID: Ss58AddressFormatId = 44; // 44 is Ss58AddressFormat::ChainXAccount
/// ChainX test network ss58 address format identifier
pub const TESTNET_ADDRESS_FORMAT_ID: Ss58AddressFormatId = 42; // 42 is Ss58AddressFormat::SubstrateAccount

/*

这段代码定义了ChainX区块链项目的网络类型和与之相关的SS58地址格式标识符.以下是对代码中各个部分的详细解释:

1. **NetworkType枚举**:
   - 描述:定义了ChainX区块链的网络类型,包括主网(Mainnet)和测试网(Testnet).

2. **Ss58AddressFormatId类型**:
   - 描述:是一个`u8`类型的别名,用于表示SS58地址格式的标识符.

3. **SS58地址格式标识符常量**:
   - MAINNET_ADDRESS_FORMAT_ID:定义了ChainX主网的SS58地址格式标识符,值为44,对应于`Ss58AddressFormat::ChainXAccount`.
   - TESTNET_ADDRESS_FORMAT_ID:定义了ChainX测试网的SS58地址格式标识符,值为42,对应于`Ss58AddressFormat::SubstrateAccount`.

SS58地址格式是一种用于Substrate和Polkadot生态系统中的地址编码方式,它基于Base58编码,
并包含了一个校验和以及版本信息.这些标识符用于在处理地址时区分不同的网络,确保交易和资产发送到正确的目的地.
通过这些定义,ChainX能够支持在不同网络环境下的地址解析和交易处理.
*/
