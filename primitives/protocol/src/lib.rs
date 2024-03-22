// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

//! Some protocol details in the ChainX.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

mod asset;
mod network;

pub use self::asset::*;
pub use self::network::*;

/// The maximum length of asset token symbol
pub const ASSET_TOKEN_SYMBOL_MAX_LEN: usize = 24;

/// The maximum length of asset token name
pub const ASSET_TOKEN_NAME_MAX_LEN: usize = 48;

/// The maximum length of asset description
pub const ASSET_DESC_MAX_LEN: usize = 128;

/// The maximum length of memo
pub const MEMO_MAX_LEN: usize = 80;

/*
这段代码是ChainX区块链项目中定义的一些协议细节.它包括了资产模块(`asset`)和网络模块(`network`)的声明,
以及一些关于资产和交易的元数据长度限制.以下是对代码中各个部分的详细解释:

1. **模块声明**:
   - `mod asset;`:声明了一个名为`asset`的模块,该模块可能包含了与ChainX区块链中资产相关的类型定义,函数和其他逻辑.
   - `mod network;`:声明了一个名为`network`的模块,该模块可能包含了与ChainX区块链网络通信和操作相关的逻辑.

2. **公开使用声明**:
   - `pub use self::asset::*;`:公开了`asset`模块中的所有公开项(如类型,函数等),使得它们可以在当前文件的作用域中直接访问.
   - `pub use self::network::*;`:公开了`network`模块中的所有公开项,同样使得它们可以在当前文件的作用域中直接访问.

3. **常量定义**:
   - `ASSET_TOKEN_SYMBOL_MAX_LEN`:定义了资产代币符号的最大长度为24个字符.这可能是为了确保符号在交易和显示时的可读性和一致性.
   - `ASSET_TOKEN_NAME_MAX_LEN`:定义了资产代币名称的最大长度为48个字符.这有助于限制名称的长度,以便于在UI中显示和处理.
   - `ASSET_DESC_MAX_LEN`:定义了资产描述的最大长度为128个字符.这可能是为了限制资产描述的长度,以便于在区块链上存储和检索.
   - `MEMO_MAX_LEN`:定义了交易备忘录(memo)的最大长度为80个字符.备忘录通常用于添加交易的额外信息或上下文,这个长度限制有助于保持区块链的状态大小在合理范围内.

这些定义为ChainX区块链上的资产创建和交易提供了一套标准化的规则,确保了系统的一致性和效率.通过限制元数据的长度,
ChainX能够更有效地管理区块链上的信息,并为用户提供清晰,一致的交互体验.

*/
