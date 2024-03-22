// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;

sp_api::decl_runtime_apis! {
    pub trait BtcLedgerApi<AccountId, Balance>
    where
        AccountId: Codec,
        Balance: Codec,
    {
        fn get_balance(who: AccountId) -> Balance;
        fn get_total() -> Balance;
    }
}

/*
这段代码是使用Substrate框架的API声明宏(`decl_runtime_apis!`)定义的一个运行时API接口,名为`BtcLedgerApi`.
这个API提供了与比特币账本相关的查询功能,允许用户或智能合约获取比特币账户的余额信息以及总供应量.以下是代码的详细解释:

### 运行时API声明

- `pub trait BtcLedgerApi<AccountId, Balance>`: 声明了一个名为`BtcLedgerApi`的公共trait(接口),
它包含两个泛型参数:`AccountId`和`Balance`.这两个参数分别用于表示账户ID和余额的类型.

- `where AccountId: Codec, Balance: Codec`: 这是一个约束条件,指明了`AccountId`和`Balance`类型必须是`Codec` trait的实现.
`Codec` trait是Substrate中用于序列化和反序列化数据的trait,这允许API返回的数据类型在网络中传输.

### API方法

- `fn get_balance(who: AccountId) -> Balance;`: 定义了一个名为`get_balance`的方法,该方法接受一个`AccountId`作为参数,
并返回与该账户ID关联的`Balance`.这个方法可以用来查询特定账户的比特币余额.

- `fn get_total() -> Balance;`: 定义了一个名为`get_total`的方法,该方法不接受参数,并返回比特币账本的总供应量(总余额).

### 总结

这个`BtcLedgerApi` API为Substrate区块链上的其他模块或智能合约提供了与比特币账本交互的能力.通过实现这个API,
可以创建用于管理比特币资产的智能合约,或者在区块链应用中集成比特币账户的余额查询功能.
*/
