// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

//! Some configurable implementations as associated type for the ChainX runtime.

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{DispatchInfoOf, SignedExtension},
    transaction_validity::{
        InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransaction,
    },
    FixedPointNumber, Perquintill, RuntimeDebug,
};

use frame_support::{
    parameter_types,
    traits::{Currency, ExistenceRequirement, Imbalance, OnUnbalanced, WithdrawReasons},
};

use pallet_transaction_payment::{Multiplier, TargetedFeeAdjustment};

use xpallet_gateway_common::Call as XGatewayCommonCall;
use xpallet_mining_staking::Call as XStakingCall;

use chainx_primitives::{AccountId, Balance};

use crate::{Authorship, Balances, Call, Runtime, XBtcLedger};

type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;

type BTCNegativeImbalance = <XBtcLedger as Currency<AccountId>>::NegativeImbalance;

pub struct Author;
impl OnUnbalanced<NegativeImbalance> for Author {
    fn on_nonzero_unbalanced(amount: NegativeImbalance) {
        if let Some(author) = Authorship::author() {
            Balances::resolve_creating(&author, amount);
        }
    }
}

pub struct DealWithFees;
impl OnUnbalanced<NegativeImbalance> for DealWithFees {
    fn on_nonzero_unbalanced(fees: NegativeImbalance) {
        // for fees, 90% to the reward pot of author, 10% to author
        let (to_reward_pot, to_author) = fees.ration(90, 10);

        let to_author_numeric_amount = to_author.peek();
        let to_reward_pot_numeric_amount = to_reward_pot.peek();

        if let Some(author) = <pallet_authorship::Pallet<Runtime>>::author() {
            let reward_pot = <xpallet_mining_staking::Pallet<Runtime>>::reward_pot_for(&author);

            <pallet_balances::Pallet<Runtime>>::resolve_creating(&author, to_author);
            <pallet_balances::Pallet<Runtime>>::resolve_creating(&reward_pot, to_reward_pot);
            <frame_system::Pallet<Runtime>>::deposit_event(
                xpallet_transaction_fee::Event::<Runtime>::FeePaid(
                    author,
                    to_author_numeric_amount,
                    reward_pot,
                    to_reward_pot_numeric_amount,
                ),
            );
        }
    }
}

pub struct DealWithBTCFees;
impl OnUnbalanced<BTCNegativeImbalance> for DealWithBTCFees {
    fn on_nonzero_unbalanced(fees: BTCNegativeImbalance) {
        // for btc fees, 100% to the block author

        let fee_amount = fees.peek();

        let beneficiary = if let Some(author) = <pallet_authorship::Pallet<Runtime>>::author() {
            author
        } else {
            <xpallet_btc_ledger::Pallet<Runtime>>::account_id()
        };

        <xpallet_btc_ledger::Pallet<Runtime>>::resolve_creating(&beneficiary, fees);
        <frame_system::Pallet<Runtime>>::deposit_event(
            xpallet_transaction_fee::Event::<Runtime>::BTCFeePaid(beneficiary, fee_amount),
        )
    }
}

parameter_types! {
    pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
    pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(1, 100_000);
    pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000u128);
}

pub type SlowAdjustingFeeUpdate<R> =
    TargetedFeeAdjustment<R, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;

/// A struct for charging additional fee for some special calls.
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ChargeExtraFee;

impl ChargeExtraFee {
    /// Returns the optional extra fee for the given `call`.
    pub fn has_extra_fee(call: &Call) -> Option<Balance> {
        // 1 PCX
        const BASE_EXTRA_FEE: Balance = 100_000_000;

        let extra_cofficient: Option<u32> = match call {
            Call::XGatewayCommon(XGatewayCommonCall::setup_trustee { .. }) => Some(1),
            Call::XStaking(xstaking) => match xstaking {
                XStakingCall::register { .. } => Some(10),
                XStakingCall::validate { .. } => Some(1),
                XStakingCall::rebond { .. } => Some(1),
                _ => None,
            },
            _ => None,
        };

        extra_cofficient.map(|cofficient| Balance::from(cofficient) * BASE_EXTRA_FEE)
    }

    /// Actually withdraws the extra `fee` from account `who`.
    pub fn withdraw_fee(who: &AccountId, fee: Balance) -> TransactionValidity {
        match Balances::withdraw(
            who,
            fee,
            WithdrawReasons::TRANSACTION_PAYMENT,
            ExistenceRequirement::KeepAlive,
        ) {
            Ok(fee) => {
                DealWithFees::on_nonzero_unbalanced(fee);
                Ok(ValidTransaction::default())
            }
            Err(_) => Err(InvalidTransaction::Payment.into()),
        }
    }
}

impl SignedExtension for ChargeExtraFee {
    const IDENTIFIER: &'static str = "ChargeExtraFee";
    type AccountId = AccountId;
    type Call = Call;
    type AdditionalSigned = ();
    type Pre = ();

    fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
        Ok(())
    }

    fn pre_dispatch(
        self,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        self.validate(who, call, info, len).map(|_| ())
    }

    fn validate(
        &self,
        who: &Self::AccountId,
        call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> TransactionValidity {
        if let Some(fee) = ChargeExtraFee::has_extra_fee(call) {
            ChargeExtraFee::withdraw_fee(who, fee)?;
        }

        Ok(ValidTransaction::default())
    }
}

/*
这段代码是使用Rust语言编写的,并且是ChainX项目的一部分.ChainX是一个区块链项目,旨在实现跨链交易和智能合约.
代码主要涉及到区块链的交易费用处理,交易有效性验证以及特殊调用的额外费用收取.下面是对代码的详细解释:

1. **模块和特性导入**:
   - 使用`codec`库来实现数据的编码和解码.
   - 使用`scale_info`库来提供类型信息.
   - 使用`sp_runtime`库,这是Substrate框架的一部分,提供了区块链运行时所需的基本特性.
   - 使用`frame_support`库,这是Substrate框架的一部分,提供了框架层的支持.

2. **自定义类型定义**:
   - `NegativeImbalance`:表示账户余额的负数变化,用于处理交易费用.
   - `BTCNegativeImbalance`:特定于XBtcLedger的负数余额变化.

3. **处理不平衡的账户**:
   - `Author`和`DealWithFees`结构体实现了`OnUnbalanced`特性,用于处理交易费用的分配.
   - `DealWithBTCFees`结构体专门用于处理BTC交易费用,将所有费用分配给区块作者.

4. **交易费用调整**:
   - `parameter_types!`宏定义了交易费用调整的参数,如目标区块满度,调整变量和最小乘数.
   - `SlowAdjustingFeeUpdate`类型用于调整交易费用,以适应网络的拥堵情况.

5. **特殊调用的额外费用**:
   - `ChargeExtraFee`结构体定义了一个额外费用收取机制,用于某些特殊调用.
   - `has_extra_fee`方法检查给定的调用是否需要额外费用,并返回费用金额.
   - `withdraw_fee`方法从指定账户中提取额外费用,并处理不平衡的情况.

6. **签名扩展**:
   - `ChargeExtraFee`还实现了`SignedExtension`特性,这意味着它可以作为交易的一部分,确保交易的签名者支付了额外的费用.

7. **错误处理**:
   - 在尝试从账户中提取额外费用时,如果操作失败,会返回`InvalidTransaction::Payment`错误.

这段代码展示了ChainX项目如何通过自定义的Rust智能合约代码来管理交易费用和特殊调用的额外费用.
*/
