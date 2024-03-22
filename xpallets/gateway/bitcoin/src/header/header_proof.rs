// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use frame_support::{
    dispatch::DispatchResult,
    log::{debug, error, info, warn},
    traits::UnixTime,
};
use sp_runtime::RuntimeDebug;
use sp_std::{cmp, convert::TryFrom};

use light_bitcoin::{
    chain::BlockHeader as BtcHeader,
    keys::Network,
    primitives::{hash_rev, Compact, H256, U256},
};

use crate::{
    types::{BtcHeaderInfo, BtcParams},
    Config, Error, Pallet,
};

pub struct HeaderVerifier<'a> {
    pub work: HeaderWork<'a>,
    pub proof_of_work: HeaderProofOfWork<'a>,
    pub timestamp: HeaderTimestamp<'a>,
}

impl<'a> HeaderVerifier<'a> {
    pub fn new<T: Config>(header_info: &'a BtcHeaderInfo) -> Self {
        let now = T::UnixTime::now();
        // if convert from u64 to u32 failed (unix timestamp should not be greater than u32::MAX),
        // ignore timestamp check, timestamp check are not important
        let current_time = u32::try_from(now.as_secs()).ok();

        Self {
            work: HeaderWork::new(header_info),
            proof_of_work: HeaderProofOfWork::new(&header_info.header),
            timestamp: HeaderTimestamp::new(&header_info.header, current_time),
        }
    }

    pub fn check<T: Config>(&self) -> DispatchResult {
        let params: BtcParams = Pallet::<T>::params_info();
        let network_id: Network = Pallet::<T>::network_id();
        if let Network::Mainnet = network_id {
            self.work.check::<T>(&params)?;
        }
        self.proof_of_work.check::<T>(&params)?;
        // ignore this in benchmarks
        #[cfg(not(feature = "runtime-benchmarks"))]
        self.timestamp.check::<T>(&params)?;

        Ok(())
    }
}

#[derive(RuntimeDebug)]
pub enum RequiredWork {
    Value(Compact),
    NotCheck,
}

pub struct HeaderWork<'a> {
    info: &'a BtcHeaderInfo,
}

impl<'a> HeaderWork<'a> {
    fn new(info: &'a BtcHeaderInfo) -> Self {
        HeaderWork { info }
    }

    fn check<T: Config>(&self, params: &BtcParams) -> DispatchResult {
        let previous_header_hash = self.info.header.previous_header_hash;
        let work = work_required::<T>(previous_header_hash, self.info.height, params);
        match work {
            RequiredWork::Value(work) => {
                if work != self.info.header.bits {
                    error!(
                        target: "runtime::bitcoin",
                        "[check_header_work] nBits do not match difficulty rules, work:{:?}, header bits:{:?}, height:{}",
                        work, self.info.header.bits, self.info.height
                    );
                    return Err(Error::<T>::HeaderNBitsNotMatch.into());
                }
                Ok(())
            }
            RequiredWork::NotCheck => Ok(()),
        }
    }
}

pub fn work_required<T: Config>(
    parent_hash: H256,
    height: u32,
    params: &BtcParams,
) -> RequiredWork {
    let max_bits = params.max_bits();
    if height == 0 {
        return RequiredWork::Value(max_bits);
    }

    let parent_header: BtcHeader = Pallet::<T>::headers(&parent_hash)
        .expect("pre header must exist here")
        .header;

    if is_retarget_height(height, params) {
        let new_work = work_required_retarget::<T>(parent_header, height, params);
        info!(
            target: "runtime::bitcoin",
            "[work_required] Retarget new work required, height:{}, retargeting_interval:{}, new_work:{:?}",
            height, params.retargeting_interval(), new_work
        );
        return new_work;
    }
    debug!(
        target: "runtime::bitcoin",
        "[work_required] Use old work required, old bits:{:?}",
        parent_header.bits
    );
    RequiredWork::Value(parent_header.bits)
}

fn is_retarget_height(height: u32, params: &BtcParams) -> bool {
    height % params.retargeting_interval() == 0
}

/// Algorithm used for retargeting work every 2 weeks
fn work_required_retarget<T: Config>(
    parent_header: BtcHeader,
    height: u32,
    params: &BtcParams,
) -> RequiredWork {
    let retarget_num = height - params.retargeting_interval();

    // timestamp of parent block
    let last_timestamp = parent_header.time;
    // bits of last block
    let last_bits = parent_header.bits;

    let (_, genesis_height) = Pallet::<T>::genesis_info();
    let mut retarget_header = parent_header;
    if retarget_num < genesis_height {
        // retarget_header = genesis_header;
        return RequiredWork::NotCheck;
    } else {
        let hash_list = Pallet::<T>::block_hash_for(&retarget_num);
        for h in hash_list {
            // look up in main chain
            if Pallet::<T>::main_chain(h) {
                let info = Pallet::<T>::headers(h).expect("block header must exist at here.");
                retarget_header = info.header;
                break;
            };
        }
    }

    // timestamp of block(height - RETARGETING_INTERVAL)
    let retarget_timestamp = retarget_header.time;

    let mut retarget: U256 = last_bits.into();
    let maximum: U256 = params.max_bits().into();

    retarget *= U256::from(retarget_timespan(
        retarget_timestamp,
        last_timestamp,
        params,
    ));
    retarget /= U256::from(params.target_timespan_seconds());

    debug!(
        target: "runtime::bitcoin",
        "[work_required_retarget] retarget:{}, maximum:{:?}",
        retarget, maximum
    );

    RequiredWork::Value(if retarget > maximum {
        params.max_bits()
    } else {
        retarget.into()
    })
}

/// Returns constrained number of seconds since last retarget
fn retarget_timespan(retarget_timestamp: u32, last_timestamp: u32, params: &BtcParams) -> u32 {
    // TODO i64??
    // subtract unsigned 32 bit numbers in signed 64 bit space in
    // order to prevent underflow before applying the range constraint.
    let timespan = last_timestamp as i64 - i64::from(retarget_timestamp);
    range_constrain(
        timespan,
        i64::from(params.min_timespan()),
        i64::from(params.max_timespan()),
    ) as u32
}

fn range_constrain(value: i64, min: i64, max: i64) -> i64 {
    cmp::min(cmp::max(value, min), max)
}

pub struct HeaderProofOfWork<'a> {
    header: &'a BtcHeader,
}

impl<'a> HeaderProofOfWork<'a> {
    fn new(header: &'a BtcHeader) -> Self {
        Self { header }
    }

    fn check<T: Config>(&self, params: &BtcParams) -> DispatchResult {
        if is_valid_proof_of_work(params.max_bits(), self.header.bits, self.header.hash()) {
            Ok(())
        } else {
            Err(Error::<T>::InvalidPoW.into())
        }
    }
}

fn is_valid_proof_of_work(max_work_bits: Compact, bits: Compact, hash: H256) -> bool {
    match (max_work_bits.to_u256(), bits.to_u256()) {
        (Ok(maximum), Ok(target)) => {
            let value = U256::from(hash_rev(hash).as_bytes());
            target <= maximum && value <= target
        }
        _ => false,
    }
}

pub struct HeaderTimestamp<'a> {
    header: &'a BtcHeader,
    current_time: Option<u32>,
}

impl<'a> HeaderTimestamp<'a> {
    fn new(header: &'a BtcHeader, current_time: Option<u32>) -> Self {
        Self {
            header,
            current_time,
        }
    }

    #[allow(unused)]
    fn check<T: Config>(&self, params: &BtcParams) -> DispatchResult {
        if let Some(current_time) = self.current_time {
            if self.header.time > current_time + params.block_max_future() {
                error!(
                    target: "runtime::bitcoin",
                    "[check_header_timestamp] Header time:{}, current time:{}, max_future{:?}",
                    self.header.time,
                    current_time,
                    params.block_max_future()
                );
                Err(Error::<T>::HeaderFuturisticTimestamp.into())
            } else {
                Ok(())
            }
        } else {
            // if get chain timestamp error, just ignore blockhead time check
            warn!(
                target: "runtime::bitcoin",
                "[check_header_timestamp] Header:{:?}, get unix timestamp error, ignore it",
                hash_rev(self.header.hash())
            );
            Ok(())
        }
    }
}

/*
这段代码是 ChainX 项目中用于比特币区块头验证的逻辑实现.它定义了一系列结构体和函数,用于检查比特币区块头的有效性,
包括工作量证明(Proof of Work, PoW),时间戳和难度目标(difficulty target).以下是代码中各个组件的详细解释:

### HeaderVerifier 结构体

- `HeaderVerifier`: 用于验证比特币区块头的各个组成部分.它包含三个字段:`work`,`proof_of_work` 和 `timestamp`,分别用于检查区块头的工作量证明,难度和时间戳.

### HeaderWork 结构体

- `HeaderWork`: 用于验证区块头的难度目标.它检查当前区块的难度目标是否与预期相符.

### work_required 函数

- `work_required`: 计算给定区块高度所需的工作量.如果当前高度是调整难度的目标高度,则会重新计算新的难度目标.

### is_retarget_height 函数

- `is_retarget_height`: 判断当前高度是否是调整难度的目标高度.

### work_required_retarget 函数

- `work_required_retarget`: 在达到调整难度的目标高度时,重新计算新的难度目标.这通常是基于过去几个区块的平均时间来调整的.

### retarget_timespan 函数

- `retarget_timespan`: 计算自上次难度调整以来经过的时间.这个时间跨度用于在重新计算难度目标时进行调整.

### range_constrain 函数

- `range_constrain`: 将一个整数限制在一个给定的范围内.

### HeaderProofOfWork 结构体

- `HeaderProofOfWork`: 用于验证区块头的工作量证明是否有效.它检查区块头的哈希值是否满足难度目标.

### is_valid_proof_of_work 函数

- `is_valid_proof_of_work`: 检查给定的哈希值是否满足指定的难度目标.

### HeaderTimestamp 结构体

- `HeaderTimestamp`: 用于验证区块头的时间戳是否在可接受的范围内.如果时间戳过于超前,则可能表明区块头无效.

### 总结

这些组件共同构成了 ChainX 项目中比特币区块头验证的核心逻辑.它们确保了比特币区块头的有效性,防止了恶意矿工提交不满足网络规则的区块.
通过这些验证,ChainX 项目能够安全地与比特币网络同步,并确保其跨链桥接功能的可靠性.
*/
