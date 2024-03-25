// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use codec::{Codec, Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use sp_runtime::RuntimeDebug;

use chainx_primitives::{AddrStr, AssetId};
use xp_runtime::Memo;

/// The id of withdrawal record (u32 is enough).
pub type WithdrawalRecordId = u32;

/// The state machine of WithdrawState:
///
/// Applying (lock token) <---> Processing (can't cancel, but can be recovered to `Applying`)
///     |                           |
///     |                           +----> NormalFinish|RootFinish (destroy token)
///     |                           |
///     |                           +----> RootCancel (unlock token)
///     |                           |
///     +---------------------------+----> NormalCancel (unlock token)
///
#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum WithdrawalState {
    Applying,
    Processing,
    NormalFinish,
    RootFinish,
    NormalCancel,
    RootCancel,
}

impl Default for WithdrawalState {
    fn default() -> Self {
        WithdrawalState::Applying
    }
}

/// WithdrawalRecord for withdrawal
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct WithdrawalRecord<AccountId, Balance, BlockNumber> {
    asset_id: AssetId,
    applicant: AccountId,
    balance: Balance,
    addr: AddrStr,
    ext: Memo,
    height: BlockNumber,
}

impl<AccountId, Balance, BlockNumber> WithdrawalRecord<AccountId, Balance, BlockNumber>
where
    AccountId: Codec + Clone,
    Balance: Codec + Copy + Clone,
    BlockNumber: Codec + Copy + Clone,
{
    pub fn new(
        applicant: AccountId,
        asset_id: AssetId,
        balance: Balance,
        addr: AddrStr,
        ext: Memo,
        height: BlockNumber,
    ) -> Self {
        Self {
            asset_id,
            applicant,
            balance,
            addr,
            ext,
            height,
        }
    }
    pub fn applicant(&self) -> &AccountId {
        &self.applicant
    }
    pub fn asset_id(&self) -> AssetId {
        self.asset_id
    }
    pub fn balance(&self) -> Balance {
        self.balance
    }
    pub fn addr(&self) -> &AddrStr {
        &self.addr
    }
    pub fn ext(&self) -> &Memo {
        &self.ext
    }
    pub fn height(&self) -> BlockNumber {
        self.height
    }
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, TypeInfo)]
pub struct Withdrawal<AccountId, Balance, BlockNumber> {
    pub asset_id: AssetId,
    pub applicant: AccountId,
    pub balance: Balance,
    pub addr: AddrStr,
    pub ext: Memo,
    pub height: BlockNumber,
    pub state: WithdrawalState,
}

impl<AccountId, Balance, BlockNumber> Withdrawal<AccountId, Balance, BlockNumber> {
    pub fn new(
        record: WithdrawalRecord<AccountId, Balance, BlockNumber>,
        state: WithdrawalState,
    ) -> Self {
        Self {
            asset_id: record.asset_id,
            applicant: record.applicant,
            balance: record.balance,
            addr: record.addr,
            ext: record.ext,
            height: record.height,
            state,
        }
    }
}

/*
这段代码是使用Rust语言编写的,它定义了一个区块链系统中的提现记录和状态机的模型.
1. `WithdrawalRecordId` 类型定义:
   - 这是一个类型别名,表示提现记录的ID,使用`u32`类型,意味着提现记录的ID是一个32位无符号整数.

2. `WithdrawalState` 枚举定义:
   - 表示提现记录可能处于的状态.这些状态包括:
     - `Applying`:申请中,表示资产已被锁定.
     - `Processing`:处理中,表示提现正在处理,此时不能取消,但可以恢复到`Applying`状态.
     - `NormalFinish`:正常完成,表示提现成功完成.
     - `RootFinish`:根完成,表示提现由系统管理员完成.
     - `NormalCancel`:正常取消,表示提现被用户正常取消.
     - `RootCancel`:根取消,表示提现被系统管理员取消.
   - 这个枚举实现了`Default` trait,意味着默认状态是`Applying`.

3. `WithdrawalRecord` 结构体定义:
   - 表示一个提现记录,包含以下字段:
     - `asset_id`:提现的资产ID.
     - `applicant`:申请人的账户ID.
     - `balance`:提现的资产数量.
     - `addr`:接收提现的地址.
     - `ext`:额外信息,如交易的备注或数据.
     - `height`:提现发生时的区块高度.
   - 提供了一个`new`方法来创建一个新的`WithdrawalRecord`实例.

4. `Withdrawal` 结构体定义:
   - 表示一个完整的提现请求,它包含了`WithdrawalRecord`的所有信息,并且增加了一个`state`字段来表示提现的状态.
   - 提供了一个`new`方法来根据`WithdrawalRecord`和状态创建一个新的`Withdrawal`实例.

这个模块的代码使用了多种Rust的特性,包括类型别名,枚举,结构体,泛型和特质(traits).它还使用了`codec`和`scale_info`库来支持序列化和反序列化,
以及`serde`库(在标准库可用的情况下)来支持JSON序列化和反序列化.这些特性使得这个模块可以方便地与其他区块链组件集成,并在区块链系统中进行资产的提现操作.
*/
