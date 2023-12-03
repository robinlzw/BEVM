// Copyright 2023 BEVM Project Authors. Licensed under GPL-3.0.

use std::convert::TryInto;

use serde::{Deserialize, Serialize};
use serde_json::json;

use sc_chain_spec::ChainSpecExtension;
use sc_service::config::TelemetryEndpoints;
use sc_service::{ChainType, Properties};

use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto::{UncheckedInto, AccountId32}, sr25519, Pair, Public};
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};

use pallet_im_online::sr25519::AuthorityId as ImOnlineId;

use bevm_primitives::{AccountId, AssetId, Balance, Block, ReferralId, Signature};
use bevm_runtime::constants::{currency::DOLLARS, time::DAYS};
use xp_assets_registrar::Chain;
use xp_protocol::NetworkType;
use xpallet_gateway_bitcoin::{BtcParams, BtcTxVerifier};
use xpallet_gateway_common::types::TrusteeInfoConfig;

use bevm_runtime as dev;
use crate::genesis::{
	btc_genesis_params, local_testnet_trustees, BtcGenesisParams, BtcTrusteeParams
};

type AccountPublic = <Signature as Verify>::Signer;

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client_api::ForkBlocks<Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client_api::BadBlocks<Block>,
	/// The light sync state extension used by the sync-state rpc.
	pub light_sync_state: sc_sync_state_rpc::LightSyncStateExtension,
}

/// Specialized `ChainSpec`.
pub type BevmChainSpec = sc_service::GenericChainSpec<dev::RuntimeGenesisConfig, Extensions>;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
	where
		AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

type AuthorityKeysTuple = (
	(AccountId, ReferralId), // (Staking ValidatorId, ReferralId)
	BabeId,
	GrandpaId,
	ImOnlineId,
	AuthorityDiscoveryId,
);

pub fn authority_keys_from_seed(seed: &str) -> AuthorityKeysTuple {
	(
		(
			get_account_id_from_seed::<sr25519::Public>(seed),
			seed.as_bytes().to_vec(),
		),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
	)
}

fn as_properties(network: NetworkType) -> Properties {
	json!({
		"ss58Format": network.ss58_addr_format_id(),
		"network": network,
		"tokenDecimals": 8,
		"tokenSymbol": "BEVM"
    })
	.as_object()
		.expect("network properties generation can not fail; qed")
		.to_owned()
}


/// Development config (single validator Alice).
pub fn development_config() -> BevmChainSpec {
	let constructor = move || {
		build_dev_genesis(
			vec![authority_keys_from_seed("Alice")],
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
            ],
			btc_genesis_params(include_str!("res/btc_genesis_params_testnet.json")),
			local_testnet_trustees(),
		)
	};


	BevmChainSpec::from_genesis(
		"Development",
		"dev",
		ChainType::Development,
		constructor,
		vec![],
		None,
		Some("pcx1"),
		None,
		Some(as_properties(NetworkType::Testnet)),
		Default::default(),
	)
}


fn build_dev_genesis(
	initial_authorities: Vec<AuthorityKeysTuple>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	bitcoin: BtcGenesisParams,
	trustees: Vec<(Chain, TrusteeInfoConfig, Vec<BtcTrusteeParams>)>,
) -> dev::RuntimeGenesisConfig {
	const ENDOWMENT: Balance = 10_000_000 * DOLLARS;
	const STASH: Balance = 100 * DOLLARS;

	let balances = endowed_accounts
		.iter()
		.cloned()
		.map(|k| (k, ENDOWMENT))
		.collect::<Vec<_>>();

	let num_endowed_accounts = endowed_accounts.len();

	// The value of STASH balance will be reserved per phragmen member.
	let phragmen_members = endowed_accounts
		.iter()
		.take((num_endowed_accounts + 1) / 2)
		.cloned()
		.map(|member| (member, STASH))
		.collect();

	let tech_comm_members = endowed_accounts
		.iter()
		.take((num_endowed_accounts + 1) / 2)
		.cloned()
		.collect::<Vec<_>>();

	let btc_genesis_trustees = trustees
		.iter()
		.find_map(|(chain, _, trustee_params)| {
			if *chain == Chain::Bitcoin {
				Some(
					trustee_params
						.iter()
						.map(|i| (i.0).clone())
						.collect::<Vec<_>>(),
				)
			} else {
				None
			}
		})
		.expect("bitcoin trustees generation can not fail; qed");
	dev::RuntimeGenesisConfig {
		sudo: dev::SudoConfig {
			key: Some(root_key),
		},
		system: dev::SystemConfig {
			code: dev::wasm_binary_unwrap().to_vec(),
			..Default::default()
		},
		babe: dev::BabeConfig {
			epoch_config: Some(dev::BABE_GENESIS_EPOCH_CONFIG),
			..Default::default()
		},
		grandpa: Default::default(),
		council: Default::default(),
		technical_committee: Default::default(),
		technical_membership: dev::TechnicalMembershipConfig {
			members: tech_comm_members.try_into().unwrap(),
			phantom: Default::default(),
		},
		democracy: Default::default(),
		treasury: Default::default(),
		elections: dev::ElectionsConfig {
			members: phragmen_members,
		},
		im_online: Default::default(),
		authority_discovery: Default::default(),
		session: dev::SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						(x.0).0.clone(),
						(x.0).0.clone(),
						dev::SessionKeys {
							babe: x.1.clone(),
							grandpa: x.2.clone(),
							im_online: x.3.clone(),
							authority_discovery: x.4.clone(),
						},
					)
				})
				.collect::<Vec<_>>(),
		},
		balances: dev::BalancesConfig { balances },
		indices: Default::default(),
		x_gateway_common: dev::XGatewayCommonConfig { trustees },
		x_gateway_bitcoin: dev::XGatewayBitcoinConfig {
			genesis_trustees: btc_genesis_trustees,
			network_id: bitcoin.network,
			confirmation_number: bitcoin.confirmation_number,
			genesis_hash: bitcoin.hash(),
			genesis_info: (bitcoin.header(), bitcoin.height),
			params_info: BtcParams::new(
				// for signet and regtest
				545259519,            // max_bits
				2 * 60 * 60,          // block_max_future
				2 * 7 * 24 * 60 * 60, // target_timespan_seconds
				10 * 60,              // target_spacing_seconds
				4,                    // retargeting_factor
			), // retargeting_factor
			btc_withdrawal_fee: 500000,
			max_withdrawal_count: 100,
			verifier: BtcTxVerifier::Recover,
		},
		x_staking: dev::XStakingConfig {
			validator_count: 40,
			sessions_per_era: 12,
			glob_dist_ratio: (12, 88), // (Treasury, X-type Asset and Staking) = (12, 88)
			mining_ratio: (10, 90),    // (Asset Mining, Staking) = (10, 90)
			minimum_penalty: 100 * DOLLARS,
			candidate_requirement: (100 * DOLLARS, 1_000 * DOLLARS), // Minimum value (self_bonded, total_bonded) to be a validator candidate
			..Default::default()
		},
		ethereum_chain_id: dev::EthereumChainIdConfig {
			chain_id: 11503u64,
			..Default::default()
		},
		x_btc_ledger: Default::default(),
	}
}