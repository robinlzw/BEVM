// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::{
    generic,
    traits::{BlakeTwo256, IdentifyAccount, Verify},
    MultiSignature, OpaqueExtrinsic,
};
use sp_std::prelude::Vec;

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them.
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Type used for expressing timestamp.
pub type Moment = u64;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// A timestamp: milliseconds since the unix epoch.
/// `u64` is enough to represent a duration of half a billion years, when the
/// time scale is milliseconds.
pub type Timestamp = u64;

/// Digest item type.
pub type DigestItem = generic::DigestItem;

/// Header type.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type.
pub type Block = generic::Block<Header, OpaqueExtrinsic>;

/// Block ID.
pub type BlockId = generic::BlockId<Block>;

// ============================================================================
// Runtime types
// ============================================================================

/// Signed version of Balance
pub type Amount = i128;

/// String for Runtime
pub type Text = Vec<u8>;

/// Asset ID
pub type AssetId = u32;

/// Asset decimal
pub type Decimals = u8;

/// Asset token symbol
pub type Token = Vec<u8>;

/// Asset token description
pub type Desc = Vec<u8>;

/// Referral ID of validator
pub type ReferralId = Vec<u8>;

pub type AddrStr = Vec<u8>;
pub type ChainAddress = Vec<u8>;


/*
这段代码定义了一系列用于区块链开发的数据类型,包括:

BlockNumber:区块号,类型为 u32.
Signature:交易签名,类型为 MultiSignature.
AccountId:链上账户的唯一标识,等同于交易签名的公钥,
类型为 <<Signature as Verify>::Signer as IdentifyAccount>::AccountId.

AccountIndex:账户索引,类型为 u32.
Balance:账户余额,类型为 u128.
Moment:时间戳,类型为 u64.
Index:交易索引,类型为 u32.
Hash:链上数据的哈希值,类型为 sp_core::H256.
Timestamp:时间戳,以毫秒为单位的UNIX时间,类型为 u64.
DigestItem:摘要项类型,类型为 generic::DigestItem.
Header:区块头类型,类型为 generic::Header<BlockNumber, BlakeTwo256>.
Block:区块类型,类型为 generic::Block<Header, OpaqueExtrinsic>.
BlockId:区块ID类型,类型为 generic::BlockId<Block>.
此外,还定义了用于运行时的一些数据类型:

Amount:已签名的余额类型,类型为 i128.
Text:运行时的字符串类型,类型为 Vec<u8>.
AssetId:资产ID类型,类型为 u32.
Decimals:资产的小数点位数,类型为 u8.
Token:资产的代币符号,类型为 Vec<u8>.
Desc:资产的代币描述,类型为 Vec<u8>.
ReferralId:验证者的推荐ID,类型为 Vec<u8>.
AddrStr:地址字符串类型,类型为 Vec<u8>.
ChainAddress:链地址类型,类型为 Vec<u8>.
*/