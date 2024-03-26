// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.
#![allow(clippy::ptr_arg)]
extern crate alloc;
use alloc::string::ToString;

pub mod validator;

use frame_support::{
    dispatch::DispatchResult,
    log::{self, debug, error, info, warn},
};
use sp_runtime::{traits::Zero, SaturatedConversion};
use sp_std::prelude::*;

use light_bitcoin::{
    chain::Transaction,
    keys::{Address, Network},
    primitives::{hash_rev, H256},
};
use sp_core::H160;

use chainx_primitives::AssetId;
use xp_gateway_bitcoin::{BtcDepositInfo, BtcTxMetaType, BtcTxTypeDetector, OpReturnAccount};
use xp_gateway_common::{AccountExtractor, DstChain};
use xpallet_assets::ChainT;
use xpallet_gateway_common::traits::{AddressBinding, ReferralBinding, TrusteeInfoUpdate};
use xpallet_support::try_str;

pub use self::validator::validate_transaction;
use crate::{
    types::{AccountInfo, BtcAddress, BtcDepositCache, BtcTxResult, BtcTxState},
    BalanceOf, Config, Event, Pallet, PendingDeposits, WithdrawalProposal,
};

pub fn process_tx<T: Config>(
    tx: Transaction,
    prev_tx: Option<Transaction>,
    network: Network,
    min_deposit: u64,
    current_trustee_pair: (Address, Address),
    last_trustee_pair: Option<(Address, Address)>,
) -> BtcTxState {
    let btc_tx_detector = BtcTxTypeDetector::new(network, min_deposit);
    let meta_type = btc_tx_detector.detect_transaction_type::<T::AccountId, _>(
        &tx,
        prev_tx.as_ref(),
        T::AccountExtractor::extract_account,
        current_trustee_pair,
        last_trustee_pair,
    );

    let tx_type = meta_type.ref_into();
    let result = match meta_type {
        BtcTxMetaType::<_>::Deposit(deposit_info) => deposit::<T>(tx.hash(), deposit_info),
        BtcTxMetaType::<_>::Withdrawal => withdraw::<T>(tx),
        BtcTxMetaType::TrusteeTransition => trustee_transition::<T>(tx),
        BtcTxMetaType::HotAndCold => BtcTxResult::Success,
        // mark `Irrelevance` be `Failure` so that it could be replayed in the future
        BtcTxMetaType::<_>::Irrelevance => BtcTxResult::Failure,
    };

    BtcTxState { tx_type, result }
}

fn trustee_transition<T: Config>(tx: Transaction) -> BtcTxResult {
    let amount = tx.outputs().iter().map(|output| output.value).sum::<u64>();

    T::TrusteeInfoUpdate::update_transition_status(Pallet::<T>::chain(), false, Some(amount));

    BtcTxResult::Success
}

fn deposit<T: Config>(txid: H256, deposit_info: BtcDepositInfo<T::AccountId>) -> BtcTxResult {
    // check address in op_return whether allow binding
    let deposit_info = T::AddressBinding::check_allowed_binding(deposit_info);
    let account_info = match (deposit_info.op_return, deposit_info.input_addr) {
        (Some((account, referral)), Some(input_addr)) => {
            let input_addr = input_addr.to_string().into_bytes();
            // remove old unbinding deposit info
            remove_pending_deposit::<T>(&input_addr, &account);
            // update or override binding info
            T::AddressBinding::update_binding(Pallet::<T>::chain(), input_addr, account.clone());
            AccountInfo::<T::AccountId>::Account((account, referral))
        }
        (Some((account, referral)), None) => {
            // has opreturn but no input addr
            debug!(
                target: "runtime::bitcoin",
                "[deposit] Deposit tx ({:?}) has no input addr, but has opreturn, who:{:?}",
                hash_rev(txid),
                account
            );
            AccountInfo::<T::AccountId>::Account((account, referral))
        }
        (None, Some(input_addr)) => {
            // no opreturn but have input addr, use input addr to get accountid
            let addr_bytes = input_addr.to_string().into_bytes();
            match T::AddressBinding::address(Pallet::<T>::chain(), addr_bytes) {
                Some(account) => AccountInfo::Account((account, None)),
                None => AccountInfo::Address(input_addr),
            }
        }
        (None, None) => {
            warn!(
                target: "runtime::bitcoin",
                "[deposit] Process deposit tx ({:?}) but missing valid opreturn and input addr",
                hash_rev(txid)
            );
            return BtcTxResult::Failure;
        }
    };

    match account_info {
        AccountInfo::<_>::Account((account, referral)) => {
            if let OpReturnAccount::Wasm(w) = account.clone() {
                T::ReferralBinding::update_binding(
                    &<Pallet<T> as ChainT<_>>::ASSET_ID,
                    &w,
                    referral,
                );
            }

            match deposit_token::<T>(txid, &account, deposit_info.deposit_value) {
                Ok(_) => {
                    info!(
                        target: "runtime::bitcoin",
                        "[deposit] Deposit tx ({:?}) success, who:{:?}, balance:{}",
                        hash_rev(txid),
                        account,
                        deposit_info.deposit_value
                    );
                    BtcTxResult::Success
                }
                Err(_) => BtcTxResult::Failure,
            }
        }
        AccountInfo::<_>::Address(input_addr) => {
            insert_pending_deposit::<T>(&input_addr, txid, deposit_info.deposit_value);
            info!(
                target: "runtime::bitcoin",
                "[deposit] Deposit tx ({:?}) into pending, addr:{:?}, balance:{}",
                hash_rev(txid),
                try_str(input_addr.to_string().into_bytes()),
                deposit_info.deposit_value
            );
            BtcTxResult::Success
        }
    }
}

fn deposit_token<T: Config>(
    txid: H256,
    who: &OpReturnAccount<T::AccountId>,
    balance: u64,
) -> DispatchResult {
    match who {
        OpReturnAccount::Evm(w) => deposit_evm::<T>(txid, w, balance),
        OpReturnAccount::Wasm(w) => deposit_wasm::<T>(txid, w, balance),
        OpReturnAccount::Aptos(w) => deposit_aptos::<T>(txid, w, balance),
        OpReturnAccount::Named(w1, w2) => deposit_named::<T>(txid, w1.clone(), w2.clone(), balance),
    }
}

fn deposit_wasm<T: Config>(txid: H256, who: &T::AccountId, balance: u64) -> DispatchResult {
    let id: AssetId = <Pallet<T> as ChainT<_>>::ASSET_ID;

    let value: BalanceOf<T> = balance.saturated_into();
    match <xpallet_gateway_records::Pallet<T>>::deposit(who, id, value) {
        Ok(()) => {
            Pallet::<T>::deposit_event(Event::<T>::Deposited(txid, who.clone(), value));
            Ok(())
        }
        Err(err) => {
            error!(
                target: "runtime::bitcoin",
                "[deposit_token] Deposit error:{:?}, must use root to fix it",
                err
            );
            Err(err)
        }
    }
}

fn deposit_evm<T: Config>(txid: H256, who: &H160, balance: u64) -> DispatchResult {
    let id: AssetId = <Pallet<T> as ChainT<_>>::ASSET_ID;

    match xpallet_assets_bridge::Pallet::<T>::apply_direct_deposit(*who, id, balance as u128) {
        Ok(_) => {
            Pallet::<T>::deposit_event(Event::<T>::DepositedEvm(
                txid,
                *who,
                balance.saturated_into(),
            ));
            Ok(())
        }
        Err(err) => {
            error!(
                target: "runtime::bitcoin",
                "[deposit_token] Deposit error:{:?}, must use root to fix it",
                err
            );
            Err(err)
        }
    }
}

fn deposit_aptos<T: Config>(txid: H256, who: &H256, balance: u64) -> DispatchResult {
    let id: AssetId = <Pallet<T> as ChainT<_>>::ASSET_ID;
    let value: BalanceOf<T> = balance.saturated_into();

    if let Some(proxy_address) = T::AddressBinding::dst_chain_proxy_address(DstChain::Aptos) {
        match <xpallet_gateway_records::Pallet<T>>::deposit(&proxy_address, id, value) {
            Ok(()) => {
                Pallet::<T>::deposit_event(Event::<T>::DepositedAptos(txid, *who, value));
            }
            Err(err) => {
                error!(
                    target: "runtime::bitcoin",
                    "[deposit_token] Deposit error:{:?}, must use root to fix it",
                    err
                );
                return Err(err);
            }
        }
    }
    Ok(())
}

fn deposit_named<T: Config>(
    txid: H256,
    prefix: Vec<u8>,
    who: Vec<u8>,
    balance: u64,
) -> DispatchResult {
    let id: AssetId = <Pallet<T> as ChainT<_>>::ASSET_ID;
    let value: BalanceOf<T> = balance.saturated_into();

    if let Some(proxy_address) =
        T::AddressBinding::dst_chain_proxy_address(DstChain::Named(prefix.clone()))
    {
        match <xpallet_gateway_records::Pallet<T>>::deposit(&proxy_address, id, value) {
            Ok(()) => {
                Pallet::<T>::deposit_event(Event::<T>::DepositedNamed(txid, prefix, who, value));
            }
            Err(err) => {
                error!(
                    target: "runtime::bitcoin",
                    "[deposit_token] Deposit error:{:?}, must use root to fix it",
                    err
                );
                return Err(err);
            }
        }
    }
    Ok(())
}

/*
这段 Rust 代码定义了一个名为 `remove_pending_deposit` 的函数,它用于从 ChainX 区块链的待处理存款缓存中移除特定比特币地址的记录.
这个函数是针对 ChainX 区块链项目中的跨链交互模块的,用于处理比特币网络中的待处理存款.

函数的参数和逻辑如下:

1. `input_address`: 一个指向 `BtcAddress` 的引用,表示要移除待处理存款记录的比特币地址.

2. `who`: 一个指向 `OpReturnAccount<T::AccountId>` 的引用,表示相关账户的信息.`OpReturnAccount` 是一个枚举,
用于表示不同区块链上的账户类型(如以太坊,WASM 合约,Aptos 等).

函数的主要逻辑:

- 使用 `PendingDeposits::<T>::take(input_address)` 从 `PendingDeposits` 映射中取出与 `input_address` 
相关联的所有待处理存款记录.`take` 方法会移除这些记录并返回它们.

- 遍历取出的记录,对于每个记录:
  - 调用 `deposit_token::<T>` 函数尝试重新将存款存入指定账户.这个函数的具体实现没有在代码片段中给出,但它应该负责处理将比特币转移到正确的账户.
  - 使用 `info!` 宏记录一条信息,包含被移除的待处理存款的详细信息,如账户信息,余额,交易 ID 和比特币地址.
  - 根据 `who` 的类型,触发不同的事件,通知系统待处理存款已被移除.这些事件允许其他模块或外部观察者了解待处理存款状态的变化.

这个函数的目的是确保待处理存款的记录能够被正确移除,并且在移除过程中尝试重新存款.同时,通过记录事件,其他模块可以响应这些变化,
例如更新用户界面或执行其他相关的逻辑.

潜在的风险点包括:

- **存款失败**:如果 `deposit_token::<T>` 函数在尝试重新存款时失败,可能会导致资金丢失或账户余额不准确.
- **事件触发错误**:如果事件触发逻辑不正确,可能会导致错误的事件被记录,从而影响其他模块的行为.
- **数据一致性**:在移除记录和尝试重新存款的过程中,需要确保数据的一致性,避免在并发操作或系统故障时出现数据丢失或损坏.

为了降低这些风险,应当确保 `deposit_token::<T>` 函数的实现是健壮的,并且系统能够正确处理并发操作.
此外,应当对移除操作和事件触发逻辑进行充分的测试.
*/
pub fn remove_pending_deposit<T: Config>(
    input_address: &BtcAddress,
    who: &OpReturnAccount<T::AccountId>,
) {
    // notice this would delete this cache
    let records = PendingDeposits::<T>::take(input_address);
    for record in records {
        // ignore error
        let _ = deposit_token::<T>(record.txid, who, record.balance);
        info!(
            target: "runtime::bitcoin",
            "[remove_pending_deposit] Use pending info to re-deposit, who:{:?}, balance:{}, cached_tx:{:?}",
            who, record.balance, record.txid,
        );

        match who.clone() {
            OpReturnAccount::Evm(w) => {
                Pallet::<T>::deposit_event(Event::<T>::PendingDepositEvmRemoved(
                    w,
                    record.balance.saturated_into(),
                    record.txid,
                    input_address.clone(),
                ));
            }
            OpReturnAccount::Wasm(w) => {
                Pallet::<T>::deposit_event(Event::<T>::PendingDepositRemoved(
                    w,
                    record.balance.saturated_into(),
                    record.txid,
                    input_address.clone(),
                ));
            }
            OpReturnAccount::Aptos(w) => {
                Pallet::<T>::deposit_event(Event::<T>::PendingDepositAptosRemoved(
                    w,
                    record.balance.saturated_into(),
                    record.txid,
                    input_address.clone(),
                ));
            }
            OpReturnAccount::Named(w1, w2) => {
                Pallet::<T>::deposit_event(Event::<T>::PendingDepositNamedRemoved(
                    w1.clone(),
                    w2.clone(),
                    record.balance.saturated_into(),
                    record.txid,
                    input_address.clone(),
                ));
            }
        }
    }
}

fn insert_pending_deposit<T: Config>(input_addr: &Address, txid: H256, balance: u64) {
    let addr_bytes = input_addr.to_string().into_bytes();

    let cache = BtcDepositCache { txid, balance };

    PendingDeposits::<T>::mutate(&addr_bytes, |list| {
        if !list.contains(&cache) {
            log::debug!(
                target: "runtime::bitcoin",
                "[insert_pending_deposit] Add pending deposit, address:{:?}, txhash:{:?}, balance:{}",
                try_str(&addr_bytes),
                txid,
                balance
            );
            list.push(cache);

            Pallet::<T>::deposit_event(Event::<T>::UnclaimedDeposit(txid, addr_bytes.clone()));
        }
    });
}

fn withdraw<T: Config>(tx: Transaction) -> BtcTxResult {
    if let Some(proposal) = WithdrawalProposal::<T>::take() {
        log::debug!(
            target: "runtime::bitcoin",
            "[withdraw] Withdraw tx {:?}, proposal:{:?}",
            proposal,
            tx
        );
        let proposal_hash = proposal.tx.hash();
        let tx_hash = tx.hash();

        if proposal_hash == tx_hash {
            // Check if the transaction is normal witness
            let input = &tx.inputs()[0];
            if input.script_witness.len() != 3 {
                error!(
                    target: "runtime::bitcoin",
                    "[withdraw] Withdraw tx {:?} is not normal witness, proposal:{:?}",
                    tx,
                    proposal
                );
                return BtcTxResult::Failure;
            }

            let mut total = BalanceOf::<T>::zero();
            for number in proposal.withdrawal_id_list.iter() {
                // just for event record
                let withdraw_balance =
                    xpallet_gateway_records::Pallet::<T>::pending_withdrawals(number)
                        .map(|record| record.balance())
                        .unwrap_or_else(BalanceOf::<T>::zero);
                total += withdraw_balance;

                match xpallet_gateway_records::Pallet::<T>::finish_withdrawal(*number, None) {
                    Ok(_) => {
                        info!(target: "runtime::bitcoin", "[withdraw] Withdrawal ({}) completion", *number);
                    }
                    Err(err) => {
                        error!(
                            target: "runtime::bitcoin",
                            "[withdraw] Withdrawal ({}) error:{:?}, must use root to fix it",
                            *number, err
                        );
                    }
                }
            }

            let btc_withdrawal_fee = Pallet::<T>::btc_withdrawal_fee();
            // real withdraw value would reduce withdraw_fee
            total -=
                (proposal.withdrawal_id_list.len() as u64 * btc_withdrawal_fee).saturated_into();

            // Record trustee signature
            T::TrusteeInfoUpdate::update_trustee_sig_record(
                Pallet::<T>::chain(),
                input.script_witness[1].as_slice(),
                total.saturated_into(),
            );

            Pallet::<T>::deposit_event(Event::<T>::Withdrawn(
                tx_hash,
                proposal.withdrawal_id_list,
                total,
            ));
            BtcTxResult::Success
        } else {
            error!(
                target: "runtime::bitcoin",
                "[withdraw] Withdraw error: mismatch (tx_hash:{:?}, proposal_hash:{:?}), id_list:{:?}, must use root to fix it",
                tx_hash, proposal_hash, proposal.withdrawal_id_list
            );
            // re-store proposal into storage.
            WithdrawalProposal::<T>::put(proposal);

            Pallet::<T>::deposit_event(Event::<T>::WithdrawalFatalErr(proposal_hash, tx_hash));
            BtcTxResult::Failure
        }
    } else {
        error!(
            target: "runtime::bitcoin",
            "[withdraw] Withdrawal error: proposal is EMPTY (tx_hash:{:?}), but receive a withdrawal tx, must use root to fix it",
            tx.hash()
        );
        // no proposal, but find a withdraw tx, it's a fatal error in withdrawal
        Pallet::<T>::deposit_event(Event::<T>::WithdrawalFatalErr(
            tx.hash(),
            Default::default(),
        ));

        BtcTxResult::Failure
    }
}

/*
这段代码是 ChainX 项目中处理比特币交易的逻辑,特别是在其比特币网关模块中.它定义了一系列函数,
用于处理不同类型的比特币交易,包括存款,取款以及受托人转换.以下是代码中各个函数的详细解释:

### 主要函数

- **process_tx**: 这是处理比特币交易的主要入口点.它使用 `BtcTxTypeDetector` 来检测交易类型,并根据交易类型执行相应的处理逻辑.

### 交易处理逻辑

- **trustee_transition**: 处理受托人转换的逻辑.这通常涉及到更新受托人的状态.

- **deposit**: 处理存款逻辑.它会检查交易的输出地址是否允许绑定,并根据操作码返回地址(op_return)和
输入地址来更新或创建新的账户信息.然后,它会尝试将存款的比特币转换为 ChainX 链上的资产.

- **deposit_token**: 根据账户类型(EVM,WASM,Aptos 或 Named)将比特币存款转换为 ChainX 链上的资产.

- **deposit_wasm**, **deposit_evm**, **deposit_aptos**, **deposit_named**: 
这些函数是 `deposit_token` 的特化版本,用于处理不同类型的账户和资产转换.

### 辅助函数

- **remove_pending_deposit**: 移除挂起的存款记录,并尝试重新存款.

- **insert_pending_deposit**: 将新的存款记录插入到挂起的存款缓存中.

- **withdraw**: 处理取款逻辑.它会检查是否有匹配的取款提案,并验证交易是否符合预期的格式.然后,它会更新挂起的取款记录,并在成功时记录事件.

### 总结

这些函数共同构成了 ChainX 项目中比特币网关模块的核心处理逻辑.它们确保了比特币交易能够在 ChainX 链上正确处理,无论是存款,取款还是受托人转换.
*/
