// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use frame_support::dispatch::{DispatchError, DispatchResult};
use sp_std::fmt;

use chainx_primitives::{Decimals, Desc, Token};
use xp_assets_registrar::Chain;

use crate::verifier::*;
use crate::Config;

#[derive(PartialEq, Eq, Clone, Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct AssetInfo {
    #[cfg_attr(feature = "std", serde(with = "xp_rpc::serde_text"))]
    token: Token,
    #[cfg_attr(feature = "std", serde(with = "xp_rpc::serde_text"))]
    token_name: Token,
    chain: Chain,
    decimals: Decimals,
    #[cfg_attr(feature = "std", serde(with = "xp_rpc::serde_text"))]
    desc: Desc,
}

impl fmt::Debug for AssetInfo {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AssetInfo")
            .field("token", &String::from_utf8_lossy(&self.token))
            .field("token_name", &String::from_utf8_lossy(&self.token_name))
            .field("chain", &self.chain)
            .field("decimals", &self.decimals)
            .field("desc", &String::from_utf8_lossy(&self.desc))
            .finish()
    }
    #[cfg(not(feature = "std"))]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("<wasm:stripped>")
    }
}

impl AssetInfo {
    pub fn new<T: Config>(
        token: Token,
        token_name: Token,
        chain: Chain,
        decimals: Decimals,
        desc: Desc,
    ) -> Result<Self, DispatchError> {
        let asset = AssetInfo {
            token,
            token_name,
            chain,
            decimals,
            desc,
        };
        asset.is_valid::<T>()?;
        Ok(asset)
    }

    pub fn is_valid<T: Config>(&self) -> DispatchResult {
        is_valid_token::<T>(&self.token)?;
        is_valid_token_name::<T>(&self.token_name)?;
        is_valid_desc::<T>(&self.desc)
    }

    pub fn token(&self) -> &Token {
        &self.token
    }

    pub fn token_name(&self) -> &Token {
        &self.token_name
    }

    pub fn chain(&self) -> Chain {
        self.chain
    }

    pub fn desc(&self) -> &Desc {
        &self.desc
    }

    pub fn decimals(&self) -> Decimals {
        self.decimals
    }

    pub fn set_desc(&mut self, desc: Desc) {
        self.desc = desc
    }

    pub fn set_token(&mut self, token: Token) {
        self.token = token
    }

    pub fn set_token_name(&mut self, token_name: Token) {
        self.token_name = token_name
    }
}

/*
这段代码定义了一个名为 `AssetInfo` 的结构体,它用于存储和处理区块链资产的元信息.
`AssetInfo` 结构体是 `xp_assets_registrar` 模块的一部分,该模块可能是一个用于注册和管理区块链资产的系统.
以下是 `AssetInfo` 结构体及其相关功能的详细解释:

### `AssetInfo` 结构体

- `token`: 资产的符号,通常是一个简短的标识符.
- `token_name`: 资产的名称,通常是一个更具描述性的名称.
- `chain`: 资产所属的区块链链,使用 `Chain` 枚举表示.
- `decimals`: 资产的小数位数,表示资产可以被分割到的最小单位.
- `desc`: 资产的描述信息,通常包含有关资产的详细信息.

### 序列化和反序列化

- `Encode` 和 `Decode` trait: 允许 `AssetInfo` 实例可以被序列化为字节串,以便在网络上传输或存储.
- `TypeInfo`: 提供了关于结构体在运行时的类型信息,这对于动态类型系统和反射是有用的.

### 条件编译

- `#[cfg(feature = "std")]`: 这段代码块中的属性和依赖项仅在启用了 "std" 特性时才会包含.
这通常意味着在标准库可用的环境中(如大多数Rust项目),`serde` 可以用来进行序列化和反序列化.

### 调试输出

- `fmt::Debug` trait: 实现了标准库中的 `Debug` trait,允许 `AssetInfo` 实例可以通过 `{:?}` 格式化宏打印出调试信息.

### 构造函数和验证

- `new` 方法: 创建一个新的 `AssetInfo` 实例,并使用 `is_valid` 方法验证其有效性.
- `is_valid` 方法: 验证 `AssetInfo` 实例中的字段是否符合特定的规则,例如代币和名称的长度,字符集等.

### 访问器和修改器

- `token`, `token_name`, `chain`, `desc`, `decimals` 方法: 提供对 `AssetInfo` 字段的只读访问.
- `set_desc`, `set_token`, `set_token_name` 方法: 允许修改 `AssetInfo` 实例的 `desc`,`token` 和 `token_name` 字段.

整体而言,`AssetInfo` 结构体为区块链资产提供了一个灵活和可验证的数据模型,可以用于多种资产管理场景,如资产发行,交易,查询等.
通过使用序列化和反序列化特性,`AssetInfo` 可以轻松地在不同的系统和节点之间传输.
*/
