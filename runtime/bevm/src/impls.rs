// Copyright 2023 BEVM Project Authors. Licensed under GPL-3.0.

//! Some configurable implementations as associated type for the ChainX runtime.

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{Bounded, DispatchInfoOf, SignedExtension},
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

use bevm_primitives::{AccountId, Balance};

use crate::Runtime;

type NegativeImbalance =
	<pallet_balances::Pallet<Runtime> as Currency<AccountId>>::NegativeImbalance;

type BTCNegativeImbalance =
	<xpallet_btc_ledger::Pallet<Runtime> as Currency<AccountId>>::NegativeImbalance;

pub struct Author;
impl OnUnbalanced<NegativeImbalance> for Author {
	fn on_nonzero_unbalanced(amount: NegativeImbalance) {
		if let Some(author) = <pallet_authorship::Pallet<Runtime>>::author() {
			<pallet_balances::Pallet<Runtime>>::resolve_creating(&author, amount);
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
	/// The portion of the `NORMAL_DISPATCH_RATIO` that we adjust the fees with. Blocks filled less
	/// than this will decrease the weight and more will increase.
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	/// The adjustment variable of the runtime. Higher values will cause `TargetBlockFullness` to
	/// change the fees more rapidly.
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(75, 1000_000);
	/// Minimum amount of the multiplier. This value cannot be too low. A test case should ensure
	/// that combined with `AdjustmentVariable`, we can recover from the minimum.
	/// See `multiplier_can_grow_from_zero`.
	pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 10u128);
	/// The maximum amount of the multiplier.
	pub MaximumMultiplier: Multiplier = Bounded::max_value();
}

pub type SlowAdjustingFeeUpdate<R> = TargetedFeeAdjustment<
	R,
	TargetBlockFullness,
	AdjustmentVariable,
	MinimumMultiplier,
	MaximumMultiplier,
>;
