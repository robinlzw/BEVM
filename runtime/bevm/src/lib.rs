// Copyright 2023 BEVM Project Authors. Licensed under GPL-3.0.

//! The Substrate Node Template runtime. This can be compiled with `#[no_std]`, ready for Wasm.
#![allow(clippy::unnecessary_cast)]
#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use static_assertions::const_assert;

use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
#[cfg(feature = "runtime-benchmarks")]
use sp_runtime::RuntimeString;
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{
		self, AccountIdConversion, BlakeTwo256, Block as BlockT, Convert, DispatchInfoOf,
		NumberFor, OpaqueKeys, SaturatedConversion, Saturating, SignedExtension, StaticLookup,
	},
	transaction_validity::{
		InvalidTransaction, TransactionPriority, TransactionSource, TransactionValidity,
		TransactionValidityError, ValidTransaction,
	},
	ApplyExtrinsicResult, DispatchError, Perbill, Percent, Permill, RuntimeDebug,
};
use sp_std::{collections::btree_map::BTreeMap, prelude::*};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use frame_system::{EnsureRoot, EnsureSigned, EnsureWithSuccess};
use pallet_grandpa::{
	AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
};
use pallet_identity::simple::IdentityInfo;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_session::historical as pallet_session_historical;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryPlainSlots;

use runtime_common::{RuntimeBlockLength, RuntimeBlockWeights, BASE_FEE};
use xp_assets_registrar::{Chain, WithdrawalLimit};
use xpallet_mining_staking::{NominatorInfo, NominatorLedger, ValidatorInfo};
use xpallet_support::traits::MultisigAddressFor;

// A few exports that help ease life for downstream crates.
pub use frame_support::{
	construct_runtime, debug, parameter_types,
	traits::{
		fungible::HoldConsideration,
		tokens::{PayFromAccount, UnityAssetBalanceConversion},
		ConstBool, ConstU32, ConstU64, Contains, Currency, EitherOfDiverse, EqualPrivilegeOnly,
		Get, Imbalance, InstanceFilter, KeyOwnerProofSystem, LinearStoragePrice, LockIdentifier,
		OnRuntimeUpgrade, OnUnbalanced, Randomness,
	},
	weights::{
		constants::{
			BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND,
		},
		ConstantMultiplier, Weight,
	},
	PalletId, StorageValue,
};

pub use bevm_primitives::{
	AccountId, AccountIndex, AddrStr, Amount, AssetId, Balance, BlockNumber, ChainAddress, Hash,
	Moment, Nonce, ReferralId, Signature, Token,
};
pub use sp_staking::SessionIndex;
pub use xp_protocol::*;
pub use xp_runtime::Memo;

use sp_core::{H160, U256};

#[cfg(feature = "std")]
pub use xpallet_gateway_bitcoin::h256_rev;
pub use xpallet_gateway_bitcoin::{
	hash_rev, types::BtcHeaderInfo, BtcHeader, BtcNetwork, BtcParams, BtcTxVerifier,
	BtcWithdrawalProposal, Compact, H256,
};
pub use xpallet_gateway_common::{
	trustees,
	types::{
		GenericTrusteeIntentionProps, GenericTrusteeSessionInfo, ScriptInfo, TrusteeInfoConfig,
	},
};
pub use xpallet_gateway_records::{Withdrawal, WithdrawalRecordId};
pub use xpallet_mining_staking::VoteWeight;

/// Constant values used within the runtime.
pub mod constants;
/// Implementations of some helper traits passed into runtime modules as associated types.
pub mod impls;

use self::{
	constants::{currency::*, time::*},
	impls::{DealWithBTCFees, DealWithFees, SlowAdjustingFeeUpdate},
};

/// Wasm binary unwrapped. If built with `SKIP_WASM_BUILD`, the function panics.
#[cfg(feature = "std")]
pub fn wasm_binary_unwrap() -> &'static [u8] {
	WASM_BINARY.expect(
		"Wasm binary is not available. This means the client is built with \
		 `SKIP_WASM_BUILD` flag and it is only usable for production chains. Please rebuild with \
		 the flag disabled.",
	)
}

/// This runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("bevm"),
	impl_name: create_runtime_str!("bevm"),
	authoring_version: 1,
	spec_version: 1,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

/// The BABE epoch configuration at genesis.
/// The existing chain is running with PrimaryAndSecondaryPlainSlots,
/// you should keep returning the same thing in BabeApi::configuration()
/// as you were doing before.
///
/// Edit: it's okay to change this here as BabeApi::configuration()
/// is only used on genesis, so this change won't have any effect on
/// the existing chains. But maybe it makes it more clear if you still
/// keep the original value.
pub const BABE_GENESIS_EPOCH_CONFIG: sp_consensus_babe::BabeEpochConfiguration =
	sp_consensus_babe::BabeEpochConfiguration {
		c: PRIMARY_PROBABILITY,
		allowed_slots: PrimaryAndSecondaryPlainSlots,
	};

#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct BaseFilter;
impl Contains<RuntimeCall> for BaseFilter {
	fn contains(call: &RuntimeCall) -> bool {
		use frame_support::traits::GetCallMetadata;

		let metadata = call.get_call_metadata();
		!XSystem::is_paused(metadata)
	}
}

pub const FORBIDDEN_CALL: u8 = 255;
pub const FORBIDDEN_ACCOUNT: u8 = 254;

impl SignedExtension for BaseFilter {
	const IDENTIFIER: &'static str = "BaseFilter";
	type AccountId = AccountId;
	type Call = RuntimeCall;
	type AdditionalSigned = ();
	type Pre = ();
	fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
		Ok(())
	}

	fn validate(
		&self,
		who: &Self::AccountId,
		call: &Self::Call,
		_info: &DispatchInfoOf<Self::Call>,
		_len: usize,
	) -> TransactionValidity {
		if !Self::contains(call) {
			return Err(InvalidTransaction::Custom(FORBIDDEN_CALL).into())
		}
		if XSystem::blacklist(who) {
			return Err(InvalidTransaction::Custom(FORBIDDEN_ACCOUNT).into())
		}
		Ok(ValidTransaction::default())
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
}

const AVERAGE_ON_INITIALIZE_WEIGHT: Perbill = Perbill::from_percent(10);
parameter_types! {
	pub const BlockHashCount: BlockNumber = 2400;
	/// We allow for 2 seconds of compute with a 6 second average block time.
	pub const MaximumBlockWeight: Weight = Weight::from_parts(2 * WEIGHT_REF_TIME_PER_SECOND, 0);
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	/// Assume 10% of weight for average on_initialize calls.
	pub MaximumExtrinsicWeight: Weight =
		AvailableBlockRatio::get().saturating_sub(AVERAGE_ON_INITIALIZE_WEIGHT)
		* MaximumBlockWeight::get();
	pub const MaximumBlockLength: u32 = 5 * 1024 * 1024;
	pub const Version: RuntimeVersion = VERSION;
	pub const SS58Prefix: u8 = xp_protocol::MAINNET_ADDRESS_FORMAT_ID;
	pub MaxCollectivesProposalWeight: Weight =
		Perbill::from_percent(50) * RuntimeBlockWeights::get().max_block;
}

const_assert!(
	AvailableBlockRatio::get().deconstruct() >= AVERAGE_ON_INITIALIZE_WEIGHT.deconstruct()
);
impl frame_system::Config for Runtime {
	type BaseCallFilter = BaseFilter;
	type BlockWeights = RuntimeBlockWeights;
	type BlockLength = RuntimeBlockLength;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// The ubiquitous origin type.
	type RuntimeOrigin = RuntimeOrigin;
	/// The aggregated dispatch type that is available for extrinsics.
	type RuntimeCall = RuntimeCall;
	/// The index type for storing how many extrinsics an account has signed.
	type Nonce = Nonce;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = Indices;
	/// The header type.
	type Block = Block;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// Version of the runtime.
	type Version = Version;
	/// Converts a module to the index of the module in `construct_runtime!`.
	///
	/// This type is being generated by `construct_runtime!`.
	type PalletInfo = PalletInfo;
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = frame_system::weights::SubstrateWeight<Runtime>;
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Babe;
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = pallet_timestamp::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const IndexDeposit: Balance = 10 * DOLLARS;
}

impl pallet_indices::Config for Runtime {
	type AccountIndex = AccountIndex;
	type Currency = Balances;
	type Deposit = IndexDeposit;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_indices::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const MaxAuthorities: u32 = 10_000;
}
impl pallet_authority_discovery::Config for Runtime {
	type MaxAuthorities = MaxAuthorities;
}

parameter_types! {
	pub const UncleGenerations: BlockNumber = 0;
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
	type EventHandler = ImOnline;
}

impl pallet_preimage::Config for Runtime {
	type WeightInfo = pallet_preimage::weights::SubstrateWeight<Runtime>;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<AccountId>;
	type Consideration = HoldConsideration<
		AccountId,
		Balances,
		PreimageHoldReason,
		LinearStoragePrice<PreimageBaseDeposit, PreimageByteDeposit, Balance>,
	>;
}

parameter_types! {
	pub const EpochDuration: u64 = EPOCH_DURATION_IN_BLOCKS as u64;
	pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
	pub const MaxNominatorRewardedPerValidator: u32 = 256;
}

pub struct ReportLongevity;

impl Get<u64> for ReportLongevity {
	fn get() -> u64 {
		xpallet_mining_staking::BondingDuration::<Runtime>::get() as u64 *
			xpallet_mining_staking::SessionsPerEra::<Runtime>::get() as u64 *
			EpochDuration::get()
	}
}

impl pallet_babe::Config for Runtime {
	type EpochDuration = EpochDuration;
	type ExpectedBlockTime = ExpectedBlockTime;
	type EpochChangeTrigger = pallet_babe::ExternalTrigger;
	type DisabledValidators = Session;
	type WeightInfo = ();
	type MaxAuthorities = MaxAuthorities;
	type MaxNominators = MaxNominatorRewardedPerValidator;
	type KeyOwnerProof =
		<Historical as KeyOwnerProofSystem<(KeyTypeId, pallet_babe::AuthorityId)>>::Proof;
	type EquivocationReportSystem =
		pallet_babe::EquivocationReportSystem<Self, Offences, Historical, ReportLongevity>;
}

impl pallet_grandpa::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type MaxAuthorities = MaxAuthorities;
	type MaxNominators = MaxNominatorRewardedPerValidator;
	type MaxSetIdSessionEntries = ConstU64<0>;
	type KeyOwnerProof = <Historical as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;
	type EquivocationReportSystem =
		pallet_grandpa::EquivocationReportSystem<Self, Offences, Historical, ReportLongevity>;
}

parameter_types! {
	pub const Offset: BlockNumber = 0;
	pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
}

impl_opaque_keys! {
	pub struct SessionKeys {
		pub babe: Babe,
		pub grandpa: Grandpa,
		pub im_online: ImOnline,
		pub authority_discovery: AuthorityDiscovery,
	}
}

/// Substrate has the controller/stash concept, the according `Convert`
/// implementation is used to find the stash of the given controller
/// account. There is no such concept in the context of ChainX, the
/// _stash_ account is also the _controller_ account.
pub struct SimpleValidatorIdConverter;

impl Convert<AccountId, Option<AccountId>> for SimpleValidatorIdConverter {
	fn convert(controller: AccountId) -> Option<AccountId> {
		Some(controller)
	}
}

impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = SimpleValidatorIdConverter;
	type ShouldEndSession = Babe;
	type NextSessionRotation = Babe;
	// We do not make use of the historical feature of pallet-session, hereby use XStaking only.
	type SessionManager = XStaking;
	type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type WeightInfo = pallet_session::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	/// No dusty accounts in ChainX.
	pub const ExistentialDeposit: Balance = 0;
	// For weight estimation, we assume that the most locks on an individual account will be 50.
	// This number may need to be adjusted in the future if this assumption no longer holds true.
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type FreezeIdentifier = RuntimeFreezeReason;
	type MaxHolds = ConstU32<5>;
	type MaxFreezes = ConstU32<5>;
}

parameter_types! {
	pub const TransactionByteFee: Balance = 10 * MILLICENTS; // 100 => 0.000001 pcx
	pub const OperationalFeeMultiplier: u8 = 5;
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, DealWithFees>;
	type OperationalFeeMultiplier = OperationalFeeMultiplier;
	type WeightToFee = constants::fee::WeightToFee;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
}

impl xpallet_transaction_fee::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
}

parameter_types! {
	pub const SessionDuration: BlockNumber = EPOCH_DURATION_IN_BLOCKS;
	pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::MAX;
	/// We prioritize im-online heartbeats over election solution submission.
	pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::MAX / 2;
	pub const MaxKeys: u32 = 10_000;
	pub const MaxPeerInHeartbeats: u32 = 10_000;
	pub const MaxPeerDataEncodingSize: u32 = 1_000;
}

impl pallet_im_online::Config for Runtime {
	type AuthorityId = ImOnlineId;
	type RuntimeEvent = RuntimeEvent;
	type ValidatorSet = Historical;
	type NextSessionRotation = Babe;
	type ReportUnresponsiveness = Offences;
	type UnsignedPriority = ImOnlineUnsignedPriority;
	type WeightInfo = pallet_im_online::weights::SubstrateWeight<Runtime>;
	type MaxKeys = MaxKeys;
	type MaxPeerInHeartbeats = MaxPeerInHeartbeats;
}

impl frame_support::traits::ValidatorSet<AccountId> for Runtime {
	type ValidatorId = AccountId;
	type ValidatorIdOf = SimpleValidatorIdConverter;

	fn session_index() -> SessionIndex {
		Session::current_index()
	}

	fn validators() -> Vec<Self::ValidatorId> {
		Session::validators()
	}
}

impl frame_support::traits::ValidatorSetWithIdentification<AccountId> for Runtime {
	type Identification = AccountId;
	type IdentificationOf = SimpleValidatorIdConverter;
}

/// Dummy implementation for the trait bound of pallet_im_online.
/// We actually make no use of the historical feature of pallet_session.
impl pallet_session_historical::Config for Runtime {
	type FullIdentification = AccountId;
	/// Substrate: given the stash account ID, find the active exposure of nominators on that
	/// account. ChainX: the full identity is always the validator account itself.
	type FullIdentificationOf = SimpleValidatorIdConverter;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
	RuntimeCall: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: RuntimeCall,
		public: <Signature as traits::Verify>::Signer,
		account: AccountId,
		nonce: Nonce,
	) -> Option<(RuntimeCall, <UncheckedExtrinsic as traits::Extrinsic>::SignaturePayload)> {
		let tip = 0;
		// take the biggest period possible.
		let period =
			BlockHashCount::get().checked_next_power_of_two().map(|c| c / 2).unwrap_or(2) as u64;
		let current_block = System::block_number()
			.saturated_into::<u64>()
			// The `System::block_number` is initialized with `n+1`,
			// so the actual block number is `n`.
			.saturating_sub(1);
		let extra: SignedExtra = (
			frame_system::CheckNonZeroSender::<Runtime>::new(),
			frame_system::CheckSpecVersion::<Runtime>::new(),
			frame_system::CheckTxVersion::<Runtime>::new(),
			frame_system::CheckGenesis::<Runtime>::new(),
			frame_system::CheckEra::<Runtime>::from(generic::Era::mortal(period, current_block)),
			frame_system::CheckNonce::<Runtime>::from(nonce),
			frame_system::CheckWeight::<Runtime>::new(),
			pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
			BaseFilter,
		);
		let raw_payload = SignedPayload::new(call, extra)
			.map_err(|e| {
				log::warn!("Unable to create signed payload: {:?}", e);
			})
			.ok()?;
		let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
		let address = Indices::unlookup(account);
		let (call, extra, _) = raw_payload.deconstruct();
		Some((call, (address, signature, extra)))
	}
}

impl frame_system::offchain::SigningTypes for Runtime {
	type Public = <Signature as traits::Verify>::Signer;
	type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
	RuntimeCall: From<C>,
{
	type Extrinsic = UncheckedExtrinsic;
	type OverarchingCall = RuntimeCall;
}

impl pallet_offences::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type IdentificationTuple = xpallet_mining_staking::IdentificationTuple<Runtime>;
	type OnOffenceHandler = XStaking;
}

impl pallet_utility::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	// One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
	pub const DepositBase: Balance = deposit(1, 88);
	// Additional storage item size of 32 bytes.
	pub const DepositFactor: Balance = deposit(0, 32);
	pub const MaxSignatories: u16 = 100;
}

impl pallet_multisig::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = MaxSignatories;
	type WeightInfo = pallet_multisig::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const LaunchPeriod: BlockNumber = 7 * MINUTES;
	pub const VotingPeriod: BlockNumber = 7 * MINUTES;
	pub const FastTrackVotingPeriod: BlockNumber = 3 * MINUTES;
	pub const InstantAllowed: bool = true;
	// 10 PCX
	pub const MinimumDeposit: Balance = 1000 * DOLLARS;
	pub const EnactmentPeriod: BlockNumber = 8 * MINUTES;
	pub const CooloffPeriod: BlockNumber = 7 * MINUTES;

	pub const PreimageBaseDeposit: Balance = 1000 * DOLLARS;
	// One cent: $10,000 / MB
	pub const PreimageByteDeposit: Balance = 1 * CENTS;
	pub const PreimageHoldReason: RuntimeHoldReason = RuntimeHoldReason::Preimage(pallet_preimage::HoldReason::Preimage);

	pub const MaxVotes: u32 = 100;
	pub const MaxProposals: u32 = 100;
	pub const MaxDeposits: u32  = 100;
	pub const MaxBlacklisted: u32 = 100;
}

impl pallet_democracy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
	type VoteLockingPeriod = EnactmentPeriod;
	type MinimumDeposit = MinimumDeposit;
	/// A straight majority of the council can decide what their next motion is.
	type ExternalOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>;
	/// A super-majority can have the next scheduled referendum be a straight majority-carries vote.
	type ExternalMajorityOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 4>;
	/// A unanimous council can have the next scheduled referendum be a straight default-carries
	/// (NTB) vote.
	type ExternalDefaultOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>;
	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
	/// be tabled immediately and with a shorter voting/enactment period.
	type SubmitOrigin = EnsureSigned<AccountId>;
	type FastTrackOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 2, 3>;
	type InstantOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 1, 1>;
	type InstantAllowed = InstantAllowed;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
	type CancellationOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>;
	// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	// Root must agree.
	type CancelProposalOrigin = EitherOfDiverse<
		pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 1, 1>,
		EnsureRoot<AccountId>,
	>;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cooloff period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
	type CooloffPeriod = CooloffPeriod;
	type Slash = Treasury;
	type Scheduler = Scheduler;
	type PalletsOrigin = OriginCaller;
	type MaxVotes = MaxVotes;
	type WeightInfo = pallet_democracy::weights::SubstrateWeight<Runtime>;
	type MaxProposals = MaxProposals;
	type Preimages = Preimage;
	type MaxBlacklisted = MaxBlacklisted;
	type MaxDeposits = MaxDeposits;
}

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 7 * MINUTES;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
	type SetMembersOrigin = EnsureRoot<Self::AccountId>;
	type MaxProposalWeight = MaxCollectivesProposalWeight;
}

parameter_types! {
	// 10 PCX
	pub const CandidacyBond: Balance = 1000 * DOLLARS;
	// 1 storage item created, key size is 32 bytes, value size is 16+16.
	pub const VotingBondBase: Balance = deposit(1, 64);
	// additional data per vote is 32 bytes (account id).
	pub const VotingBondFactor: Balance = deposit(0, 32);
	pub const VotingBond: Balance = DOLLARS;
	pub const TermDuration: BlockNumber = 5 * MINUTES;
	pub const DesiredMembers: u32 = 11;
	pub const DesiredRunnersUp: u32 = 7;
	pub const MaxVotesPerVoter: u32 = 16;
	pub const MaxVoters: u32 = 512;
	pub const MaxCandidates: u32 = 64;
	pub const ElectionsPhragmenPalletId: LockIdentifier = *b"pcx/phre";
}

// Make sure that there are no more than `MaxMembers` members elected via elections-phragmen.
const_assert!(DesiredMembers::get() <= CouncilMaxMembers::get());

impl pallet_elections_phragmen::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type PalletId = ElectionsPhragmenPalletId;
	type Currency = Balances;
	type ChangeMembers = Council;
	// NOTE: this implies that council's genesis members cannot be set directly and must come from
	// this module.
	type InitializeMembers = Council;
	type CurrencyToVote = sp_staking::currency_to_vote::U128CurrencyToVote;
	type CandidacyBond = CandidacyBond;
	type VotingBondBase = VotingBondBase;
	type VotingBondFactor = VotingBondFactor;
	type LoserCandidate = Treasury;
	type KickedMember = Treasury;
	type DesiredMembers = DesiredMembers;
	type DesiredRunnersUp = DesiredRunnersUp;
	type TermDuration = TermDuration;
	type MaxVoters = MaxVoters;
	type MaxVotesPerVoter = MaxVotesPerVoter;
	type MaxCandidates = MaxCandidates;
	type WeightInfo = pallet_elections_phragmen::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const TechnicalMotionDuration: BlockNumber = 5 * MINUTES;
	pub const TechnicalMaxProposals: u32 = 100;
	pub const TechnicalMaxMembers: u32 = 100;
}

type TechnicalCollective = pallet_collective::Instance2;
impl pallet_collective::Config<TechnicalCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = TechnicalMotionDuration;
	type MaxProposals = TechnicalMaxProposals;
	type MaxMembers = TechnicalMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
	type SetMembersOrigin = EnsureRoot<Self::AccountId>;
	type MaxProposalWeight = MaxCollectivesProposalWeight;
}

type EnsureRootOrHalfCouncil = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
>;
impl pallet_membership::Config<pallet_membership::Instance1> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AddOrigin = EnsureRootOrHalfCouncil;
	type RemoveOrigin = EnsureRootOrHalfCouncil;
	type SwapOrigin = EnsureRootOrHalfCouncil;
	type ResetOrigin = EnsureRootOrHalfCouncil;
	type PrimeOrigin = EnsureRootOrHalfCouncil;
	type MembershipInitialized = TechnicalCommittee;
	type MembershipChanged = TechnicalCommittee;
	type MaxMembers = TechnicalMaxMembers;
	type WeightInfo = pallet_membership::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	// 10 PCX
	pub const ProposalBondMinimum: Balance = 1000 * DOLLARS;
	// 100 PCX
	pub const ProposalBondMaximum: Balance = 10000 * DOLLARS;
	pub const SpendPeriod: BlockNumber = 6 * MINUTES;
	pub const NoBurn: Permill = Permill::from_percent(0);
	pub const TipCountdown: BlockNumber = 3 * MINUTES;
	pub const TipFindersFee: Percent = Percent::from_percent(20);
	pub const TipReportDepositBase: Balance = DOLLARS;
	pub const DataDepositPerByte: Balance = CENTS;
	pub const BountyDepositBase: Balance = DOLLARS;
	pub const BountyDepositPayoutDelay: BlockNumber = 4 * MINUTES;
	pub const TreasuryPalletId: PalletId = PalletId(*b"bevmtrsy");
	pub const BountyUpdatePeriod: BlockNumber = 90 * MINUTES;
	pub const MaximumReasonLength: u32 = 16384;
	pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
	pub const BountyValueMinimum: Balance = 10 * DOLLARS;
	pub const MaxApprovals: u32 = 100;
	pub const MaxBalance: Balance = Balance::max_value();
	pub const CuratorDepositMultiplier: Permill = Permill::from_percent(50);
	pub const CuratorDepositMin: Balance = 1 * DOLLARS;
	pub const CuratorDepositMax: Balance = 100 * DOLLARS;
	pub const SpendPayoutPeriod: BlockNumber = 30 * DAYS;
	pub TreasuryAccount: AccountId = Treasury::account_id();
	pub const MaxTipAmount: u128 = 500 * DOLLARS;
}

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type ApproveOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 5>,
	>;
	type RejectOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>,
	>;
	type RuntimeEvent = RuntimeEvent;
	type OnSlash = Treasury;
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type ProposalBondMaximum = ProposalBondMaximum;
	type SpendPeriod = SpendPeriod;
	type Burn = NoBurn;
	type BurnDestination = ();
	type SpendFunds = Bounties;
	type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
	type MaxApprovals = MaxApprovals;
	type SpendOrigin = EnsureWithSuccess<EnsureRoot<AccountId>, AccountId, MaxBalance>;
	type AssetKind = ();
	type Beneficiary = AccountId;
	type BeneficiaryLookup = Indices;
	type Paymaster = PayFromAccount<Balances, TreasuryAccount>;
	type BalanceConverter = UnityAssetBalanceConversion;
	type PayoutPeriod = SpendPayoutPeriod;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

impl pallet_bounties::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type BountyDepositBase = BountyDepositBase;
	type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
	type BountyUpdatePeriod = BountyUpdatePeriod;
	type CuratorDepositMultiplier = CuratorDepositMultiplier;
	type CuratorDepositMin = CuratorDepositMin;
	type CuratorDepositMax = CuratorDepositMax;
	type BountyValueMinimum = BountyValueMinimum;
	type DataDepositPerByte = DataDepositPerByte;
	type MaximumReasonLength = MaximumReasonLength;
	type WeightInfo = pallet_bounties::weights::SubstrateWeight<Runtime>;
	type ChildBountyManager = ();
}

impl pallet_tips::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type DataDepositPerByte = DataDepositPerByte;
	type MaximumReasonLength = MaximumReasonLength;
	type Tippers = Elections;
	type TipCountdown = TipCountdown;
	type TipFindersFee = TipFindersFee;
	type TipReportDepositBase = TipReportDepositBase;
	type MaxTipAmount = MaxTipAmount;
	type WeightInfo = pallet_tips::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * MaximumBlockWeight::get();
	// Retry a scheduled item every 10 blocks (1 minute) until the preimage exists.
	pub const NoPreimagePostponement: Option<u32> = Some(10);
}

impl pallet_scheduler::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	#[cfg(feature = "runtime-benchmarks")]
	type MaxScheduledPerBlock = ConstU32<512>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type MaxScheduledPerBlock = ConstU32<50>;
	type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type Preimages = Preimage;
}

parameter_types! {
	pub const BasicDeposit: Balance = 10 * DOLLARS;       // 258 bytes on-chain
	pub const FieldDeposit: Balance = 250 * CENTS;        // 66 bytes on-chain
	pub const SubAccountDeposit: Balance = 2 * DOLLARS;   // 53 bytes on-chain
	pub const MaxSubAccounts: u32 = 100;
	pub const MaxAdditionalFields: u32 = 100;
	pub const MaxRegistrars: u32 = 20;
}

impl pallet_identity::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BasicDeposit = BasicDeposit;
	type FieldDeposit = FieldDeposit;
	type SubAccountDeposit = SubAccountDeposit;
	type MaxSubAccounts = MaxSubAccounts;
	type MaxAdditionalFields = MaxAdditionalFields;
	type IdentityInformation = IdentityInfo<MaxAdditionalFields>;
	type MaxRegistrars = MaxRegistrars;
	type Slashed = Treasury;
	type ForceOrigin = EnsureRootOrHalfCouncil;
	type RegistrarOrigin = EnsureRootOrHalfCouncil;
	type WeightInfo = pallet_identity::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	// One storage item; key size 32, value size 8; .
	pub const ProxyDepositBase: Balance = deposit(1, 8);
	// Additional storage item size of 33 bytes.
	pub const ProxyDepositFactor: Balance = deposit(0, 33);
	pub const MaxProxies: u16 = 32;
	pub const AnnouncementDepositBase: Balance = deposit(1, 8);
	pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
	pub const MaxPending: u16 = 32;
}

/// The type used to represent the kinds of proxying allowed.
#[derive(
	Copy,
	Clone,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Encode,
	Decode,
	RuntimeDebug,
	MaxEncodedLen,
	TypeInfo,
)]
pub enum ProxyType {
	Any = 0,
	NonTransfer = 1,
	Governance = 2,
	Staking = 3,
	IdentityJudgement = 4,
	CancelProxy = 5,
}

impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}

impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => matches!(
				c,
				RuntimeCall::System(..)
                    | RuntimeCall::Scheduler(..)
                    | RuntimeCall::Babe(..)
                    | RuntimeCall::Timestamp(..)
                    | RuntimeCall::Indices(pallet_indices::Call::claim{..})
                    | RuntimeCall::Indices(pallet_indices::Call::free{..})
                    | RuntimeCall::Indices(pallet_indices::Call::freeze{..})
                    // Specifically omitting Indices `transfer`, `force_transfer`
                    // Specifically omitting the entire Balances pallet
                    | RuntimeCall::XStaking(..)
                    | RuntimeCall::Session(..)
                    | RuntimeCall::Grandpa(..)
                    | RuntimeCall::ImOnline(..)
                    | RuntimeCall::Democracy(..)
                    | RuntimeCall::Council(..)
                    | RuntimeCall::TechnicalCommittee(..)
                    | RuntimeCall::Elections(..)
                    | RuntimeCall::TechnicalMembership(..)
                    | RuntimeCall::Treasury(..)
                    | RuntimeCall::Utility(..)
                    | RuntimeCall::Identity(..)
                    | RuntimeCall::Proxy(..)
                    | RuntimeCall::Multisig(..)
			),
			ProxyType::Governance => matches!(
				c,
				RuntimeCall::Democracy(..) |
					RuntimeCall::Council(..) |
					RuntimeCall::TechnicalCommittee(..) |
					RuntimeCall::Elections(..) |
					RuntimeCall::Treasury(..) |
					RuntimeCall::Utility(..)
			),
			ProxyType::Staking => matches!(
				c,
				RuntimeCall::XStaking(..) | RuntimeCall::Session(..) | RuntimeCall::Utility(..)
			),
			ProxyType::IdentityJudgement => matches!(
				c,
				RuntimeCall::Identity(pallet_identity::Call::provide_judgement { .. }) |
					RuntimeCall::Utility(..)
			),
			ProxyType::CancelProxy => {
				matches!(c, RuntimeCall::Proxy(pallet_proxy::Call::reject_announcement { .. }))
			},
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			(ProxyType::NonTransfer, _) => true,
			_ => false,
		}
	}
}

impl pallet_proxy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = MaxProxies;
	type WeightInfo = pallet_proxy::weights::SubstrateWeight<Runtime>;
	type MaxPending = MaxPending;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

///////////////////////////////////////////
// Chainx pallets
///////////////////////////////////////////
impl xpallet_system::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
}

impl xpallet_gateway_records::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = xpallet_gateway_records::weights::SubstrateWeight<Runtime>;
}

pub struct MultisigProvider;
impl MultisigAddressFor<AccountId> for MultisigProvider {
	fn calc_multisig(who: &[AccountId], threshold: u16) -> AccountId {
		Multisig::multi_account_id(who, threshold)
	}
}

impl xpallet_gateway_common::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Validator = XStaking;
	type DetermineMultisigAddress = MultisigProvider;
	type CouncilOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>;
	type Bitcoin = XGatewayBitcoin;
	type BitcoinTrustee = XGatewayBitcoin;
	type BitcoinTrusteeSessionProvider = trustees::bitcoin::BtcTrusteeSessionManager<Runtime>;
	type BitcoinTotalSupply = XGatewayBitcoin;
	type BitcoinWithdrawalProposal = XGatewayBitcoin;
	type WeightInfo = xpallet_gateway_common::weights::SubstrateWeight<Runtime>;
}

impl xpallet_gateway_bitcoin::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type UnixTime = Timestamp;
	type CouncilOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>;
	type AccountExtractor = xp_gateway_bitcoin::OpReturnExtractor;
	type TrusteeSessionProvider = trustees::bitcoin::BtcTrusteeSessionManager<Runtime>;
	type TrusteeInfoUpdate = XGatewayCommon;
	type ReferralBinding = XGatewayCommon;
	type AddressBinding = XGatewayCommon;
	type WeightInfo = xpallet_gateway_bitcoin::weights::SubstrateWeight<Runtime>;
}

pub struct SimpleTreasuryAccount;
impl xpallet_support::traits::TreasuryAccount<AccountId> for SimpleTreasuryAccount {
	fn treasury_account() -> AccountId {
		TreasuryPalletId::get().into_account_truncating()
	}
}

parameter_types! {
	// Total issuance is 7723350PCX by the end of ChainX 1.0.
	// 210000 - (7723350 / 50) = 55533
	pub const MigrationSessionOffset: SessionIndex = 55533;
	pub const MinimumReferralId: u32 = 2;
	pub const MaximumReferralId: u32 = 12;
}

impl xpallet_mining_staking::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type SessionDuration = SessionDuration;
	type MinimumReferralId = MinimumReferralId;
	type MaximumReferralId = MaximumReferralId;
	type SessionInterface = Self;
	type TreasuryAccount = SimpleTreasuryAccount;
	type AssetMining = ();
	type DetermineRewardPotAccount =
		xpallet_mining_staking::SimpleValidatorRewardPotAccountDeterminer<Runtime>;
	type ValidatorRegistration = Session;
	type WeightInfo = xpallet_mining_staking::weights::SubstrateWeight<Runtime>;
}

impl pallet_sudo::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}

impl xpallet_ethereum_chain_id::Config for Runtime {}

impl xpallet_btc_ledger::Config for Runtime {
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type CouncilOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>;
	type PalletId = TreasuryPalletId;
}

// /// Current approximation of the gas/s consumption considering
// /// EVM execution over compiled WASM (on 4.4Ghz CPU).
// /// Given the 500ms Weight, from which 75% only are used for transactions,
// /// the total EVM execution gas limit is: GAS_PER_SECOND * 0.500 * 0.75 ~= 15_000_000.
// pub const GAS_PER_SECOND: u64 = 40_000_000;
//
// /// Approximate ratio of the amount of Weight per Gas.
// /// u64 works for approximations because Weight is a very small unit compared to gas.
// pub const WEIGHT_PER_GAS: u64 = WEIGHT_PER_SECOND / GAS_PER_SECOND;
//
// /// Maximum weight per block
// pub const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;
//
// parameter_types! {
//     pub BlockGasLimit: U256
//         = U256::from(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT / WEIGHT_PER_GAS);
//     pub PrecompilesValue: ChainXPrecompiles<Runtime> = ChainXPrecompiles::<_>::new();
// }
//
// pub struct ChainXGasWeightMapping;
// impl pallet_evm::GasWeightMapping for ChainXGasWeightMapping {
//     fn gas_to_weight(gas: u64) -> Weight {
//         gas.saturating_mul(WEIGHT_PER_GAS)
//     }
//     fn weight_to_gas(weight: Weight) -> u64 {
//         weight.wrapping_div(WEIGHT_PER_GAS)
//     }
// }
//
// impl pallet_evm::Config for Runtime {
//     type FeeCalculator = BaseFee;
//     type GasWeightMapping = ChainXGasWeightMapping;
//     type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
//     type CallOrigin = EnsureAddressRoot<AccountId>;
//     type WithdrawOrigin = EnsureAddressNever<AccountId>;
//     type AddressMapping = HashedAddressMapping<BlakeTwo256>;
//     type Currency = XBtcLedger;
//     type RuntimeEvent = RuntimeEvent;
//     type Runner = pallet_evm::runner::stack::Runner<Self>;
//     type PrecompilesType = ChainXPrecompiles<Runtime>;
//     type PrecompilesValue = PrecompilesValue;
//     type ChainId = EthereumChainId;
//     type OnChargeTransaction = pallet_evm::EVMCurrencyAdapter<XBtcLedger, DealWithBTCFees>;
//     type BlockGasLimit = BlockGasLimit;
//     type FindAuthor = ();
//     type WeightInfo = pallet_evm::weights::SubstrateWeight<Self>;
// }

// impl pallet_ethereum::Config for Runtime {
//     type RuntimeEvent = RuntimeEvent;
//     type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
// }

parameter_types! {
	pub DefaultBaseFeePerGas: U256 = U256::from(BASE_FEE);
}

// parameter_types! {
//     // 0x1111111111111111111111111111111111111111
//     pub EvmCaller: H160 = H160::from_slice(&[17u8;20][..]);
// }
// impl xpallet_assets_bridge::Config for Runtime {
//     type RuntimeEvent = RuntimeEvent;
//     type EvmCaller = EvmCaller;
//     type NativeCurrency = Balances;
// }

construct_runtime!(
	pub struct Runtime
	{
		// Basic stuff.
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>} = 0,
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 2,

		// Must be before session.
		Babe: pallet_babe::{Pallet, Call, Storage, Config<T>, ValidateUnsigned} = 3,

		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 4,
		Indices: pallet_indices::{Pallet, Call, Storage, Config<T>, Event<T>} = 5,
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 6,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Event<T>} = 7,

		// Consensus support.
		Authorship: pallet_authorship::{Pallet, Storage} = 8,
		Offences: pallet_offences::{Pallet, Storage, Event} = 9,
		Historical: pallet_session_historical::{Pallet} = 10,
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 11,
		Grandpa: pallet_grandpa::{Pallet, Call, Storage, Config<T>, Event, ValidateUnsigned} = 12,
		ImOnline: pallet_im_online::{Pallet, Call, Storage, Event<T>, ValidateUnsigned, Config<T>} = 13,
		AuthorityDiscovery: pallet_authority_discovery::{Pallet, Config<T>} = 14,

		// Governance stuff.
		Democracy: pallet_democracy::{Pallet, Call, Storage, Config<T>, Event<T>} = 15,
		Council: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 16,
		TechnicalCommittee: pallet_collective::<Instance2>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 17,
		Elections: pallet_elections_phragmen::{Pallet, Call, Storage, Event<T>, Config<T>} = 18,
		TechnicalMembership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} = 19,
		Treasury: pallet_treasury::{Pallet, Call, Storage, Config<T>, Event<T>} = 20,
		Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>, HoldReason} = 121,

		Identity: pallet_identity::{Pallet, Call, Storage, Event<T>} = 21,

		Utility: pallet_utility::{Pallet, Call, Event} = 22,
		Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 23,

		// BEVM basics.
		XSystem: xpallet_system::{Pallet, Call, Storage, Event<T>} = 24,
		XStaking: xpallet_mining_staking::{Pallet, Call, Storage, Event<T>, Config<T>} = 27,

		// Crypto gateway stuff.
		XGatewayRecords: xpallet_gateway_records::{Pallet, Call, Storage, Event<T>} = 29,
		XGatewayCommon: xpallet_gateway_common::{Pallet, Call, Storage, Event<T>, Config<T>} = 30,
		XGatewayBitcoin: xpallet_gateway_bitcoin::{Pallet, Call, Storage, Event<T>, Config<T>} = 31,

		// It might be possible to merge this module into pallet_transaction_payment in future, thus
		// we put it at the end for keeping the extrinsic ordering.
		XTransactionFee: xpallet_transaction_fee::{Pallet, Event<T>} = 35,

		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>} = 36,

		Bounties: pallet_bounties::{Pallet, Call, Storage, Event<T>} = 37,
		Tips: pallet_tips::{Pallet, Call, Storage, Event<T>} = 38,

		// Put Sudo last so that the extrinsic ordering stays the same once it's removed.
		Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 39,

		// Ethereum compatibility
		EthereumChainId: xpallet_ethereum_chain_id::{Pallet, Call, Storage, Config<T>} = 40,
		// Evm: pallet_evm::{Pallet, Config, Call, Storage, Event<T>} = 41,
		// Ethereum: pallet_ethereum::{Pallet, Call, Storage, Event, Config, Origin} = 42,

		// XAssetsBridge: xpallet_assets_bridge::{Pallet, Call, Storage, Config<T>, Event<T>} = 45,

		XBtcLedger: xpallet_btc_ledger::{Pallet, Call, Storage, Config<T>, Event<T>} = 46,
	}
);

/// The address format for describing accounts.
pub type Address = <Indices as StaticLookup>::Source;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
	BaseFilter,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra>;

/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
>;

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}
		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> sp_std::vec::Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_consensus_grandpa::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> sp_consensus_grandpa::AuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn current_set_id() -> sp_consensus_grandpa::SetId {
			Grandpa::current_set_id()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			equivocation_proof: sp_consensus_grandpa::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			key_owner_proof: sp_consensus_grandpa::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			let key_owner_proof = key_owner_proof.decode()?;

			Grandpa::submit_unsigned_equivocation_report(
				equivocation_proof,
				key_owner_proof,
			)
		}

		fn generate_key_ownership_proof(
			_set_id: sp_consensus_grandpa::SetId,
			authority_id: GrandpaId,
		) -> Option<sp_consensus_grandpa::OpaqueKeyOwnershipProof> {
			use parity_scale_codec::Encode;

			Historical::prove((sp_consensus_grandpa::KEY_TYPE, authority_id))
				.map(|p| p.encode())
				.map(sp_consensus_grandpa::OpaqueKeyOwnershipProof::new)
		}
	}

	impl sp_consensus_babe::BabeApi<Block> for Runtime {
		fn configuration() -> sp_consensus_babe::BabeConfiguration {
			// The choice of `c` parameter (where `1 - c` represents the
			// probability of a slot being empty), is done in accordance to the
			// slot duration and expected target block time, for safely
			// resisting network delays of maximum two seconds.
			// <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
			let epoch_config = Babe::epoch_config().unwrap_or(BABE_GENESIS_EPOCH_CONFIG);
			sp_consensus_babe::BabeConfiguration {
				slot_duration: Babe::slot_duration(),
				epoch_length: EpochDuration::get(),
				c: epoch_config.c,
				authorities: Babe::authorities().to_vec(),
				randomness: Babe::randomness(),
				allowed_slots: epoch_config.allowed_slots,
			}
		}

		fn current_epoch_start() -> sp_consensus_babe::Slot {
			Babe::current_epoch_start()
		}

		fn current_epoch() -> sp_consensus_babe::Epoch {
			Babe::current_epoch()
		}

		fn next_epoch() -> sp_consensus_babe::Epoch {
			Babe::next_epoch()
		}

		fn generate_key_ownership_proof(
			_slot: sp_consensus_babe::Slot,
			authority_id: sp_consensus_babe::AuthorityId,
		) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
			Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
				.map(|p| p.encode())
				.map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
			key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			let key_owner_proof = key_owner_proof.decode()?;

			Babe::submit_unsigned_equivocation_report(
				equivocation_proof,
				key_owner_proof,
			)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl sp_authority_discovery::AuthorityDiscoveryApi<Block> for Runtime {
		fn authorities() -> Vec<AuthorityDiscoveryId> {
			AuthorityDiscovery::authorities()
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl xpallet_mining_staking_rpc_runtime_api::XStakingApi<Block, AccountId, Balance, VoteWeight, BlockNumber> for Runtime {
		fn validators() -> Vec<ValidatorInfo<AccountId, Balance, VoteWeight, BlockNumber>> {
			XStaking::validators_info()
		}
		fn validator_info_of(who: AccountId) -> ValidatorInfo<AccountId, Balance, VoteWeight, BlockNumber> {
			XStaking::validator_info_of(who)
		}
		fn staking_dividend_of(who: AccountId) -> BTreeMap<AccountId, Balance> {
			XStaking::staking_dividend_of(who)
		}
		fn nomination_details_of(who: AccountId) -> BTreeMap<AccountId, NominatorLedger<Balance, VoteWeight, BlockNumber>> {
			XStaking::nomination_details_of(who)
		}
		fn nominator_info_of(who: AccountId) -> NominatorInfo<BlockNumber> {
			XStaking::nominator_info_of(who)
		}
	}

	impl xpallet_gateway_records_rpc_runtime_api::XGatewayRecordsApi<Block, AccountId, Balance, BlockNumber> for Runtime {
		fn withdrawal_list() -> BTreeMap<u32, Withdrawal<AccountId, Balance, BlockNumber>> {
			XGatewayRecords::withdrawal_list()
		}

		fn withdrawal_list_by_chain(chain: Chain) -> BTreeMap<u32, Withdrawal<AccountId, Balance, BlockNumber>> {
			XGatewayRecords::withdrawals_list_by_chain(chain)
		}
	}

	impl xpallet_gateway_bitcoin_rpc_runtime_api::XGatewayBitcoinApi<Block, AccountId> for Runtime {
		fn verify_tx_valid(
			raw_tx: Vec<u8>,
			withdrawal_id_list: Vec<u32>,
			full_amount: bool,
		) -> Result<bool, DispatchError> {
			XGatewayBitcoin::verify_tx_valid(raw_tx, withdrawal_id_list, full_amount)
		}

		fn get_withdrawal_proposal() -> Option<BtcWithdrawalProposal<AccountId>> {
			XGatewayBitcoin::get_withdrawal_proposal()
		}

		fn get_genesis_info() -> (BtcHeader, u32) {
			XGatewayBitcoin::get_genesis_info()
		}

		fn get_btc_block_header(txid: H256) -> Option<BtcHeaderInfo> {
			XGatewayBitcoin::get_btc_block_header(txid)
		}
	}

	impl xpallet_btc_ledger_runtime_api::BtcLedgerApi<Block, AccountId, Balance> for Runtime {
		fn get_balance(who: AccountId) -> Balance {
			XBtcLedger::free_balance(&who)
		}
		fn get_total() -> Balance {
			XBtcLedger::get_total()
		}
	}

	impl xpallet_gateway_common_rpc_runtime_api::XGatewayCommonApi<Block, AccountId, Balance, BlockNumber> for Runtime {
		fn bound_addrs(who: AccountId) -> BTreeMap<Chain, Vec<ChainAddress>> {
			XGatewayCommon::bound_addrs(&who)
		}

		fn withdrawal_limit(asset_id: AssetId) -> Result<WithdrawalLimit<Balance>, DispatchError> {
			XGatewayCommon::withdrawal_limit(&asset_id)
		}

		#[allow(clippy::type_complexity)]
		fn withdrawal_list_with_fee_info(asset_id: AssetId) -> Result<
			BTreeMap<
				WithdrawalRecordId,
				(
					Withdrawal<AccountId, Balance, BlockNumber>,
					WithdrawalLimit<Balance>,
				),
			>,
			DispatchError,
		>
		{
			XGatewayCommon::withdrawal_list_with_fee_info(&asset_id)
		}

		fn verify_withdrawal(asset_id: AssetId, value: Balance, addr: AddrStr, memo: Memo) -> Result<(), DispatchError> {
			XGatewayCommon::verify_withdrawal(asset_id, value, &addr, &memo)
		}

		fn trustee_multisigs() -> BTreeMap<Chain, AccountId> {
			XGatewayCommon::trustee_multisigs()
		}

		fn trustee_properties(chain: Chain, who: AccountId) -> Option<GenericTrusteeIntentionProps<AccountId>> {
			XGatewayCommon::trustee_intention_props_of(who, chain)
		}

		fn trustee_session_info(chain: Chain, session_number: i32) -> Option<GenericTrusteeSessionInfo<AccountId, BlockNumber>> {
			if session_number < 0 {
				let number = match session_number {
					-1i32 => Some(XGatewayCommon::trustee_session_info_len(chain)),
					-2i32 => XGatewayCommon::trustee_session_info_len(chain).checked_sub(1),
					_ => None
				};
				if let Some(number) = number {
					XGatewayCommon::trustee_session_info_of(chain, number)
				}else{
					None
				}
			}else{
				let number = session_number as u32;
				XGatewayCommon::trustee_session_info_of(chain, number)
			}

		}

		fn generate_trustee_session_info(chain: Chain, candidates: Vec<AccountId>) -> Result<(GenericTrusteeSessionInfo<AccountId, BlockNumber>, ScriptInfo<AccountId>), DispatchError> {
			let info = XGatewayCommon::try_generate_session_info(chain, candidates)?;
			// check multisig address
			let _ = XGatewayCommon::generate_multisig_addr(chain, &info.0)?;
			Ok(info)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade() -> (Weight, Weight) {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here. If any of the pre/post migration checks fail, we shall stop
			// right here and right now.
			let weight = Executive::try_runtime_upgrade().unwrap();
			(weight, BlockWeights::get().max_block)
		}

		fn execute_block_no_check(block: Block) -> Weight {
			Executive::execute_block_no_check(block)
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{list_benchmark, Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;

			let mut list = Vec::<BenchmarkList>::new();

			list_benchmark!(list, extra, xpallet_mining_staking, XStaking);
			list_benchmark!(list, extra, xpallet_gateway_records, XGatewayRecords);
			list_benchmark!(list, extra, xpallet_gateway_common, XGatewayCommon);
			list_benchmark!(list, extra, xpallet_gateway_bitcoin, XGatewayBitcoin);

			let storage_info = AllPalletsWithSystem::storage_info();

			return (list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, RuntimeString> {
			use frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch, TrackedStorageKey};

			use frame_system_benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;

			impl frame_system_benchmarking::Config for Runtime {}
			impl baseline::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// // Block Number
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
				// // Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
				// // Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
				// // Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
				// // System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
				// // Treasury Account
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da95ecffd7b6c0f78751baa9d281e0bfa3a6d6f646c70792f74727372790000000000000000000000000000000000000000").to_vec().into(),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			add_benchmarks!(params, batches);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}
}

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	define_benchmarks!(
		[frame_benchmarking, BaselineBench::<Runtime>]
		[frame_system, SystemBench::<Runtime>]
		[xpallet_mining_staking, XStaking]
		[xpallet_gateway_records, XGatewayRecords]
		[xpallet_gateway_common,  XGatewayCommon]
		[xpallet_gateway_bitcoin, XGatewayBitcoin]
	);
}
