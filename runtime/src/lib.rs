#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "512"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod apis;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod configs;
mod weights;

use smallvec::smallvec;
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    traits::{BlakeTwo256, IdentifyAccount, Verify},
    MultiSignature,
};
use frame_support::traits::fungible::HoldConsideration;
use frame_support::traits::EqualPrivilegeOnly;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_system::EnsureWithSuccess;
use sp_runtime::RuntimeDebug;
use frame_support::traits::tokens::UnityAssetBalanceConversion;
use pallet_identity::legacy::IdentityInfo;
use frame_support::traits::WithdrawReasons;
use frame_support::traits::InstanceFilter;
use frame_support::traits::LinearStoragePrice;
use sp_runtime::traits::ConvertInto;
use pallet_nfts::PalletFeatures;
use frame_support::BoundedVec;
use frame_support::traits::AsEnsureOriginWithArg;
use frame_support::traits::tokens::pay::PayAssetFromAccount;
use frame_support::traits::EitherOfDiverse;
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use frame_system::EnsureRoot;
use frame_support::PalletId;
use frame_system::EnsureSigned;
use frame_support::parameter_types;
use frame_support::traits::ConstU32;
use frame_support::traits::ConstU128;
use sp_runtime::traits::ConstU64;
use frame_support::traits::VariantCountOf;
use pallet_transaction_payment::Multiplier;
use frame_support::weights::IdentityFee;
use pallet_transaction_payment::ConstFeeMultiplier;
use pallet_transaction_payment::FungibleAdapter;
use sp_runtime::traits::One;

use crate::configs::xcm_config::RelayLocation;
use pallet_xcm::EnsureXcm;
use pallet_xcm::IsVoiceOfBody;
use xcm::latest::prelude::*;


use frame_support::weights::{
    constants::WEIGHT_REF_TIME_PER_SECOND, Weight, WeightToFeeCoefficient, WeightToFeeCoefficients,
    WeightToFeePolynomial,
};
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
pub use sp_runtime::{MultiAddress, Perbill, Permill};

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

use weights::ExtrinsicBaseWeight;

/// Import the template pallet.
pub use pallet_parachain_template;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Nonce = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// An index to a block.
pub type BlockNumber = u32;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, ()>;

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;

/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// The SignedExtension to the basic transaction logic.
#[docify::export(template_signed_extra)]
pub type SignedExtra = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
    cumulus_primitives_storage_weight_reclaim::StorageWeightReclaim<Runtime>,
    frame_metadata_hash_extension::CheckMetadataHash<Runtime>,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
    generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
>;

/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
/// node's balance type.
///
/// This should typically create a mapping between the following ranges:
///   - `[0, MAXIMUM_BLOCK_WEIGHT]`
///   - `[Balance::min, Balance::max]`
///
/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
///   - Setting it to `0` will essentially disable the weight fee.
///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
    type Balance = Balance;
    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        // in Rococo, extrinsic base weight (smallest non-zero weight) is mapped to 1 CENTS:
        // in our template, we map to 1/10 of that, or 1/10 CENTS
        let p = CENTS / 10;
        let q = 100 * Balance::from(ExtrinsicBaseWeight::get().ref_time());
        smallvec![WeightToFeeCoefficient {
            degree: 1,
            negative: false,
            coeff_frac: Perbill::from_rational(p % q, q),
            coeff_integer: p / q,
        }]
    }
}

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
    use super::*;
    use sp_runtime::{
        generic,
        traits::{BlakeTwo256, Hash as HashT},
    };

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;
    /// Opaque block hash type.
    pub type Hash = <BlakeTwo256 as HashT>::Output;
}

impl_opaque_keys! {
    pub struct SessionKeys {
        pub aura: Aura,
    }
}
impl pallet_insecure_randomness_collective_flip::Config for Runtime {} 

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("parachain-template-runtime"),
    impl_name: create_runtime_str!("parachain-template-runtime"),
    authoring_version: 1,
    spec_version: 1,
    impl_version: 0,
    apis: apis::RUNTIME_API_VERSIONS,
    transaction_version: 1,
    state_version: 1,
};

/// This determines the average expected block time that we are targeting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
pub const MILLISECS_PER_BLOCK: u64 = 6000;

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
//       Attempting to do so will brick block production.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

// Unit = the base number of indivisible units for balances
pub const MILLICENTS: Balance = 1_000_000_000;
pub const CENTS: Balance = 1_000 * MILLICENTS; // assume this is worth about a cent.
pub const DOLLARS: Balance = 100 * CENTS;

pub const fn deposit(items: u32, bytes: u32) -> Balance {
    items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
}
parameter_types! {
pub BlockWeights: frame_system::limits::BlockWeights =
frame_system::limits::BlockWeights::with_sensible_defaults(
	Weight::from_parts(2u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
	NORMAL_DISPATCH_RATIO,
);}



/// We assume that ~5% of the block weight is consumed by `on_initialize` handlers. This is
/// used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5);

/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
/// `Operational` extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// We allow for 2 seconds of compute with a 6 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_parts(
    WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2),
    cumulus_primitives_core::relay_chain::MAX_POV_SIZE as u64,
);

/// Maximum number of blocks simultaneously accepted by the Runtime, not yet included
/// into the relay chain.
const UNINCLUDED_SEGMENT_CAPACITY: u32 = 3;
/// How many parachain blocks are processed by the relay chain per parent. Limits the
/// number of blocks authored per slot.
const BLOCK_PROCESSING_VELOCITY: u32 = 1;
/// Relay chain slot duration, in milliseconds.
const RELAY_CHAIN_SLOT_DURATION_MILLIS: u32 = 6000;

/// Aura consensus hook
type ConsensusHook = cumulus_pallet_aura_ext::FixedVelocityConsensusHook<
    Runtime,
    RELAY_CHAIN_SLOT_DURATION_MILLIS,
    BLOCK_PROCESSING_VELOCITY,
    UNINCLUDED_SEGMENT_CAPACITY,
>;

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}


// Create the runtime by composing the FRAME pallets that were previously configured.
#[frame_support::runtime]
mod runtime {
    #[runtime::runtime]
    #[runtime::derive(
        RuntimeCall,
        RuntimeEvent,
        RuntimeError,
        RuntimeOrigin,
        RuntimeFreezeReason,
        RuntimeHoldReason,
        RuntimeSlashReason,
        RuntimeLockId,
        RuntimeTask
    )]
    pub struct Runtime;

    #[runtime::pallet_index(0)]
    pub type System = frame_system;
    #[runtime::pallet_index(1)]
    pub type ParachainSystem = cumulus_pallet_parachain_system;
   
    #[runtime::pallet_index(3)]
    pub type ParachainInfo = parachain_info;




   

    #[runtime::pallet_index(24)]
    pub type AuraExt = cumulus_pallet_aura_ext;

    // XCM helpers.
    #[runtime::pallet_index(30)]
    pub type XcmpQueue = cumulus_pallet_xcmp_queue;
    #[runtime::pallet_index(31)]
    pub type PolkadotXcm = pallet_xcm;
    #[runtime::pallet_index(32)]
    pub type CumulusXcm = cumulus_pallet_xcm;
    #[runtime::pallet_index(33)]
    pub type MessageQueue = pallet_message_queue;

    // Template
    #[runtime::pallet_index(50)]
    pub type TemplatePallet = pallet_parachain_template;
    #[runtime::pallet_index(51)]
    pub type RandomnessCollectiveFlip = pallet_insecure_randomness_collective_flip::Pallet<Runtime>;

    
    
	#[runtime::pallet_index(55)]
	pub type Multisig = pallet_multisig::Pallet<Runtime>;    
    
	#[runtime::pallet_index(56)]
	pub type Proxy = pallet_proxy::Pallet<Runtime>;    
    
	#[runtime::pallet_index(57)]
	pub type Sudo = pallet_sudo;    
    
	#[runtime::pallet_index(58)]
	pub type Session = pallet_session;    
    
	#[runtime::pallet_index(59)]
	pub type Preimage = pallet_preimage::Pallet<Runtime>;    
    
	#[runtime::pallet_index(60)]
	pub type TechnicalCouncil = pallet_collective<Instance1>;    
    
	#[runtime::pallet_index(61)]
	pub type NftFractionalization = pallet_nft_fractionalization::Pallet<Runtime>;    
    
	#[runtime::pallet_index(62)]
	pub type Scheduler = pallet_scheduler::Pallet<Runtime>;    
    
	#[runtime::pallet_index(63)]
	pub type Democracy = pallet_democracy::Pallet<Runtime>;    
    
	#[runtime::pallet_index(64)]
	pub type Treasury = pallet_treasury::Pallet<Runtime>;    
    
	#[runtime::pallet_index(65)]
	pub type Identity = pallet_identity::Pallet<Runtime>;    
    
	#[runtime::pallet_index(66)]
	pub type Nfts = pallet_nfts::Pallet<Runtime>;    
    
	#[runtime::pallet_index(67)]
	pub type Timestamp = pallet_timestamp;    
    
	#[runtime::pallet_index(68)]
	pub type Membership = pallet_membership::Pallet<Runtime>;    
    
	#[runtime::pallet_index(69)]
	pub type TransactionPayment = pallet_transaction_payment;    
    
	#[runtime::pallet_index(70)]
	pub type Vesting = pallet_vesting::Pallet<Runtime>;    
    
	#[runtime::pallet_index(71)]
	pub type Aura = pallet_aura;    
    
	#[runtime::pallet_index(72)]
	pub type Authorship = pallet_authorship::Pallet<Runtime>;    
    
	#[runtime::pallet_index(73)]
	pub type GeneralCouncil = pallet_collective<Instance2>;    
    
	#[runtime::pallet_index(74)]
	pub type Utility = pallet_utility::Pallet<Runtime>;    
    
	#[runtime::pallet_index(75)]
	pub type Collective = pallet_collective::Pallet<Runtime>;    
    
	#[runtime::pallet_index(76)]
	pub type Balances = pallet_balances;    
    
	#[runtime::pallet_index(77)]
	pub type CollatorSelection = pallet_collator_selection;    
    
	#[runtime::pallet_index(78)]
	pub type Assets = pallet_assets::Pallet<Runtime>;
}


parameter_types! {
    pub const DepositBaseMultisig: u128 = 1000 * 1;
    pub const DepositFactorMultisig: u128 = 1000* 1;
    pub const MaxSignatoriesMultisig: u32 = 20;
}

impl pallet_multisig::Config for Runtime {
	type WeightInfo = pallet_multisig::weights::SubstrateWeight<Runtime>;
	type DepositFactor = DepositFactorMultisig;
	type RuntimeCall = RuntimeCall;
	type DepositBase = DepositBaseMultisig;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type MaxSignatories = MaxSignatoriesMultisig;

}

parameter_types! {
	// One storage item; key size 32, value size 8; .
	pub const ProxyDepositBase: Balance = deposit(1, 8);
	// Additional storage item size of 33 bytes.
	pub const ProxyDepositFactor: Balance = deposit(0, 33);
	pub const AnnouncementDepositBase: Balance = deposit(1, 8);
	pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
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
	scale_info::TypeInfo,
)]
pub enum ProxyType {
	Any,
}
impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}
impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, _c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
		}
	}
}

parameter_types! {
    pub const MaxProxies: u32 = 32;
    pub const MaxPending: u32 = 32;
}

impl pallet_proxy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type CallHasher = BlakeTwo256;
	type MaxPending = MaxPending;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
	type MaxProxies = MaxProxies;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type Currency = Balances;
	type RuntimeCall = RuntimeCall;
	type ProxyDepositFactor = ProxyDepositFactor;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type WeightInfo = pallet_proxy::weights::SubstrateWeight<Runtime>;

}


impl pallet_sudo::Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;

}

parameter_types! {
    pub const SessionPeriod: u32 = HOURS * 6;
    pub const Offset: u32 = 0;
}

impl pallet_session::Config for Runtime {
	type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
	type ShouldEndSession = pallet_session::PeriodicSessions<SessionPeriod, Offset>;
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type SessionManager = CollatorSelection;
	type WeightInfo = ();
	type NextSessionRotation = pallet_session::PeriodicSessions<SessionPeriod, Offset>;
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type Keys = SessionKeys;

}


parameter_types! {
	pub const PreimageBaseDeposit: Balance = deposit(2, 64);
	pub const PreimageByteDeposit: Balance = deposit(0, 1);
	pub const PreimageHoldReason: RuntimeHoldReason = RuntimeHoldReason::Preimage(pallet_preimage::HoldReason::Preimage);
}


impl pallet_preimage::Config for Runtime {
	type WeightInfo = pallet_preimage::weights::SubstrateWeight<Runtime>;
	type ManagerOrigin = EnsureRoot<AccountId>;
	type Currency = Balances;
	type Consideration = HoldConsideration<AccountId,Balances,PreimageHoldReason,LinearStoragePrice<PreimageBaseDeposit, PreimageByteDeposit, Balance>,>;
	type RuntimeEvent = RuntimeEvent;

}

parameter_types! {
    pub TechnicalMaxCollectivesProposalWeight: Weight = Perbill::from_percent(50) * BlockWeights::get().max_block;
    pub const TechnicalCouncilMaxProposals: u32 = 100;
    pub const TechnicalCouncilMaxMembers: u32 = 100;
    pub const TechnicalCouncilMotionDuration: BlockNumber = MINUTES * 3;
}

type TechnicalCouncilCollective = pallet_collective::Instance1;

impl pallet_collective::Config<TechnicalCouncilCollective> for Runtime{
	type MaxProposals = TechnicalCouncilMaxProposals;
	type MotionDuration = TechnicalCouncilMotionDuration;
	type RuntimeEvent = RuntimeEvent;
	type MaxMembers = TechnicalCouncilMaxMembers;
	type SetMembersOrigin = EnsureRoot<Self::AccountId>;
	type MaxProposalWeight = TechnicalMaxCollectivesProposalWeight;
	type Proposal = RuntimeCall;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
	type RuntimeOrigin = RuntimeOrigin;
	type DefaultVote = pallet_collective::PrimeDefaultVote;

}


parameter_types! {
	pub const NftFractionalizationPalletId: PalletId = PalletId(*b"fraction");
	pub NewAssetSymbol: BoundedVec<u8, AssetsStringLimit> = (*b"FRAC").to_vec().try_into().unwrap();
	pub NewAssetName: BoundedVec<u8, AssetsStringLimit> = (*b"Frac").to_vec().try_into().unwrap();
}

parameter_types! {
    pub const AssetsStringLimit: u32 = 50;
}

impl pallet_nft_fractionalization::Config for Runtime {
	type AssetId = <Self as pallet_assets::Config>::AssetId;
	type StringLimit = AssetsStringLimit;
	type PalletId = NftFractionalizationPalletId;
	type NewAssetSymbol = NewAssetSymbol;
	type NewAssetName = NewAssetName;
	type NftId = <Self as pallet_nfts::Config>::ItemId;
	type RuntimeHoldReason = RuntimeHoldReason;
	type AssetBalance = <Self as pallet_balances::Config>::Balance;
	type Assets = Assets;
	type NftCollectionId = <Self as pallet_nfts::Config>::CollectionId;
	type RuntimeEvent = RuntimeEvent;
	type Deposit = AssetDeposit;
	type WeightInfo = pallet_nft_fractionalization::weights::SubstrateWeight<Runtime>;
	type Currency = Balances;
	type Nfts = Nfts;

}

parameter_types! {
    pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * BlockWeights::get().max_block;
}

impl pallet_scheduler::Config for Runtime {
	type MaximumWeight = MaximumSchedulerWeight;
	type WeightInfo = ();
	type RuntimeCall = RuntimeCall;
	type Preimages = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeEvent = RuntimeEvent;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = ();
	type PalletsOrigin = OriginCaller;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;

}

parameter_types! {
    pub const VoteLockingPeriodDemocracy: u32 = MINUTES * 2;
    pub const MinimumDeposit: u128 = 1_000_000_000_000*100;
    pub const Period: u32 = MINUTES * 5;
    pub const CooloffPeriod: u32 = MINUTES * 2;
    pub const MaxAll: u32 = 128;
    pub const FastTrackVotingPeriod: u32 = MINUTES / 2;
    pub const InstantAllowed: bool = true;
}

impl pallet_democracy::Config for Runtime {
	type Currency = Balances;
	type FastTrackOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, (), 2, 3>;
	type ExternalMajorityOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, (), 1, 2>;
	type Slash = ();
	type CooloffPeriod = CooloffPeriod;
	type CancellationOrigin = EitherOfDiverse<EnsureRoot<AccountId>, pallet_collective::EnsureProportionAtLeast<AccountId, (), 2, 3>>;
	type MaxVotes = MaxAll;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	type MaxBlacklisted = MaxAll;
	type MaxProposals = MaxAll;
	type PalletsOrigin = OriginCaller;
	type SubmitOrigin = EnsureSigned<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, ()>;
	type ExternalDefaultOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, (), 1, 1>;
	type MaxDeposits = MaxAll;
	type EnactmentPeriod = Period;
	type LaunchPeriod = Period;
	type Preimages = Preimage;
	type InstantOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, (), 1, 1>;
	type Scheduler = Scheduler;
	type VotingPeriod = Period;
	type MinimumDeposit = MinimumDeposit;
	type VoteLockingPeriod = VoteLockingPeriodDemocracy;
	type CancelProposalOrigin = EitherOfDiverse<EnsureRoot<AccountId>, pallet_collective::EnsureProportionAtLeast<AccountId, (), 1, 1>>;
	type InstantAllowed = InstantAllowed;
	type ExternalOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, (), 1, 2>;
	type WeightInfo = pallet_democracy::weights::SubstrateWeight<Runtime>;

}

parameter_types! {
	
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const MaxBalance: Balance = Balance::max_value();
	pub TreasuryAccount: AccountId = Treasury::account_id();
}

parameter_types! {
    pub const TreasuryBurn: Permill = Permill::from_percent(50);
    pub const TreasuryMaxApprovals: u32 = 100;
    pub const TreasurySpendPeriod: BlockNumber = DAYS * 1;
    pub const TreasurySpendPayoutPeriod: BlockNumber = DAYS * 30;
}

impl pallet_treasury::Config for Runtime {
	type SpendPeriod = TreasurySpendPeriod;
	type Burn = TreasuryBurn;
	type BurnDestination = ();
	type BalanceConverter = UnityAssetBalanceConversion;
	type Paymaster = PayAssetFromAccount<Assets, TreasuryAccount>;
	type SpendFunds = ();
	type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
	type RejectOrigin = EnsureRoot<AccountId>;
	type BeneficiaryLookup = Self::Lookup;
	type MaxApprovals = TreasuryMaxApprovals;
	type SpendOrigin = EnsureWithSuccess<EnsureRoot<AccountId>, AccountId, MaxBalance>;
	type RuntimeEvent = RuntimeEvent;
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type PayoutPeriod = TreasurySpendPayoutPeriod;
	type Beneficiary = AccountId;
	type AssetKind = u32;

}

parameter_types! {
    pub const MaxRegistrars: u32 = 20;
    pub const SubAccountDeposit: u128 = 1000*100;
    pub const MaxUsernameLength: u32 = 32;
    pub const PendingUsernameExpiration: u32 = DAYS * 7;
    pub const BasicDeposit: u128 = 1000*1;
    pub const ByteDeposit: u128 = 1000*1;
    pub const MaxSuffixLength: u32 = 7;
    pub const MaxSubAccounts: u32 = 100;
}

impl pallet_identity::Config for Runtime {
	type MaxSuffixLength = MaxSuffixLength;
	type RegistrarOrigin = EnsureRoot<Self::AccountId>;
	type UsernameAuthorityOrigin = EnsureRoot<Self::AccountId>;
	type MaxSubAccounts = MaxSubAccounts;
	type PendingUsernameExpiration = PendingUsernameExpiration;
	type WeightInfo = pallet_identity::weights::SubstrateWeight<Runtime>;
	type SubAccountDeposit = SubAccountDeposit;
	type ForceOrigin = EnsureRoot<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type OffchainSignature = Signature;
	type Currency = Balances;
	type ByteDeposit = ByteDeposit;
	type Slashed = ();
	type MaxRegistrars = MaxRegistrars;
	type SigningPublicKey = <Signature as sp_runtime::traits::Verify>::Signer;
	type IdentityInformation = IdentityInfo<ConstU32<100>>;
	type MaxUsernameLength = MaxUsernameLength;
	type BasicDeposit = BasicDeposit;

}

parameter_types! {
        pub NftsPalletFeatures: PalletFeatures = PalletFeatures::all_enabled();
}

parameter_types! {
    pub const MetadataDepositBaseNfts: u128 = 1000*1;
    pub const MaxDeadlineDurationNfts: u32 = 1000;
    pub const StringLimitNfts: u32 = 256;
    pub const ItemAttributesApprovalsLimitNfts: u32 = 100;
    pub const DepositPerByteNfts: u128 = 10;
    pub const KeyLimitNfts: u32 = 64;
    pub const MaxAttributesPerCallNfts: u32 = 5;
    pub const CollectionDepositNfts: u128 = 1000*10;
    pub const AttributeDepositBaseNfts: u128 = 1000*1;
    pub const MaxTipsNfts: u32 = 10;
    pub const ValueLimitNfts: u32 = 256;
    pub const ApprovalsLimitNfts: u32 = 100;
    pub const ItemDepositNfts: u128 = 1000*1;
}

impl pallet_nfts::Config for Runtime {
	type MetadataDepositBase = MetadataDepositBaseNfts;
	type ItemAttributesApprovalsLimit = ItemAttributesApprovalsLimitNfts;
	type ItemDeposit = ItemDepositNfts;
	type ValueLimit = ValueLimitNfts;
	type MaxTips = MaxTipsNfts;
	type Features = NftsPalletFeatures;
	type OffchainPublic = <Signature as sp_runtime::traits::Verify>::Signer;
	type MaxDeadlineDuration = MaxDeadlineDurationNfts;
	type WeightInfo = pallet_nfts::weights::SubstrateWeight<Runtime>;
	type AttributeDepositBase = AttributeDepositBaseNfts;
	type OffchainSignature = Signature;
	type CollectionId = u32;
	type CreateOrigin = EnsureSigned<Self::AccountId>;
	type ForceOrigin = EnsureRoot<Self::AccountId>;
	type Currency = Balances;
	type Locker = ();
	type ItemId = u32;
	type KeyLimit = KeyLimitNfts;
	type RuntimeEvent = RuntimeEvent;
	type StringLimit = StringLimitNfts;
	type MaxAttributesPerCall = MaxAttributesPerCallNfts;
	type CollectionDeposit = CollectionDepositNfts;
	type DepositPerByte = DepositPerByteNfts;
	type ApprovalsLimit = ApprovalsLimitNfts;

}


impl pallet_timestamp::Config for Runtime {
	type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
	type Moment = u64;
	type OnTimestampSet = Aura;
	type WeightInfo = ();

}

parameter_types! {
    pub const TechnicalMaxMembers: u32 = 100;
}

impl pallet_membership::Config for Runtime {
	type MembershipInitialized = ();
	type WeightInfo = pallet_membership::weights::SubstrateWeight<Runtime>;
	type ResetOrigin = EnsureRoot<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type SwapOrigin = EnsureRoot<AccountId>;
	type MembershipChanged = ();
	type AddOrigin = EnsureRoot<AccountId>;
	type PrimeOrigin = EnsureRoot<AccountId>;
	type MaxMembers = TechnicalMaxMembers;
	type RemoveOrigin = EnsureRoot<AccountId>;

}

parameter_types! {
    pub FeeMultiplier: Multiplier = Multiplier::one();
}


parameter_types! {
    pub const FTPOperationalFeeMultiplier: u8 = 5;
}

impl pallet_transaction_payment::Config for Runtime {
	type LengthToFee = IdentityFee<Balance>;
	type RuntimeEvent = RuntimeEvent;
	type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
	type OperationalFeeMultiplier = FTPOperationalFeeMultiplier;
	type OnChargeTransaction = FungibleAdapter<Balances, ()>;
	type WeightToFee = IdentityFee<Balance>;

}

parameter_types! {
	pub const MinVestedTransfer: Balance = 100 * DOLLARS;
	pub UnvestedFundsAllowedWithdrawReasons: WithdrawReasons =
		WithdrawReasons::except(WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE);
}
impl pallet_vesting::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BlockNumberToBalance = ConvertInto;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = pallet_vesting::weights::SubstrateWeight<Runtime>;
	type UnvestedFundsAllowedWithdrawReasons = UnvestedFundsAllowedWithdrawReasons;
	type BlockNumberProvider = System;
	const MAX_VESTING_SCHEDULES: u32 = 28;
}



parameter_types! {
    pub const AllowMultipleBlocksPerSlot: bool = false;
    pub const MaxAuthoritiesAura: u32 = 32;
}

impl pallet_aura::Config for Runtime {
	type MaxAuthorities = MaxAuthoritiesAura;
	type DisabledValidators = ();
	type AllowMultipleBlocksPerSlot = AllowMultipleBlocksPerSlot;
	type SlotDuration = pallet_aura::MinimumPeriodTimesTwo<Runtime>;
	type AuthorityId = AuraId;

}


pub struct AuraAccountAdapter;
impl frame_support::traits::FindAuthor<AccountId> for AuraAccountAdapter {
	fn find_author<'a, I>(digests: I) -> Option<AccountId>
	where
		I: 'a + IntoIterator<Item = (frame_support::ConsensusEngineId, &'a [u8])>,
	{
		pallet_aura::AuraAuthorId::<Runtime>::find_author(digests)
			.and_then(|k| AccountId::try_from(k.as_ref()).ok())
	}
}




impl pallet_authorship::Config for Runtime {
	type EventHandler = ();
	type FindAuthor = AuraAccountAdapter;

}

parameter_types! {
    pub const GeneralCouncilMaxMembers: u32 = 100;
    pub const GeneralCouncilMotionDuration: BlockNumber = MINUTES * 3;
    pub const GeneralCouncilMaxProposals: u32 = 100;
    pub GeneralMaxCollectivesProposalWeight: Weight = Perbill::from_percent(50) * BlockWeights::get().max_block;
}

type GeneralCouncilCollective = pallet_collective::Instance2;

impl pallet_collective::Config<GeneralCouncilCollective> for Runtime{
	type MaxProposals = GeneralCouncilMaxProposals;
	type Proposal = RuntimeCall;
	type MaxProposalWeight = GeneralMaxCollectivesProposalWeight;
	type SetMembersOrigin = EnsureRoot<Self::AccountId>;
	type RuntimeOrigin = RuntimeOrigin;
	type MotionDuration = GeneralCouncilMotionDuration;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type RuntimeEvent = RuntimeEvent;
	type MaxMembers = GeneralCouncilMaxMembers;

}



impl pallet_utility::Config for Runtime {
	type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
	type PalletsOrigin = OriginCaller;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;

}

parameter_types! {
    pub MaxCollectivesProposalWeight: Weight = Perbill::from_percent(50) * BlockWeights::get().max_block;
    pub const CouncilMaxMembers: u32 = 100;
    pub const CouncilMotionDuration: BlockNumber = MINUTES * 3;
    pub const CouncilMaxProposals: u32 = 100;
}

impl pallet_collective::Config for Runtime {
	type MaxProposalWeight = MaxCollectivesProposalWeight;
	type RuntimeOrigin = RuntimeOrigin;
	type MaxMembers = CouncilMaxMembers;
	type SetMembersOrigin = EnsureRoot<Self::AccountId>;
	type Proposal = RuntimeCall;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
	type MaxProposals = CouncilMaxProposals;
	type MotionDuration = CouncilMotionDuration;
	type RuntimeEvent = RuntimeEvent;
	type DefaultVote = pallet_collective::PrimeDefaultVote;

}

/// Existential deposit.
pub const EXISTENTIAL_DEPOSIT: u128 = 500;


parameter_types! {
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = RuntimeHoldReason;
	type Balance = Balance;
	type ExistentialDeposit = ConstU128<500>;
	type DustRemoval = ();
	type AccountStore = System;
	type RuntimeFreezeReason = RuntimeHoldReason;
	type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
	type MaxReserves = ();
	type FreezeIdentifier = RuntimeFreezeReason;
	type MaxLocks = MaxLocks;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type RuntimeEvent = RuntimeEvent;

}
parameter_types! {
    pub const PotId: PalletId = PalletId(*b"PotStake");
  
    // StakingAdmin pluralistic body.
    pub const StakingAdminBodyId: BodyId = BodyId::Defense;
}

/// We allow root and the StakingAdmin to execute privileged collator selection operations.
pub type CollatorSelectionUpdateOrigin = EitherOfDiverse<
    EnsureRoot<AccountId>,
    EnsureXcm<IsVoiceOfBody<RelayLocation, StakingAdminBodyId>>,
>;


parameter_types! {
    pub const MaxInvulnerables: u32 = 20;
    pub const PCSMaxCandidates: u32 = 100;
    pub const MinEligibleCollators: u32 = 4;
    pub const SessionLength: BlockNumber = HOURS * 6;
}

impl pallet_collator_selection::Config for Runtime {
	type MaxInvulnerables = MaxInvulnerables;
	type RuntimeEvent = RuntimeEvent;
	type UpdateOrigin = CollatorSelectionUpdateOrigin;
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type KickThreshold = SessionPeriod;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type Currency = Balances;
	type ValidatorRegistration = Session;
	type MaxCandidates = PCSMaxCandidates;
	type WeightInfo = ();
	type PotId = PotId;
	type MinEligibleCollators = MinEligibleCollators;

}

parameter_types! {
    pub const RemoveItemsLimit: u32 = 1000;
    pub const AssetDeposit: Balance = DOLLARS * 100;
    pub const MetadataDepositPerByte: Balance = DOLLARS * 1;
    pub const StringLimit: u32 = 50;
    pub const ApprovalDeposit: Balance = DOLLARS * 1;
    pub const MetadataDepositBase: Balance = DOLLARS * 10;
    pub const AssetAccountDeposit: Balance = DOLLARS * 1;
}

impl pallet_assets::Config for Runtime {
	type AssetDeposit = AssetDeposit;
	type Extra = ();
	type RuntimeEvent = RuntimeEvent;
	type AssetAccountDeposit = AssetAccountDeposit;
	type Balance = u128;
	type AssetId = u32;
	type MetadataDepositBase = MetadataDepositBase;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
	type RemoveItemsLimit = RemoveItemsLimit;
	type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId>;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type AssetIdParameter = codec::Compact<u32>;
	type CallbackHandle = ();
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = StringLimit;
	type Freezer = ();

}
cumulus_pallet_parachain_system::register_validate_block! {
    Runtime = Runtime,
    BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
}

