// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

//! Runtime API definition required by ChainX RPC extensions.

#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::DispatchError;
use sp_std::vec::Vec;
pub use xpallet_gateway_bitcoin::{types::BtcHeaderInfo, BtcHeader, BtcWithdrawalProposal, H256};

sp_api::decl_runtime_apis! {
    pub trait XGatewayBitcoinApi<AccountId>
        where AccountId: codec::Codec
    {
        fn verify_tx_valid(
            raw_tx: Vec<u8>,
            withdrawal_id_list: Vec<u32>,
            full_amount: bool,
        ) -> Result<bool, DispatchError>;

        fn get_withdrawal_proposal() -> Option<BtcWithdrawalProposal<AccountId>>;

        fn get_genesis_info() -> (BtcHeader, u32);

        fn get_btc_block_header(txid: H256) -> Option<BtcHeaderInfo>;
    }
}

/*
这段代码定义了一个名为 `XGatewayBitcoinApi` 的运行时 API 接口,它是 ChainX 区块链 RPC 扩展所需的.
这个 API 用于与 ChainX 区块链中的比特币网关功能交互,允许外部服务查询和验证与比特币相关的信息.

### API 方法

1. **verify_tx_valid**: 此方法用于验证比特币交易的有效性.它接受原始交易数据 `raw_tx`,
一个包含提款 ID 的列表 `withdrawal_id_list`,以及一个布尔值 `full_amount`,指示是否验证全部金额.
如果交易有效,返回 `Ok(true)`;如果无效或在验证过程中遇到错误,返回 `Err(DispatchError)`.

2. **get_withdrawal_proposal**: 返回当前的比特币提款提案 `BtcWithdrawalProposal`,
如果不存在则返回 `None`.这个提案包含了提款交易的详细信息,如提款 ID 列表和目标比特币地址.

3. **get_genesis_info**: 返回比特币链的创世区块信息,包括创世区块头 `BtcHeader` 和区块版本号 `u32`.

4. **get_btc_block_header**: 根据给定的交易 ID `txid`,返回对应的比特币区块头信息 `BtcHeaderInfo`.如果找不到对应的区块头,则返回 `None`.

### 总结

`XGatewayBitcoinApi` 提供了一系列与比特币网关相关的功能,使得 ChainX 区块链能够与比特币网络进行交互.
通过这个 API,可以实现比特币的存取款处理,交易验证和区块信息查询等功能.
*/
