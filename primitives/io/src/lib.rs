// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};

use sp_core::crypto::AccountId32;
use sp_runtime::RuntimeDebug;
use sp_runtime_interface::runtime_interface;

#[derive(Clone, Copy, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum Ss58CheckError {
    /// Bad alphabet.
    BadBase58,
    /// Bad length.
    BadLength,
    /// Unknown ss58 address format.
    UnknownSs58AddressFormat,
    /// Invalid checksum.
    InvalidChecksum,
    /// Invalid prefix
    InvalidPrefix,
    /// Invalid format.
    InvalidFormat,
    /// Invalid derivation path.
    InvalidPath,
    /// Mismatch version.
    MismatchVersion,
    /// Disallowed SS58 Address Format for this datatype.
    FormatNotAllowed,
}

#[runtime_interface]
pub trait Ss58Codec {
    fn from_ss58check(addr: &[u8]) -> Result<AccountId32, Ss58CheckError> {
        use sp_core::crypto::{PublicError, Ss58AddressFormat, Ss58Codec};
        let s = String::from_utf8_lossy(addr).into_owned();
        AccountId32::from_ss58check_with_version(&s)
            .map_err(|err| match err {
                PublicError::BadBase58 => Ss58CheckError::BadBase58,
                PublicError::BadLength => Ss58CheckError::BadLength,
                PublicError::UnknownSs58AddressFormat(_) => {
                    Ss58CheckError::UnknownSs58AddressFormat
                }
                PublicError::InvalidChecksum => Ss58CheckError::InvalidChecksum,
                PublicError::InvalidPrefix => Ss58CheckError::InvalidPrefix,
                PublicError::InvalidFormat => Ss58CheckError::InvalidFormat,
                PublicError::InvalidPath => Ss58CheckError::InvalidPath,
                PublicError::FormatNotAllowed => Ss58CheckError::FormatNotAllowed,
            })
            .and_then(|(account, ver)| match ver {
                ver if ver == Ss58AddressFormat::from(44u16) => Ok(account),
                _ => Err(Ss58CheckError::MismatchVersion),
            })
    }

    #[version(2)]
    fn from_ss58check(addr: &[u8]) -> Result<AccountId32, Ss58CheckError> {
        use sp_core::crypto::{PublicError, Ss58Codec};
        let s = String::from_utf8_lossy(addr).into_owned();
        AccountId32::from_ss58check_with_version(&s)
            .map(|(account, _)| {
                // https://github.com/paritytech/substrate/blob/polkadot-v0.9.18/primitives/core/src/crypto.rs#L310
                // Support all ss58 versions.
                account
            })
            .map_err(|err| match err {
                PublicError::BadBase58 => Ss58CheckError::BadBase58,
                PublicError::BadLength => Ss58CheckError::BadLength,
                PublicError::UnknownSs58AddressFormat(_) => {
                    Ss58CheckError::UnknownSs58AddressFormat
                }
                PublicError::InvalidChecksum => Ss58CheckError::InvalidChecksum,
                PublicError::InvalidPrefix => Ss58CheckError::InvalidPrefix,
                PublicError::InvalidFormat => Ss58CheckError::InvalidFormat,
                PublicError::InvalidPath => Ss58CheckError::InvalidPath,
                PublicError::FormatNotAllowed => Ss58CheckError::FormatNotAllowed,
            })
    }
}

#[test]
fn ss58_check() {
    use sp_core::crypto::{set_default_ss58_version, Ss58AddressFormat};
    let addr42 = b"5CE864FPj1Z48qrvdCAQ48iTfkcBFMoUWt2UAnR4Np22kZFM";
    let addr44 = b"5PoSc3LCVbJWSxfrSFvSowFJxitmMj4Wtm8jQ9hfJXD1K5vF";
    let pubkey =
        hex::decode("072ec6e199a69a1a38f0299afc083b2b6c85899bdad56d250b2ec39a9788b7a2").unwrap();

    set_default_ss58_version(Ss58AddressFormat::from(44u16));
    let account = ss_58_codec::from_ss58check(addr44).unwrap();
    assert_eq!(AsRef::<[u8]>::as_ref(&account), pubkey.as_slice());
    assert!(ss_58_codec::from_ss58check(addr42).is_ok());

    set_default_ss58_version(Ss58AddressFormat::from(42u16));
    let account = ss_58_codec::from_ss58check(addr42).unwrap();
    assert_eq!(AsRef::<[u8]>::as_ref(&account), pubkey.as_slice());
    assert!(ss_58_codec::from_ss58check(addr44).is_ok());
}

/*
这段代码是Substrate框架中用于处理SS58地址编码和解码的一部分.SS58是一种用于Substrate和Polkadot生态系统中的地址格式,
它基于Base58编码,并且包含了一个校验和以及版本信息.以下是对代码中各个部分的详细解释:

1. **Ss58CheckError枚举**:
   - 描述:定义了在SS58地址检查过程中可能遇到的错误类型.
   - 字段:包括各种错误情况,如错误的Base58编码,长度不符,未知的SS58地址格式,无效的校验和,无效的前缀,无效的格式,
   无效的派生路径,版本不匹配,不允许的SS58地址格式.

2. **Ss58Codec trait**:
   - 描述:定义了SS58编解码器的接口,用于从SS58格式的地址解码出`AccountId32`.
   - 方法:
     - `from_ss58check`:尝试从给定的字节切片中解码出一个`AccountId32`,如果成功则返回账户ID,否则返回一个错误.
     - `from_ss58check`(带有版本注释):这是一个特定版本的`from_ss58check`方法,它允许在不同的版本下进行解码,
     但在这个例子中,它被注释掉了,可能是因为它与第一个方法冲突.

3. **测试函数ss58_check**:
   - 描述:提供了一个测试用例,用于验证SS58编解码器的正确性.
   - 行为:设置了默认的SS58地址版本,然后使用两个不同的SS58地址进行编解码测试,确保返回的账户ID与预期的公钥切片相匹配.

这段代码的主要作用是确保SS58地址能够在Substrate和Polkadot生态系统中被正确地处理.通过实现`Ss58Codec` trait,
开发者可以创建自定义的编解码器,以支持不同的SS58地址版本和格式.
测试函数确保了编解码器在处理不同版本的SS58地址时的鲁棒性.
*/
