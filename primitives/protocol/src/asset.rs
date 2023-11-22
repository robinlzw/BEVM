// Copyright 2023 BEVM Project Authors. Licensed under GPL-3.0.

use bevm_primitives::{AssetId, Decimals};

/// Native asset of BEVM chain.
pub const BEVM: AssetId = 0;
/// Decimals of BEVM, the native token of BEVM Chain.
pub const BEVM_DECIMALS: Decimals = 8;

/// BTC asset in BEVM backed by the Mainnet Bitcoin.
pub const BTC: AssetId = 1;
/// Decimals of BTC.
pub const BTC_DECIMALS: Decimals = 8;

