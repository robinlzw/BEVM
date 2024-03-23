// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

use light_bitcoin::{
    chain::{BlockHeader as BtcHeader, Transaction as BtcTransaction},
    keys::Address,
    merkle::PartialMerkleTree,
    primitives::{Compact, H256},
};

use chainx_primitives::ReferralId;
use xp_gateway_bitcoin::{BtcTxType, OpReturnAccount};

/// BtcAddress is an bitcoin address encoded in base58
/// like: "1Nekoo5VTe7yQQ8WFqrva2UbdyRMVYCP1t" or "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy"
/// not layout state or public or else.
pub type BtcAddress = Vec<u8>;

#[derive(Clone, RuntimeDebug, TypeInfo)]
pub struct BtcRelayedTx {
    pub block_hash: H256,
    pub raw: BtcTransaction,
    pub merkle_proof: PartialMerkleTree,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct BtcRelayedTxInfo {
    pub block_hash: H256,
    pub merkle_proof: PartialMerkleTree,
}

impl BtcRelayedTxInfo {
    pub fn into_relayed_tx(self, tx: BtcTransaction) -> BtcRelayedTx {
        BtcRelayedTx {
            block_hash: self.block_hash,
            raw: tx,
            merkle_proof: self.merkle_proof,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct BtcHeaderInfo {
    pub header: BtcHeader,
    pub height: u32,
}

#[derive(PartialEq, Eq, Clone, Copy, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct BtcHeaderIndex {
    pub hash: H256,
    pub height: u32,
}

#[derive(PartialEq, Clone, Copy, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct BtcTxState {
    pub tx_type: BtcTxType,
    pub result: BtcTxResult,
}

#[derive(PartialEq, Clone, Copy, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum BtcTxResult {
    Success,
    Failure,
}

pub enum AccountInfo<AccountId> {
    /// A value of type `L`.
    Account((OpReturnAccount<AccountId>, Option<ReferralId>)),
    /// A value of type `R`.
    Address(Address),
}

#[derive(PartialEq, Clone, Encode, Decode, Default, RuntimeDebug, TypeInfo)]
pub struct BtcDepositCache {
    pub txid: H256,
    pub balance: u64,
}

#[derive(PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct BtcWithdrawalProposal<AccountId> {
    pub sig_state: VoteResult,
    pub withdrawal_id_list: Vec<u32>,
    pub tx: BtcTransaction,
    pub trustee_list: Vec<(AccountId, bool)>,
}

impl<AccountId> BtcWithdrawalProposal<AccountId> {
    pub fn new(
        sig_state: VoteResult,
        withdrawal_id_list: Vec<u32>,
        tx: BtcTransaction,
        trustee_list: Vec<(AccountId, bool)>,
    ) -> Self {
        Self {
            sig_state,
            withdrawal_id_list,
            tx,
            trustee_list,
        }
    }
}

#[derive(PartialEq, Clone, Copy, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum VoteResult {
    Unfinish,
    Finish,
}

#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, Default, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct BtcParams {
    max_bits: u32,
    block_max_future: u32,
    target_timespan_seconds: u32,
    target_spacing_seconds: u32,
    retargeting_factor: u32,
    retargeting_interval: u32,
    min_timespan: u32,
    max_timespan: u32,
}

impl BtcParams {
    pub fn new(
        max_bits: u32,
        block_max_future: u32,
        target_timespan_seconds: u32,
        target_spacing_seconds: u32,
        retargeting_factor: u32,
    ) -> BtcParams {
        Self {
            max_bits,
            block_max_future,
            target_timespan_seconds,
            target_spacing_seconds,
            retargeting_factor,
            retargeting_interval: target_timespan_seconds / target_spacing_seconds,
            min_timespan: target_timespan_seconds / retargeting_factor,
            max_timespan: target_timespan_seconds * retargeting_factor,
        }
    }

    pub fn max_bits(&self) -> Compact {
        Compact::new(self.max_bits)
    }
    pub fn block_max_future(&self) -> u32 {
        self.block_max_future
    }
    pub fn target_timespan_seconds(&self) -> u32 {
        self.target_timespan_seconds
    }
    pub fn retargeting_interval(&self) -> u32 {
        self.retargeting_interval
    }
    pub fn min_timespan(&self) -> u32 {
        self.min_timespan
    }
    pub fn max_timespan(&self) -> u32 {
        self.max_timespan
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum BtcTxVerifier {
    Recover,
    RuntimeInterface,
    #[cfg(any(feature = "runtime-benchmarks", test))]
    /// Test would ignore sign check and always return true
    Test,
}

impl Default for BtcTxVerifier {
    fn default() -> Self {
        Self::Recover
    }
}

/*
这段代码定义了在 Substrate 框架下与比特币网关交互所需的数据结构和类型.
它包括了比特币交易,区块头,提款提案,参数配置等的结构体和枚举类型.
以下是这些类型和结构体的详细说明:

### BtcAddress
- 一个简单的 `Vec<u8>` 类型,用于表示比特币地址.

### BtcRelayedTx
- 包含区块哈希,原始比特币交易和默克尔证明的 `BtcRelayedTx` 结构体.

### BtcRelayedTxInfo
- 包含区块哈希和默克尔证明的 `BtcRelayedTxInfo` 结构体,用于表示中继交易的信息.

### BtcHeaderInfo
- 包含比特币区块头和区块高度的 `BtcHeaderInfo` 结构体.

### BtcHeaderIndex
- 包含区块哈希和高度的 `BtcHeaderIndex` 结构体,用于索引比特币区块头.

### BtcTxState
- 包含交易类型和交易结果的 `BtcTxState` 结构体,用于表示比特币交易的状态.

### BtcTxResult
- 一个枚举类型,表示比特币交易的结果,可以是成功(Success)或失败(Failure).

### AccountInfo
- 一个枚举类型,表示账户信息,可以是一个包含 `OpReturnAccount` 和 `ReferralId` 的元组,或者是一个比特币地址.

### BtcDepositCache
- 包含交易 ID 和余额的 `BtcDepositCache` 结构体,用于缓存比特币存款信息.

### BtcWithdrawalProposal
- 包含签名状态,提款 ID 列表,比特币交易和受托人列表的 `BtcWithdrawalProposal` 结构体,用于处理比特币提款提案.

### VoteResult
- 一个枚举类型,表示投票结果,可以是未完成(Unfinish)或已完成(Finish).

### BtcParams
- 包含比特币网络参数的 `BtcParams` 结构体,如最大难度值,区块最大未来时间,目标时间跨度,目标间隔时间,难度调整因子等.

### BtcTxVerifier
- 一个枚举类型,表示比特币交易验证器的类型,可以是 Recover(恢复),
RuntimeInterface(运行时接口)或 Test(测试),其中 Test 用于基准测试和测试场景,会忽略签名检查并始终返回 true.

这些类型和结构体是构建和运行比特币网关智能合约的基础,它们用于处理比特币交易,
验证交易的有效性,管理提款提案和缓存存款信息等.通过这些定义,
Substrate 框架的比特币网关模块可以与比特币区块链进行交互,实现资产的跨链转移和交易.
*/
