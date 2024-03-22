// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments, clippy::unnecessary_mut_passed)]

use sp_std::collections::btree_map::BTreeMap;

use codec::Codec;

pub use chainx_primitives::AssetId;
pub use xpallet_assets::{AssetType, TotalAssetInfo};

sp_api::decl_runtime_apis! {
    pub trait XAssetsApi<AccountId, Balance>
    where
        AccountId: Codec,
        Balance: Codec,
    {
        fn assets_for_account(who: AccountId) -> BTreeMap<AssetId, BTreeMap<AssetType, Balance>>;

        fn assets() -> BTreeMap<AssetId, TotalAssetInfo<Balance>>;
    }
}

/*
`XAssetsApi`的运行时API trait,用于与区块链上的资产相关联的功能.这个API提供了两种方法来查询账户持有的资产和获取所有资产的总信息.

以下是代码的详细解释:
1. **公共定义**:代码中公开了一些类型定义,如`AssetId`(资产ID),`AssetType`(资产类型),
和`TotalAssetInfo`(资产总信息),这些类型在`xpallet_assets`模块中定义.

2. **`XAssetsApi` trait**:定义了一个运行时API trait,它包含两个方法:
   - `assets_for_account`:接受一个`AccountId`(账户ID)作为参数,并返回一个双层映射,其中外层键是`AssetId`,
   内层键是`AssetType`,值是账户持有的该资产的数量(`Balance`).
   - `assets`:返回一个映射,其中键是`AssetId`,值是`TotalAssetInfo`,包含了关于每个资产的总量和其他相关信息.

这个API的设计允许区块链上的智能合约或其他运行时组件查询账户的资产持有情况和资产的总体信息,这对于实现资产管理,交易,市场等功能至关重要.
通过定义这样的API,ChainX项目能够提供一个标准化的方式来处理和访问资产数据,从而促进了区块链应用的开发和互操作性.

*/
