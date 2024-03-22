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
这段代码是ChainX区块链运行时的一部分,它定义了一些可配置的实现,作为ChainX运行时的关联类型.
这些实现包括处理交易费用,特殊调用的额外费用以及交易有效性的逻辑.代码使用了Substrate框架和相关库,如`frame_support`和`pallet`系列模块.

### 主要组件和功能:

1. **类型定义**:
   - `NegativeImbalance`:定义了`Balances`和`XBtcLedger`的负不平衡类型,用于处理货币的减少.
   - `Author`和`DealWithFees`:实现了`OnUnbalanced`特征,用于处理交易费用的分配.`Author`将费用分配给区块作者,而`DealWithFees`将费用分为奖励池和作者.
   - `DealWithBTCFees`:专门用于处理BTC相关费用,将所有费用分配给区块作者或BTC账本的账户.

2. **费用参数**:
   - `TargetBlockFullness`,`AdjustmentVariable`和`MinimumMultiplier`:定义了交易费用调整的参数,包括目标区块饱和度,调整变量和最小乘数.

3. **费用更新**:
   - `SlowAdjustingFeeUpdate`:使用`TargetedFeeAdjustment`结构体来调整交易费用,基于定义的参数.

4. **额外费用逻辑**:
   - `ChargeExtraFee`:定义了一个结构体,用于对某些特殊调用收取额外费用.它提供了检查特定调用是否需要额外费用的方法,并实现了从账户中提取这些费用的逻辑.
   如,设置信托人(`setup_trustee`)需要额外费用,注册,验证和重新绑定质押也需要额外费用.

5. **交易有效性**:
   - `ChargeExtraFee`还实现了`SignedExtension`特征,这允许它作为交易的签名扩展.这包括在交易预处理阶段验证和收取额外费用.
    这意味着它可以在交易执行前检查和收取额外费用.如果账户余额不足以支付这些费用,交易将被视为无效.

    
总的来说,这段代码为ChainX区块链提供了一套完整的费用管理和交易有效性检查机制,确保了区块链的安全性和经济模型的可持续性.

------------------------------------------------------------------------------------------------------------------------   
在区块链和加密货币的上下文中,"负不平衡类型"(Negative Imbalance)通常指的是账户余额的一种状态,其中账户的支出超过了其可用余额.
这种不平衡通常发生在用户试图执行交易,但账户中的资金不足以支付交易费用时.

在Rust编程语言和Substrate框架中,`NegativeImbalance`类型是`frame_support`库中定义的一个概念,
它表示当账户尝试进行超出其余额的支付时,所发生的不平衡量.这种不平衡量是一个可以表示为负数的金额,表示账户欠系统多少资金.

在上述代码中,`NegativeImbalance`类型是与`Currency` trait相关联的,这意味着它是特定于货币系统的.例如:

```rust
type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;
```

这里,`Balances`是一个实现了`Currency` trait的类型,它代表ChainX区块链中的货币系统.`AccountId`是账户的唯一标识符.
`NegativeImbalance`类型允许区块链系统处理和跟踪账户的欠款情况.

在处理交易费用时,如果用户账户中的余额不足以支付交易费用,就会发生负不平衡.区块链系统需要有机制来处理这种情况,例如,它可能会拒绝交易,
冻结账户或允许账户透支(在某些情况下).`NegativeImbalance`类型提供了一种标准化的方式来表示和处理这些情况.



*/


