// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments, clippy::unnecessary_mut_passed)]

use sp_std::collections::btree_map::BTreeMap;

use codec::Codec;

pub use chainx_primitives::{AssetId, Decimals};
pub use xpallet_assets::Chain;
pub use xpallet_gateway_records::{Withdrawal, WithdrawalRecordId, WithdrawalState};

sp_api::decl_runtime_apis! {
    pub trait XGatewayRecordsApi<AccountId, Balance, BlockNumber>
    where
        AccountId: Codec,
        Balance: Codec,
        BlockNumber: Codec,
    {
        fn withdrawal_list() -> BTreeMap<WithdrawalRecordId, Withdrawal<AccountId, Balance, BlockNumber>>;

        fn withdrawal_list_by_chain(chain: Chain) -> BTreeMap<WithdrawalRecordId, Withdrawal<AccountId, Balance, BlockNumber>>;
    }
}

/*
这段代码定义了一个名为 `XGatewayRecordsApi` 的运行时 API 接口,它是 ChainX 项目中用于管理和查询跨链交易记录的接口.
这个 API 允许其他模块或外部客户端查询取款列表和特定链上的取款记录.

### API 接口声明

- `withdrawal_list`: 返回一个 BTreeMap,其中包含了所有的取款记录.这个映射的键是取款记录 ID (`WithdrawalRecordId`),值是取款记录本身 (`Withdrawal`).

- `withdrawal_list_by_chain`: 接受一个 `Chain` 类型的参数,返回该链上所有取款记录的 BTreeMap.这个函数允许查询特定区块链上的取款信息.

### 类型参数

- `AccountId`: 用于标识账户的类型,它必须实现了 `Codec` trait 以便可以进行序列化和反序列化.

- `Balance`: 用于表示账户余额的类型,同样需要实现 `Codec` trait.

- `BlockNumber`: 用于表示区块号的类型,也需要实现 `Codec` trait.

### 依赖项

- `sp_std`: 提供了集合类型 `BTreeMap` 的实现,用于存储有序的键值对.

- `codec`: 提供了 `Codec` trait,用于数据序列化和反序列化.

- `chainx_primitives` 和 `xpallet_assets`: 提供了 ChainX 项目中使用的原始类型和模块.

- `xpallet_gateway_records`: 提供了 `Withdrawal` 结构体和相关类型,用于表示取款记录.

### 总结

`XGatewayRecordsApi` API 为 ChainX 项目中的跨链交易记录提供了一个查询接口,使得其他模块或外部客户端能够获取取款信息.
这对于监控跨链资产流动,审计和用户界面显示等场景非常重要.通过实现这个 API,ChainX 项目能够提供一个透明和可查询的跨链交易记录系统.
*/
