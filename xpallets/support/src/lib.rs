// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::{format, string::String};

pub mod traits;

/// Try to convert a slice of bytes to a string.
#[inline]
pub fn try_str<S: AsRef<[u8]>>(src: S) -> String {
    if src
        .as_ref()
        .iter()
        .try_for_each(|byte| {
            if byte.is_ascii_graphic() {
                Ok(())
            } else {
                Err(())
            }
        })
        .is_ok()
    {
        str(src.as_ref())
    } else {
        hex(src.as_ref())
    }
}

/// Try to convert a slice of bytes to a address string.
#[inline]
pub fn try_addr<S: AsRef<[u8]>>(src: S) -> String {
    if src
        .as_ref()
        .iter()
        .try_for_each(|byte| {
            if byte.is_ascii_alphanumeric() {
                Ok(())
            } else {
                Err(())
            }
        })
        .is_ok()
    {
        str(src.as_ref())
    } else {
        hex(src.as_ref())
    }
}

/// Converts a slice of bytes to a string.
#[inline]
fn str(s: &[u8]) -> String {
    String::from_utf8_lossy(s).into_owned()
}

/// Converts a slice of bytes to a hex value, and then converts to a string with 0x prefix added.
#[inline]
fn hex(s: &[u8]) -> String {
    format!("0x{}", hex::encode(s))
}

/*
这段代码是ChainX区块链项目中的一个辅助模块,提供了几个用于处理字节切片到字符串转换的函数.
这些函数在处理区块链数据时非常有用,尤其是在处理地址和交易数据时.下面是对这些函数的详细解释:

1. **try_str**: 尝试将一个字节切片转换为一个字符串.它检查字节切片中的每个字节是否是ASCII图形字符
(例如字母,数字和一些特殊符号).如果所有字节都是ASCII图形字符,则使用`str`函数进行转换;否则,它会调用`hex`函数进行转换.

2. **try_addr**: 类似于`try_str`,但是它检查字节切片中的每个字节是否是ASCII字母数字字符.这通常用于处理以太坊风格的地址,
这些地址只包含字母和数字.如果所有字节都符合条件,则转换为字符串;否则,使用十六进制表示.

3. **str**: 这是一个内部辅助函数,用于将字节切片转换为一个UTF-8字符串.它使用`String::from_utf8_lossy`来创建一个损失性的UTF-8字符串,
这意味着如果字节不是有效的UTF-8序列,它们会被替换为``(U+FFFD).

4. **hex**: 这也是一个内部辅助函数,用于将字节切片转换为带有`0x`前缀的十六进制字符串.它使用`hex::encode`来获取十六进制编码的字符串.

这些函数的设计允许它们在不依赖标准库(`std`)的情况下运行,这在区块链的无依赖环境(如Substrate运行时)中非常有用.通过提供这些转换函数,
ChainX项目能够更容易地处理和显示区块链数据,同时保持代码的健壮性和可读性.
*/
