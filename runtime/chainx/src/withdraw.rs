#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use core::marker::PhantomData;
use fp_evm::{
    Context, ExitRevert, ExitSucceed, PrecompileFailure, PrecompileOutput, PrecompileResult,
};
use frame_support::log;
use pallet_evm::{AddressMapping, Precompile};
use sp_core::{hexdisplay::HexDisplay, H160, U256};
use sp_runtime::{traits::UniqueSaturatedInto, AccountId32};
use sp_std::vec;

const MIN_BTC_TRANSFER_VALUE: u128 = 10_000_000_000;
const BASE_GAS_COST: u64 = 100_000;

pub struct Withdraw<
    T: xpallet_assets_bridge::Config
        + xpallet_gateway_common::Config
        + xpallet_gateway_records::Config,
> {
    _marker: PhantomData<T>,
}

impl<
        T: xpallet_assets_bridge::Config
            + xpallet_gateway_common::Config
            + xpallet_gateway_records::Config,
    > Withdraw<T>
{
    fn process(caller: &H160, input: &[u8]) -> Result<(), PrecompileFailure> {
        match input.first() {
            // Withdraw BTC
            Some(&0) if input.len() >= 67 && input.len() <= 95 => {
                // input = (flag, 1 byte) + value(32 bytes) + to(btc address, 34-62 bytes)
                // https://www.doubloin.com/learn/how-long-are-bitcoin-addresses
                log::debug!(target: "evm-withdraw", "btc: call");

                Self::process_withdraw_btc(caller, &input[1..]).map_err(|err| {
                    log::warn!(target: "evm-withdraw", "btc: err = {:?}", err);
                    err
                })?;

                log::debug!(target: "evm-withdraw", "btc: success");

                Ok(())
            }
            // Withdraw PCX
            Some(&1) if input.len() == 65 => {
                // input = (flag, 1 byte) + value(32 bytes) + to(substrate pubkey, 32 bytes)

                log::debug!(target: "evm-withdraw", "pcx: call");

                Self::process_withdraw_pcx(caller, &input[1..]).map_err(|err| {
                    log::warn!(target: "evm-withdraw", "pcx: err = {:?}", err);
                    err
                })?;

                log::debug!(target: "evm-withdraw", "pcx: success");

                Ok(())
            }
            _ => {
                log::warn!(target: "evm-withdraw", "invalid input: {:?}", input);

                Err(PrecompileFailure::Revert {
                    exit_status: ExitRevert::Reverted,
                    output: "invalid withdraw(0x403) input".into(),
                    cost: BASE_GAS_COST,
                })
            }
        }
    }

    fn account_from_pubkey(pubkey: &[u8]) -> Result<T::AccountId, PrecompileFailure> {
        frame_support::ensure!(
            pubkey.len() == 32,
            PrecompileFailure::Revert {
                exit_status: ExitRevert::Reverted,
                output: "invalid chainx pubkey".into(),
                cost: BASE_GAS_COST
            }
        );

        let mut target = [0u8; 32];
        target[0..32].copy_from_slice(&pubkey[0..32]);

        T::AccountId::decode(&mut &AccountId32::new(target).encode()[..]).map_err(|_| {
            PrecompileFailure::Revert {
                exit_status: ExitRevert::Reverted,
                output: "decode AccountId32 failed".into(),
                cost: BASE_GAS_COST,
            }
        })
    }

    fn balance(value: &[u8], is_btc: bool) -> Result<u128, PrecompileFailure> {
        frame_support::ensure!(
            value.len() == 32,
            PrecompileFailure::Revert {
                exit_status: ExitRevert::Reverted,
                output: "invalid balance".into(),
                cost: BASE_GAS_COST
            }
        );

        let mut balance = U256::from_big_endian(&value[0..32]).low_u128();

        if balance == 0 {
            return Err(PrecompileFailure::Revert {
                exit_status: ExitRevert::Reverted,
                output: "zero balance".into(),
                cost: BASE_GAS_COST,
            });
        }

        if is_btc {
            // evm balance decimals=18, wasm balance decimals=8
            if balance < MIN_BTC_TRANSFER_VALUE {
                return Err(PrecompileFailure::Revert {
                    exit_status: ExitRevert::Reverted,
                    output: "balance < 10 Gwei".into(),
                    cost: BASE_GAS_COST,
                });
            }

            balance = balance
                .checked_div(MIN_BTC_TRANSFER_VALUE)
                .unwrap_or(u128::MAX);
        }

        Ok(balance)
    }

    fn process_withdraw_pcx(caller: &H160, input: &[u8]) -> Result<(), PrecompileFailure> {
        let balance = Self::balance(&input[0..32], false)?;
        let to = Self::account_from_pubkey(&input[32..64])?;

        log::debug!(target: "evm-withdraw", "from(evm): {:?}", caller);
        log::debug!(target: "evm-withdraw", "to(pcx): {:?}", HexDisplay::from(&to.encode()));
        log::debug!(target: "evm-withdraw", "value(sub): {:?}", balance);

        xpallet_assets_bridge::Pallet::<T>::withdraw_pcx_from_evm(*caller, to, balance).map_err(
            |err| {
                log::debug!(target: "evm-withdraw", "withdraw_pcx: {:?}", err);

                PrecompileFailure::Revert {
                    exit_status: ExitRevert::Reverted,
                    output: "withdraw pcx failed".into(),
                    cost: BASE_GAS_COST,
                }
            },
        )
    }

    fn process_withdraw_btc(caller: &H160, input: &[u8]) -> Result<(), PrecompileFailure> {
        let from = T::AddressMapping::into_account_id(*caller);
        let balance = Self::balance(&input[0..32], true)?;
        let btc_addr = &input[32..];

        log::debug!(target: "evm-withdraw", "from(evm): {:?}", caller);
        log::debug!(target: "evm-withdraw", "to(btc): {:?}", btc_addr);
        log::debug!(target: "evm-withdraw", "value(sub): {:?}", balance);

        xpallet_assets_bridge::Pallet::<T>::swap_btc_to_xbtc(*caller, balance).map_err(|err| {
            log::debug!(target: "evm-withdraw", "btc_to_xbtc: {:?}", err);

            PrecompileFailure::Revert {
                exit_status: ExitRevert::Reverted,
                output: "swap btc failed".into(),
                cost: BASE_GAS_COST,
            }
        })?;

        xpallet_gateway_common::Pallet::<T>::verify_withdrawal(
            1,
            balance.unique_saturated_into(),
            btc_addr,
            &Default::default(),
        )
        .map_err(|err| {
            log::debug!(target: "evm-withdraw", "verify_withdrawal: {:?}", err);

            PrecompileFailure::Revert {
                exit_status: ExitRevert::Reverted,
                output: "verify withdrawal failed".into(),
                cost: BASE_GAS_COST,
            }
        })?;

        xpallet_gateway_records::Pallet::<T>::withdraw(
            &from,
            1,
            balance.unique_saturated_into(),
            btc_addr.to_vec(),
            Default::default(),
        )
        .map_err(|err| {
            log::debug!(target: "evm-withdraw", "xbtc withdraw: {:?}", err);

            PrecompileFailure::Revert {
                exit_status: ExitRevert::Reverted,
                output: "xbtc withdraw failed".into(),
                cost: BASE_GAS_COST,
            }
        })?;

        Ok(())
    }
}

impl<T> Precompile for Withdraw<T>
where
    T: xpallet_assets_bridge::Config
        + xpallet_gateway_common::Config
        + xpallet_gateway_records::Config,
    T::AccountId: Decode,
{
    fn execute(
        input: &[u8],
        _target_gas: Option<u64>,
        context: &Context,
        _: bool,
    ) -> PrecompileResult {
        log::debug!(target: "evm-withdraw", "caller: {:?}", context.caller);

        Self::process(&context.caller, input).map(|_| {
            // Refer: https://github.com/rust-ethereum/ethabi/blob/master/ethabi/src/encoder.rs#L144
            let mut out = vec![0u8; 32];
            out[31] = 1u8;

            Ok(PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                cost: BASE_GAS_COST,
                output: out.to_vec(),
                logs: Default::default(),
            })
        })?
    }
}

/*
这段代码是一个Rust实现的预编译合约,用于处理ChainX区块链与以太坊虚拟机(EVM)之间的资产转移.
具体来说,它允许从EVM中提取资产(比特币或PCX)到ChainX区块链.以下是代码的主要功能和组件:

### `Withdraw` 结构体
- `Withdraw` 是一个预编译合约,用于处理资产的提取.
- `_marker` 是一个类型标记,使用 `PhantomData` 来表示这个结构体与特定的运行时配置 `T` 相关.

### `process` 方法
- 这个方法是预编译合约的主要逻辑,它根据输入数据来处理资产的提取.
- 输入数据的第一个字节用于指示提取的资产类型(例如,0表示比特币,1表示PCX).
- 输入数据的长度和格式根据资产类型进行验证.

### 比特币提取处理 (`process_withdraw_btc`)
- 从输入数据中提取比特币地址和余额.
- 使用 `xpallet_assets_bridge` 和 `xpallet_gateway_common` 来处理比特币到ChainX的资产(xBTC)的交换.
- 使用 `xpallet_gateway_records` 来记录提款操作.

### PCX提取处理 (`process_withdraw_pcx`)
- 从输入数据中提取PCX的接收者地址和余额.
- 调用 `xpallet_assets_bridge` 来处理从EVM到ChainX的PCX转移.

### `account_from_pubkey` 方法
- 将EVM的公钥转换为ChainX的账户ID.

### `balance` 方法
- 从输入数据中提取余额,并根据资产类型进行处理.

### `Precompile` trait 实现
- `execute` 方法是 `Precompile` trait 的实现,它在EVM调用预编译合约时被调用.
- 方法执行 `process` 函数,并将结果封装为 `PrecompileOutput` 返回.

### 错误处理
- 使用 `PrecompileFailure` 来表示执行过程中可能发生的错误.
- 错误信息被记录,并且会有一个固定的气体成本.

### 日志记录
- 使用 `log` 模块来记录执行过程中的调试信息.

这段代码的设计允许ChainX区块链与EVM兼容,使得用户可以从EVM环境(如以太坊)中提取资产到ChainX区块链.
这对于跨链资产转移和去中心化金融(DeFi)应用是非常重要的.通过这种方式,ChainX可以作为一个桥梁,连接不同的区块链生态系统,促进资产的流动性和互操作性.
*/
