// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

//! ChainX Runtime Modules shared primitive types.

#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{DispatchError, DispatchResult};
use sp_std::prelude::Vec;

const MAXIMUM_MEMO_LEN: u8 = 128;

/// Returns Ok(_) if the input slice passes the xss check.
///
/// Although xss is imperceptible on-chain, we want to make it
/// look safer off-chain.
#[inline]
pub fn xss_check(input: &[u8]) -> DispatchResult {
    if input.contains(&b'<') || input.contains(&b'>') {
        return Err(DispatchError::Other(
            "'<' and '>' are not allowed, which could be abused off-chain.",
        ));
    }
    Ok(())
}

/// Type for leaving a note when sending a transaction.
#[derive(PartialEq, Eq, Clone, sp_core::RuntimeDebug, Encode, Decode, Default, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Memo(Vec<u8>);

impl From<Vec<u8>> for Memo {
    fn from(raw: Vec<u8>) -> Self {
        Self(raw)
    }
}

impl From<&[u8]> for Memo {
    fn from(raw: &[u8]) -> Self {
        Self(raw.to_vec())
    }
}

impl AsRef<[u8]> for Memo {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

#[cfg(feature = "std")]
impl std::fmt::Display for Memo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.0))
    }
}

#[cfg(not(feature = "std"))]
impl core::fmt::Display for Memo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Memo {
    /// Returns true if the inner byte length is in the range of [0, 128] and passes the xss check.
    pub fn check_validity(&self) -> DispatchResult {
        if self.0.len() > MAXIMUM_MEMO_LEN as usize {
            Err(DispatchError::Other(
                "transaction memo too long, valid byte length range: [0, 128]",
            ))
        } else {
            xss_check(&self.0)
        }
    }
}

/// Used for evm rpc
pub enum Never {}
impl<T> fp_rpc::ConvertTransaction<T> for Never {
    fn convert_transaction(&self, _transaction: pallet_ethereum::Transaction) -> T {
        // The Never type is not instantiable, but this method requires the type to be
        // instantiated to be called (`&self` parameter), so if the code compiles we have the
        // guarantee that this function will never be called.
        unreachable!()
    }
}

/*
这段代码定义了一个用于ChainX Runtime Modules的共享原始类型模块.
其中包含了一个用于执行跨链交易时留下的备注信息类型Memo以及对其进行操作的一些函数.

1.xss_check 函数用于检查输入的字节切片是否通过了XSS(跨站脚本攻击)检查.
如果输入包含字符<或>,则认为可能被用于离链的恶意操作,返回错误.否则返回成功结果.

2.Memo 类型是一个包含字节向量的结构体,支持从Vec<u8>和&[u8]转换,且提供了展示(显示)格式的实现.它还定义了一个方法 check_validity,
用于检查备注内容的合法性.该方法首先检查备注的字节长度是否超过了最大长度128字节,如果超过则返回错误;否则调用 xss_check 函数进行XSS检查.

3.Never 枚举类型用于EVM RPC(远程过程调用)转换,其实现了 fp_rpc::ConvertTransaction 特征,但该方法使用了 unreachable!() 链接宏,
意味着该方法永远不应该被调用,因为 Never 类型是不可实例化的.

这些定义为ChainX Runtime Modules提供了一个安全,规范的交易备注机制.

*/