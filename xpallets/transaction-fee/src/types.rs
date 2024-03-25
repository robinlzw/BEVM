// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use sp_runtime::{traits::AtLeast32BitUnsigned, RuntimeDebug};

use pallet_transaction_payment::InclusionFee;

/// The `final_fee` is composed of:
///   - (Optional) `inclusion_fee`: Only the `Pays::Yes` transaction can have the inclusion fee.
///   - (Optional) `tip`: If included in the transaction, the tip will be added on top. Only
///     signed transactions can have a tip.
///
/// ```ignore
/// final_fee = inclusion_fee + tip;
/// ```
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct FeeDetails<Balance> {
    /// The minimum fee for a transaction to be included in a block.
    pub inclusion_fee: Option<InclusionFee<Balance>>,
    // Do not serialize and deserialize `tip` as we actually can not pass any tip to the RPC.
    #[cfg_attr(feature = "std", serde(skip))]
    pub tip: Balance,
    /// Additional fee for some ChainX specific calls.
    pub extra_fee: Balance,
    pub final_fee: Balance,
}

impl<Balance: AtLeast32BitUnsigned + Copy> FeeDetails<Balance> {
    pub fn new(
        base: pallet_transaction_payment::FeeDetails<Balance>,
        maybe_extra_fee: Option<Balance>,
    ) -> Self {
        match maybe_extra_fee {
            Some(extra_fee) => Self {
                extra_fee,
                final_fee: base.final_fee() + extra_fee,
                ..base.into()
            },
            None => base.into(),
        }
    }
}

impl<Balance: AtLeast32BitUnsigned + Copy> From<pallet_transaction_payment::FeeDetails<Balance>>
    for FeeDetails<Balance>
{
    fn from(details: pallet_transaction_payment::FeeDetails<Balance>) -> Self {
        let final_fee = details.final_fee();
        Self {
            inclusion_fee: details.inclusion_fee,
            tip: details.tip,
            extra_fee: 0u32.into(),
            final_fee,
        }
    }
}

/*
这段代码定义了`FeeDetails`结构体,它用于表示ChainX区块链项目中交易费用的详细信息.`FeeDetails`结构体
包含了交易的纳入费用(`inclusion_fee`),小费(`tip`),额外费用(`extra_fee`)以及最终费用(`final_fee`).
以下是对`FeeDetails`结构体及其相关实现的详细解释:

1. **FeeDetails**:
   - 一个结构体,用于描述交易费用的详细信息.它包含了四个字段:`inclusion_fee`,`tip`,`extra_fee`和`final_fee`.
   - `inclusion_fee`是交易被纳入区块所需的最小费用,只有支付费用的交易(`Pays::Yes`)才会有纳入费用.
   - `tip`是交易中可能包含的小费,只有签名交易才能有小费.
   - `extra_fee`是一些特定于ChainX调用的额外费用.
   - `final_fee`是最终的费用,由纳入费用,小费和额外费用组成.

2. **实现**:
   - `FeeDetails`实现了`Encode`,`Decode`,`Clone`,`Eq`,`PartialEq`,`RuntimeDebug`和`TypeInfo` trait,
   这些trait使得`FeeDetails`可以被序列化和反序列化,在Arc之间复制,进行比较,调试打印和类型信息查询.
   - 如果启用了`std` feature,`FeeDetails`还会实现`Serialize`和`Deserialize` trait,允许它使用Serde库进行JSON序列化和反序列化.

3. **new**方法:
   - 这是一个辅助方法,用于根据`pallet_transaction_payment::FeeDetails`创建一个新的`FeeDetails`实例.
   它接受基础费用详情和额外费用作为参数,并计算最终费用.

4. **From trait**:
   - `FeeDetails`实现了`From` trait,允许它从`pallet_transaction_payment::FeeDetails`类型转换而来.这个转换计算最终费用,并初始化`FeeDetails`的字段.

5. **依赖项**:
   - `codec`库用于序列化和反序列化.
   - `scale_info`库用于类型信息.
   - `sp_runtime`库提供了运行时所需的trait,如`RuntimeDebug`和`AtLeast32BitUnsigned`.
   - `pallet_transaction_payment`模块提供了交易费用计算的基础设施.

`FeeDetails`结构体为ChainX区块链项目提供了一种标准化的方法来表示和处理交易费用,这对于节点操作者,前端应用程序和用户理解交易费用结构非常重要.
通过提供这些详细信息,ChainX项目能够确保交易费用的透明度和一致性.
*/
