use appchain_barnacle_runtime::{
	currency::{OCTS, UNITS as BEAT},
	opaque::{Block, SessionKeys},
	AccountId, BabeConfig, Balance, BalancesConfig, GenesisConfig, GrandpaConfig, ImOnlineConfig,
	OctopusAppchainConfig, OctopusLposConfig, SessionConfig, Signature, SudoConfig, SystemConfig,
	WASM_BINARY,
};
use beefy_primitives::crypto::AuthorityId as BeefyId;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_octopus_appchain::sr25519::AuthorityId as OctopusId;
use sc_chain_spec::ChainSpecExtension;
use sc_service::{ChainType, Properties};
use serde::{Deserialize, Serialize};
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

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
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

pub fn octopus_testnet_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../resources/octopus-testnet.json")[..])
}

pub fn octopus_mainnet_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../resources/octopus-mainnet.json")[..])
}

fn session_keys(
	babe: BabeId,
	grandpa: GrandpaId,
	im_online: ImOnlineId,
	beefy: BeefyId,
	octopus: OctopusId,
) -> SessionKeys {
	SessionKeys { babe, grandpa, im_online, beefy, octopus }
}

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(
	seed: &str,
	stash_amount: Balance,
) -> (AccountId, BabeId, GrandpaId, ImOnlineId, BeefyId, OctopusId, Balance) {
	(
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<BeefyId>(seed),
		get_from_seed::<OctopusId>(seed),
		stash_amount,
	)
}

/// Helper function to generate an properties
pub fn get_properties(symbol: &str, decimals: u32, ss58format: u32) -> Properties {
	let mut properties = Properties::new();
	properties.insert("tokenSymbol".into(), symbol.into());
	properties.insert("tokenDecimals".into(), decimals.into());
	properties.insert("ss58Format".into(), ss58format.into());

	properties
}

/// Helper function to generate appchain config
pub fn appchain_config(
	anchor_contract: &str,
	asset_id_by_token_id: &str,
	premined_amount: Balance,
	era_payout: Balance,
) -> (String, String, Balance, Balance) {
	(anchor_contract.to_string(), asset_id_by_token_id.to_string(), premined_amount, era_payout)
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
	let properties = get_properties("BEAT", 18, 42);

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice", 50_000 * OCTS)],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				Some(vec![
					(get_account_id_from_seed::<sr25519::Public>("Alice"), 12_500_000 * BEAT),
					(get_account_id_from_seed::<sr25519::Public>("Bob"), 10_000_000 * BEAT),
				]),
				// Appchain config
				appchain_config(
					// Appchain Relay Contract
					"",
					// Appchain Asset Id by Name
					"usdn.testnet",
					// Premined Amount
					87_500_000 * BEAT,
					// Era Payout
					4_657 * BEAT,
				),
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		Some("beat-development"),
		None,
		// Properties
		Some(properties),
		// Extensions
		Default::default(),
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
	let properties = get_properties("BEAT", 18, 42);

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![
					authority_keys_from_seed("Alice", 50_000 * OCTS),
					authority_keys_from_seed("Bob", 50_000 * OCTS),
				],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				Some(vec![
					(get_account_id_from_seed::<sr25519::Public>("Alice"), 12_499_990 * BEAT),
					(get_account_id_from_seed::<sr25519::Public>("Bob"), 10 * BEAT),
				]),
				// Appchain config
				appchain_config(
					// Appchain Relay Contract
					"",
					// Appchain Asset Id by Name
					"usdn.testnet",
					// Premined Amount
					87_500_000 * BEAT,
					// Era Payout
					4_657 * BEAT,
				),
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		Some("beat-local"),
		// Properties
		None,
		Some(properties),
		// Extensions
		Default::default(),
	))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(
		AccountId,
		BabeId,
		GrandpaId,
		ImOnlineId,
		BeefyId,
		OctopusId,
		Balance,
	)>,
	root_key: AccountId,
	endowed_accounts: Option<Vec<(AccountId, Balance)>>,
	appchain_config: (String, String, Balance, Balance),
	_enable_println: bool,
) -> GenesisConfig {
	let endowed_accounts: Vec<(AccountId, Balance)> = endowed_accounts.unwrap_or_else(|| {
		vec![
			(get_account_id_from_seed::<sr25519::Public>("Alice"), 0),
			(get_account_id_from_seed::<sr25519::Public>("Bob"), 0),
			(get_account_id_from_seed::<sr25519::Public>("Charlie"), 0),
			(get_account_id_from_seed::<sr25519::Public>("Dave"), 0),
			(get_account_id_from_seed::<sr25519::Public>("Eve"), 0),
			(get_account_id_from_seed::<sr25519::Public>("Ferdie"), 0),
		]
	});
	// endow all authorities.
	// initial_authorities.iter().map(|x| &x.0).for_each(|x| {
	// 	if !endowed_accounts.contains(x) {
	// 		endowed_accounts.push(x.clone())
	// 	}
	// });

	let validators = initial_authorities.iter().map(|x| (x.0.clone(), x.6)).collect::<Vec<_>>();

	// const ENDOWMENT: Balance = 10_000_000 * DOLLARS;
	// const STASH: Balance = 100 * 1_000_000_000_000_000_000; // 100 OCT with 18 decimals

	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k.0.clone(), k.1)).collect(),
		},
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						session_keys(
							x.1.clone(),
							x.2.clone(),
							x.3.clone(),
							x.4.clone(),
							x.5.clone(),
						),
					)
				})
				.collect::<Vec<_>>(),
		},
		sudo: SudoConfig { key: Some(root_key) },
		babe: BabeConfig {
			authorities: vec![],
			epoch_config: Some(appchain_barnacle_runtime::BABE_GENESIS_EPOCH_CONFIG),
		},
		im_online: ImOnlineConfig { keys: vec![] },
		grandpa: GrandpaConfig { authorities: vec![] },
		transaction_payment: Default::default(),
		beefy: Default::default(),
		octopus_appchain: OctopusAppchainConfig {
			anchor_contract: appchain_config.0,
			asset_id_by_token_id: vec![(appchain_config.1, 0)],
			validators,
			premined_amount: appchain_config.2,
		},
		octopus_lpos: OctopusLposConfig { era_payout: appchain_config.3, ..Default::default() },
		octopus_assets: Default::default(),
	}
}
