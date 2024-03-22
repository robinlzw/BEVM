// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use frame_support::{
    dispatch::DispatchResult,
    log::{debug, error},
};
use sp_std::prelude::Vec;

use light_bitcoin::{chain::Transaction, primitives::H256};

use crate::{types::BtcRelayedTx, Config, Error};

pub fn validate_transaction<T: Config>(
    tx: &BtcRelayedTx,
    merkle_root: H256,
    prev_tx: Option<&Transaction>,
) -> DispatchResult {
    let tx_hash = tx.raw.hash();
    debug!(
        target: "runtime::bitcoin",
        "[validate_transaction] tx_hash:{:?}, relay tx:{:?}",
        tx_hash, tx
    );

    // verify merkle proof
    let mut matches = Vec::new();
    let mut _indexes = Vec::new();
    let hash = tx
        .merkle_proof
        .extract_matches(&mut matches, &mut _indexes)
        .map_err(|_| Error::<T>::BadMerkleProof)?;
    if merkle_root != hash {
        error!(
            target: "runtime::bitcoin",
            "[validate_transaction] Check merkle tree proof error, merkle_root:{:?}, hash:{:?}",
            merkle_root, hash
        );
        return Err(Error::<T>::BadMerkleProof.into());
    }
    if !matches.iter().any(|h| *h == tx_hash) {
        error!(
            target: "runtime::bitcoin",
            "[validate_transaction] Tx hash should in matches of partial merkle tree"
        );
        return Err(Error::<T>::BadMerkleProof.into());
    }

    if let Some(prev) = prev_tx {
        // verify prev tx for input
        // only check the first(0) input in transaction
        let previous_txid = prev.hash();
        let expected_id = tx.raw.inputs[0].previous_output.txid;
        if previous_txid != expected_id {
            error!(
                target: "runtime::bitcoin",
                "[validate_transaction] Relay previous tx's hash not equal to relay tx first input, expected_id:{:?}, prev:{:?}",
                expected_id, previous_txid
            );
            return Err(Error::<T>::InvalidPrevTx.into());
        }
    }
    Ok(())
}

/*
这段代码是 ChainX 项目中用于验证比特币交易的逻辑,特别是在其比特币网关模块中.`validate_transaction` 函数是该逻辑的核心,
它负责验证通过 ChainX 网关中继的比特币交易是否有效.以下是对该函数的详细解释:

### 函数详情

- **validate_transaction**: 接收一个 `BtcRelayedTx` 类型的参数(代表中继的比特币交易),一个 `H256` 类型的 
`merkle_root`(代表默克尔树的根哈希),以及一个可选的 `Transaction` 类型的 `prev_tx` 参数(代表交易的前一个交易,用于验证输入).

### 验证逻辑

1. **验证默克尔证明**: 函数首先使用 `extract_matches` 方法从提供的默克尔证明中提取匹配项,并计算得到默克尔根哈希.
然后,它比较计算得到的默克尔根哈希与预期的 `merkle_root` 是否相同.如果不相等,说明默克尔证明无效,函数将返回错误.

2. **检查交易哈希是否匹配**: 如果默克尔根哈希验证通过,函数接下来检查交易哈希是否包含在默克尔证明的匹配项中.
如果不包含,说明交易哈希不在默克尔树的证明中,这也是一个错误.

3. **验证前一个交易**: 如果提供了 `prev_tx`,函数将继续验证交易的输入.它比较交易的第一个输入的
 `previous_output.txid`(即前一个输出的交易哈希)是否与 `prev_tx` 的哈希相等.如果不相等,说明交易的输入不正确,函数将返回错误.

### 总结

`validate_transaction` 函数是 ChainX 项目中确保比特币交易有效性的关键环节.它通过默克尔证明验证交易是否包含在比特币区块中,
并检查交易的输入是否正确.这些验证步骤对于防止双重支付和其他潜在的欺诈行为至关重要.
*/
