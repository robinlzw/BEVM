// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use super::*;

pub fn mint_into_encode(account: H160, amount: u128) -> Vec<u8> {
    // signature ++ account ++ amount
    let length = 16 + 20 + 32;
    let mut v = Vec::with_capacity(length);

    // bytes4(keccak256(bytes("mint_into(address,uint256)"))
    // 0xefe51695
    let sig_mint = [239u8, 229, 22, 149];

    // first 16-bytes
    v.extend_from_slice(&sig_mint[..]);
    v.extend_from_slice(&[0u8; 12][..]);

    // second 20-bytes
    v.extend_from_slice(&account[..]);

    // third 32-bytes
    v.extend_from_slice(&[0u8; 16][..]);
    v.extend_from_slice(&amount.to_be_bytes()[..]);

    v
}

pub fn burn_from_encode(account: H160, amount: u128) -> Vec<u8> {
    // signature ++ account ++ amount
    let length = 16 + 20 + 32;
    let mut v = Vec::with_capacity(length);

    // bytes4(keccak256(bytes("burn_from(address,uint256)"))
    // 0x0f536f84
    let sig_burn = [15u8, 83, 111, 132];

    // first 16-bytes
    v.extend_from_slice(&sig_burn[..]);
    v.extend_from_slice(&[0u8; 12][..]);

    // second 20-bytes
    v.extend_from_slice(&account[..]);

    // third 32-bytes
    v.extend_from_slice(&[0u8; 16][..]);
    v.extend_from_slice(&amount.to_be_bytes()[..]);

    v
}

/*
这段代码定义了两个函数,`mint_into_encode` 和 `burn_from_encode`,它们用于生成以太坊智能合约调用所需的编码数据.
这些函数主要用于构建调用合约函数的交易数据,特别是在涉及代币铸造(mint)和销毁(burn)的场景中.

### `mint_into_encode` 函数

`mint_into_encode` 函数用于编码"mint_into"函数调用的数据.这个函数通常在智能合约中定义,
用于允许合约所有者或其他授权地址铸造新的代币并将其分配给特定账户.
函数的参数是接收代币的账户地址(`account`)和要铸造的代币数量(`amount`).

函数首先定义了数据的长度,然后创建了一个空的 `Vec<u8>` 用于存储编码后的数据.接着,它按照以下步骤构建数据:

1. 添加函数选择器(`sig_mint`),这是一个通过 Keccak256 哈希计算得到的函数签名,用于标识要调用的合约函数.
2. 添加 12 个字节的零填充,这是因为函数选择器占用了 4 字节,但数据需要 20 字节的地址长度.
3. 添加 20 字节的账户地址.
4. 添加 16 个字节的零填充,然后是 32 个字节的代币数量(大端序).

### `burn_from_encode` 函数

`burn_from_encode` 函数用于编码"burn_from"函数调用的数据.这个函数通常在代币合约中定义,
用于允许合约所有者或其他授权地址销毁特定账户中的代币.参数同样是账户地址(`account`)和要销毁的代币数量(`amount`).

构建数据的步骤与 `mint_into_encode` 类似,只是使用了不同的函数选择器(`sig_burn`)来标识销毁函数.

### 总结

这两个函数生成的数据可以作为以太坊交易的数据字段(`data`),用于调用相应的智能合约函数.
这些编码的数据遵循了以太坊合约调用的标准格式,其中包含了函数选择器和必要的参数.
*/