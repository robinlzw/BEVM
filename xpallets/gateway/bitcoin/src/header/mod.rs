// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

mod header_proof;

use frame_support::log::{error, info};
use sp_runtime::DispatchResult;
use sp_std::{cmp::Ordering, prelude::*};

use light_bitcoin::primitives::{hash_rev, H256};

use crate::{
    types::{BtcHeaderIndex, BtcHeaderInfo},
    Config, ConfirmedIndex, Error, MainChain, Pallet,
};

pub use self::header_proof::HeaderVerifier;

/// Look back the headers to pick the confirmed index,
/// return the header indexes on the look back path.
///
/// The definition of block confirmation count:
/// confirmed_height = now_height - (confirmations - 1)
///           |--- confirmations = 4 ---|
/// b(prev) - b(confirm)  -  b  -  b  - b
///           4              3     2    1       (confirmations)
///           97             98    99   100     (height)
///
fn look_back_confirmed_header<T: Config>(
    header_info: &BtcHeaderInfo,
) -> (Option<BtcHeaderIndex>, Vec<BtcHeaderIndex>) {
    let confirmations = Pallet::<T>::confirmation_number();
    let mut chain = Vec::with_capacity(confirmations as usize);
    let mut prev_hash = header_info.header.previous_header_hash;

    // put current header
    chain.push(BtcHeaderIndex {
        hash: header_info.header.hash(),
        height: header_info.height,
    });
    // e.g. when confirmations is 4, loop 3 times max
    for cnt in 1..confirmations {
        if let Some(current_info) = Pallet::<T>::headers(&prev_hash) {
            chain.push(BtcHeaderIndex {
                hash: prev_hash,
                height: current_info.height,
            });
            prev_hash = current_info.header.previous_header_hash;
        } else {
            // if cannot find current header info, should be exceed genesis height, jump out of loop
            // e.g. want to get the previous header of #98, but genesis height is 98,
            // obviously, we cannot find the header of #97.
            info!(
                target: "runtime::bitcoin",
                "[update_confirmed_header] Can not find header ({:?}), current reverse count:{}",
                hash_rev(prev_hash),
                cnt
            );
            break;
        }
    }
    if chain.len() == confirmations as usize {
        // confirmations must more than 0, thus, chain.last() must be some
        (chain.last().cloned(), chain)
    } else {
        (None, chain)
    }
}

pub fn update_confirmed_header<T: Config>(header_info: &BtcHeaderInfo) -> Option<BtcHeaderIndex> {
    let (confirmed, chain) = look_back_confirmed_header::<T>(header_info);
    for index in chain {
        set_main_chain::<T>(index.height, index.hash);
    }
    confirmed.map(|index| {
        ConfirmedIndex::<T>::put(index);
        index
    })
}

fn set_main_chain<T: Config>(height: u32, main_hash: H256) {
    let hashes = Pallet::<T>::block_hash_for(&height);
    if hashes.len() == 1 {
        MainChain::<T>::insert(&hashes[0], true);
        return;
    }
    for hash in hashes {
        if hash == main_hash {
            MainChain::<T>::insert(&hash, true);
        } else {
            MainChain::<T>::remove(&hash);
        }
    }
}

pub fn check_confirmed_header<T: Config>(header_info: &BtcHeaderInfo) -> DispatchResult {
    let (confirmed, _) = look_back_confirmed_header::<T>(header_info);
    if let Some(current_confirmed) = ConfirmedIndex::<T>::get() {
        if let Some(now_confirmed) = confirmed {
            return match current_confirmed.height.cmp(&now_confirmed.height) {
                Ordering::Greater => {
                    // e.g:
                    //          current_confirmed
                    // b  ---------------- b  ------ b --- b --- b(best)
                    // |(now_confirmed)--- b  ------ b --- b(now)
                    // 99              100       101  102    103
                    // current_confirmed > now_confirmed
                    Ok(())
                }
                Ordering::Equal => {
                    // e.g:
                    //current_confirmed
                    // b --------------- b  ------ b --- b(best)
                    // |(now_confirmed)- b  ------ b --- b(now)
                    // 99              100       101  102    103
                    // current_confirmed = now_confirmed
                    if current_confirmed.hash == now_confirmed.hash {
                        Ok(())
                    } else {
                        // e.g:
                        //
                        //  b --------- b(current_confirmed) b  ------ b --- b(best)
                        //  | --------- b(now_confirmed) --- b  ------ b --- b(now)
                        // 99              100       101  102    103
                        // current_confirmed = now_confirmed
                        Err(Error::<T>::AncientFork.into())
                    }
                }
                Ordering::Less => {
                    // normal should not happen, for call `check_confirmed_header` should under
                    // current <= best
                    error!(
                        "[check_confirmed_header] Should not happen, current confirmed is less than confirmed for this header, \
                        current:{:?}, now:{:?}", current_confirmed, now_confirmed
                    );
                    Err(Error::<T>::AncientFork.into())
                }
            };
        }
    }
    // do not have confirmed yet.
    Ok(())
}

/*
这段代码是 ChainX 项目中用于处理比特币区块头确认的逻辑.它定义了一系列函数,用于在 ChainX 区块链上跟踪和验证比特币区块头的确认状态.
以下是代码中各个组件的详细解释:

### look_back_confirmed_header 函数

- 此函数用于查找并返回一个比特币区块头的确认高度和到达该高度的路径上的区块头索引.它根据当前链的高度和所需的确认数来确定目标区块头.

### update_confirmed_header 函数

- 此函数使用 `look_back_confirmed_header` 函数来更新已确认的区块头索引.它将查找到的路径上的每个区块头标记为主链,并返回最终的已确认区块头索引.

### set_main_chain 函数

- 此函数用于将特定高度的区块头标记为主链.它会检查给定高度的所有区块头,并根据提供的主哈希来更新主链状态.

### check_confirmed_header 函数

- 此函数用于检查给定的区块头是否已经达到足够的确认数.它会比较当前已确认的区块头和提供的区块头的高度和哈希值,以确定它们之间的关系.

### 总结

这些函数共同构成了 ChainX 项目中比特币区块头确认逻辑的核心.它们确保 ChainX 区块链能够正确地跟踪比特币网络的状态,并在比特币区块被确认后更新 ChainX 区块链上的状态.
这对于跨链桥接和资产转移的安全性至关重要,因为它防止了双重支付和其他潜在的安全问题.通过这些验证,ChainX 项目能够确保其与比特币网络的同步,并维护其跨链功能的可靠性.

-----------------------------------------------------------------------------------------------------------------------------------------------
ChainX 能够跟踪比特币网络的状态是因为它实现了一个与比特币网络交互的系统,这个系统能够监听比特币区块链上的区块和交易,从而获取和验证比特币网络的数据.
以下是 ChainX 实现这一功能的关键组件和原理:

1. **比特币区块头提交**:ChainX 节点会监听比特币网络,并获取比特币区块头信息.区块头包含了区块的大部分信息,但不包括完整的交易数据.这种方式可以减少数据传输的带宽需求.

2. **区块头验证**:ChainX 使用特定的逻辑来验证接收到的比特币区块头是否有效.这包括检查工作量证明(Proof of Work, PoW),时间戳,难度目标等,确保区块头符合比特币网络的共识规则.

3. **确认机制**:ChainX 通过累积确认来增加对比特币区块头的信任度.当一个区块被比特币网络确认一定次数后,ChainX 将其视为稳定,并可以在此基础上执行跨链操作.

4. **主链跟踪**:ChainX 维护一个主链列表,用于跟踪比特币区块链上的有效链.这有助于识别和排除孤立的区块或分叉.

5. **跨链桥接逻辑**:ChainX 的跨链桥接逻辑使用比特币区块头信息来锁定和转移资产.例如,当比特币被发送到 ChainX 时,ChainX 会等待足够的确认来确保比特币交易不可逆.

6. **智能合约和 Pallet**:ChainX 使用 Substrate 框架开发的智能合约和 Pallet 来处理比特币数据.这些 Pallet 负责存储比特币区块头信息,处理比特币资产转移和执行其他与比特币相关的操作.

通过这些机制,ChainX 能够安全地与比特币网络同步,并在其区块链上执行基于比特币状态的跨链操作.这使得 ChainX 能够利用比特币的安全性和去中心化特性,同时为用户提供更广泛的资产和应用.

*/
