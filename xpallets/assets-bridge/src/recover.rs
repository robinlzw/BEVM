// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use super::*;

/// Converts the given binary data into ASCII-encoded hex. It will be twice
/// the length.
pub fn to_ascii_hex(data: &[u8]) -> Vec<u8> {
    let mut r = Vec::with_capacity(data.len() * 2);
    let mut push_nibble = |n| r.push(if n < 10 { b'0' + n } else { b'a' - 10 + n });
    for &b in data.iter() {
        push_nibble(b / 16);
        push_nibble(b % 16);
    }
    r
}

/// Attempts to recover the Ethereum address from a message signature signed by
/// using the Ethereum RPC's `personal_sign` and `eth_sign`.
pub fn eth_recover(s: &EcdsaSignature, what: &[u8], extra: &[u8]) -> Option<H160> {
    let msg = keccak_256(&ethereum_signable_message(what, extra));
    let mut res = H160::default();
    res.0
        .copy_from_slice(&keccak_256(&secp256k1_ecdsa_recover(&s.0, &msg).ok()?[..])[12..]);
    Some(res)
}

/// Constructs the message that Ethereum RPC's `personal_sign` and `eth_sign`
/// would sign.
pub fn ethereum_signable_message(what: &[u8], extra: &[u8]) -> Vec<u8> {
    let prefix = b"evm:";
    let mut l = prefix.len() + what.len() + extra.len();
    let mut rev = Vec::new();
    while l > 0 {
        rev.push(b'0' + (l % 10) as u8);
        l /= 10;
    }
    let mut v = b"\x19Ethereum Signed Message:\n".to_vec();
    v.extend(rev.into_iter().rev());
    v.extend_from_slice(&prefix[..]);
    v.extend_from_slice(what);
    v.extend_from_slice(extra);
    v
}

/*
这段代码提供了几个与以太坊签名和地址恢复相关的函数,这些函数在处理以太坊兼容的签名和验证时非常有用.下面是对这些函数的详细解释:

### `to_ascii_hex` 函数

此函数将二进制数据转换为ASCII编码的十六进制字符串.输入是一个`[u8]`字节切片,输出是一个`Vec<u8>`,表示十六进制字符串的字节表示.
每个字节被转换成两个十六进制数字,因此输出向量的长度是输入数据长度的两倍.

### `eth_recover` 函数

此函数尝试从给定的ECDSA签名中恢复出以太坊地址.它接受一个`EcdsaSignature`签名对象和两个字节切片:`what`和`extra`.
`what`是被签名的消息内容,`extra`是附加到消息上的额外数据,通常用于确保签名的特定上下文(例如,以太坊RPC的`personal_sign`和`eth_sign`).
函数首先构造出签名消息的Keccak-256哈希,然后使用`secp256k1_ecdsa_recover`函数从哈希和签名中恢复出公钥.最后,它从公钥中提取以太坊地址.

### `ethereum_signable_message` 函数

此函数构造出以太坊RPC的`personal_sign`和`eth_sign`会签名的消息格式.这个消息格式以特定的前缀开始,通常是`"evm:"`,
后面跟着消息内容的长度(转换为ASCII编码的数字),然后是消息内容本身和任何额外的数据.这个构造过程确保了签名可以被以太坊节点正确验证.

这些函数在实现跨链桥接,处理以太坊签名的消息和验证签名者身份时非常有用.
例如,如果一个Substrate链上的智能合约需要验证一个来自以太坊用户的签名,它可以使用这些函数来恢复签名者的以太坊地址,并确认签名是否有效.
*/
