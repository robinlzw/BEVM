// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

//! Runtime API definition for transaction fee module.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_runtime::traits::{MaybeDisplay, MaybeFromStr};

pub use xpallet_transaction_fee::{FeeDetails, InclusionFee};

sp_api::decl_runtime_apis! {
    pub trait XTransactionFeeApi<Balance> where
        Balance: Codec + MaybeDisplay + MaybeFromStr,
    {
        fn query_fee_details(uxt: Block::Extrinsic, len: u32) -> FeeDetails<Balance>;
    }
}

/*
这段代码定义了一个名为`XTransactionFeeApi`的Runtime API(应用程序接口),用于查询交易费用模块的详细信息.
这个API是ChainX区块链项目的一部分,它允许其他模块或外部应用程序查询特定交易的交易费用.

以下是对代码中的关键组件的详细解释:

1. **Codec**:
   - `codec` crate提供了序列化和反序列化功能,允许数据结构在不同格式(如JSON,二进制等)之间转换.

2. **MaybeDisplay and MaybeFromStr**:
   - 这些trait来自`sp_runtime`库,分别用于支持自定义的显示(`Display`)和字符串到类型的解析(`FromStr`).
   这些trait允许在运行时对类型进行更灵活的处理,例如在控制台或用户界面上显示数值,或者从字符串构造类型.

3. **XTransactionFeeApi**:
   - 这个trait定义了一个名为`query_fee_details`的函数,它接受一个区块外部交易(`Block::Extrinsic`)和一个长度参数(`u32`),
   并返回一个`FeeDetails`对象.`FeeDetails`包含了交易费用的详细信息,如交易的纳入费用(`InclusionFee`).

4. **sp_api::decl_runtime_apis!**:
   - 这是一个宏,用于声明运行时API.它定义了API的名称,输入参数,输出结果和关联的类型.在这个例子中,`Balance`类型是API的一部分,
   它需要实现`Codec`,`MaybeDisplay`和`MaybeFromStr` trait.

5. **FeeDetails and InclusionFee**:
   - 这些类型来自`xpallet_transaction_fee`模块,`FeeDetails`包含了交易费用的详细信息,而`InclusionFee`表示交易被纳入区块所需支付的费用.

这个API的设计允许区块链的其他部分或外部服务查询交易费用,这对于前端应用程序来说非常有用,因为它们可以显示交易费用信息给用户,
或者在提交交易之前进行估算.此外,这也为区块链的治理和费用策略提供了透明度.
*/
