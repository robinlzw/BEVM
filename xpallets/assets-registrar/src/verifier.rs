// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use xp_protocol::{ASSET_DESC_MAX_LEN, ASSET_TOKEN_NAME_MAX_LEN, ASSET_TOKEN_SYMBOL_MAX_LEN};

use super::*;

/// Token can only use ASCII alphanumeric character or "-.|~".
pub fn is_valid_token<T: Config>(token: &[u8]) -> DispatchResult {
    if token.len() > ASSET_TOKEN_SYMBOL_MAX_LEN || token.is_empty() {
        return Err(Error::<T>::InvalidAssetTokenSymbolLength.into());
    }
    let is_valid = |c: &u8| -> bool { c.is_ascii_alphanumeric() || b"-.|~".contains(c) };
    for c in token {
        if !is_valid(c) {
            return Err(Error::<T>::InvalidAssetTokenSymbolChar.into());
        }
    }
    Ok(())
}

/// A valid token name should have a legal length and be visible ASCII chars only.
pub fn is_valid_token_name<T: Config>(token_name: &[u8]) -> DispatchResult {
    if token_name.len() > ASSET_TOKEN_NAME_MAX_LEN || token_name.is_empty() {
        return Err(Error::<T>::InvalidAssetTokenNameLength.into());
    }
    xp_runtime::xss_check(token_name)?;
    for c in token_name {
        if !is_ascii_visible(c) {
            return Err(Error::<T>::InvalidAscii.into());
        }
    }
    Ok(())
}

/// A valid desc should be visible ASCII chars only and not too long.
pub fn is_valid_desc<T: Config>(desc: &[u8]) -> DispatchResult {
    if desc.len() > ASSET_DESC_MAX_LEN {
        return Err(Error::<T>::InvalidAssetDescLength.into());
    }
    xp_runtime::xss_check(desc)?;
    for c in desc {
        if !is_ascii_visible(c) {
            return Err(Error::<T>::InvalidAscii.into());
        }
    }
    Ok(())
}

/// Visible ASCII char [0x20, 0x7E]
#[inline]
fn is_ascii_visible(c: &u8) -> bool {
    *c == b' ' || c.is_ascii_graphic()
}

/*
这段代码定义了一组用于验证区块链资产元信息的函数,确保资产的符号(token),名称(token_name)和描述(desc)符合特定的格式要求.
这些验证函数是 `xp_assets_registrar` 模块的一部分,该模块可能是用于在区块链上注册和管理资产的系统.

### 函数解释

1. **`is_valid_token`**: 验证资产符号的有效性.资产符号的长度必须在指定的最大长度 `ASSET_TOKEN_SYMBOL_MAX_LEN` 内,且不能为空.
同时,符号只能包含ASCII字母数字字符或特定的几个符号("-.|~").如果不符合这些条件,将返回相应的错误.

2. **`is_valid_token_name`**: 验证资产名称的有效性.资产名称的长度同样必须在指定的最大长度 `ASSET_TOKEN_NAME_MAX_LEN` 内,且不能为空.
名称中的每个字符都必须是可见的ASCII字符,且没有脚本攻击(XSS)的风险.如果不符合这些条件,将返回相应的错误.

3. **`is_valid_desc`**: 验证资产描述的有效性.资产描述的长度必须在指定的最大长度 `ASSET_DESC_MAX_LEN` 内.
描述中的每个字符也必须是可见的ASCII字符,且没有XSS风险.如果不符合这些条件,将返回相应的错误.

4. **`is_ascii_visible`**: 一个内联函数,用于检查给定的字节是否是可见的ASCII字符.
在ASCII表中,可见字符的范围是从空格(0x20)到删除线(0x7E).这个函数用于确保资产名称和描述中的字符不会包含控制字符或不可打印的字符.

### 安全性考虑

- **长度限制**: 对资产符号,名称和描述的长度进行限制,以防止潜在的缓冲区溢出或拒绝服务攻击.
- **字符集限制**: 限制资产符号和名称只能使用特定的字符集,以确保兼容性和避免注入攻击.
- **XSS检查**: 使用 `xp_runtime::xss_check` 函数检查资产名称和描述中是否有XSS风险,这是一种防范跨站脚本攻击的安全措施.

整体而言,这些验证函数确保了资产元信息的格式正确,安全,并且易于用户理解和使用.通过在资产注册过程中执行这些验证,可以提高区块链系统的安全性和可靠性.
*/
