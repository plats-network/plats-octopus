use appchain_plats_runtime::{
	currency::{PLT, UNITS},
	opaque::{Block, SessionKeys},
	AccountId, BabeConfig, Balance, BalancesConfig, GenesisConfig, GrandpaConfig, ImOnlineConfig,
	OctopusAppchainConfig, OctopusLposConfig, SessionConfig, Signature, SudoConfig, SystemConfig,
	WASM_BINARY,
};
use beefy_primitives::crypto::AuthorityId as BeefyId;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_octopus_appchain::AuthorityId as OctopusId;
use sc_chain_spec::ChainSpecExtension;
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};

use hex_literal::hex;
use sp_core::crypto::UncheckedInto;

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
/// Octopus testnet generator
pub fn octopus_testnet_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../resources/testnet.json")[..])
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
) -> (AccountId, BabeId, GrandpaId, ImOnlineId, BeefyId, OctopusId) {
	(
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<BeefyId>(seed),
		get_from_seed::<OctopusId>(seed),
	)
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

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
				vec![authority_keys_from_seed("Alice")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				Some(vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
				]),
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		// Extensions
		Default::default(),
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

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
				vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				Some(vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
				]),
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		// Extensions
		Default::default(),
	))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, BabeId, GrandpaId, ImOnlineId, BeefyId, OctopusId)>,
	root_key: AccountId,
	endowed_accounts: Option<Vec<AccountId>>,
	_enable_println: bool,
) -> GenesisConfig {
	let mut endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(|| {
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Charlie"),
			get_account_id_from_seed::<sr25519::Public>("Dave"),
			get_account_id_from_seed::<sr25519::Public>("Eve"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
		]
	});
	// endow all authorities.
	initial_authorities.iter().map(|x| &x.0).for_each(|x| {
		if !endowed_accounts.contains(x) {
			endowed_accounts.push(x.clone())
		}
	});

	let validators = initial_authorities.iter().map(|x| (x.0.clone(), STASH)).collect::<Vec<_>>();

	const ENDOWMENT: Balance = 10_000_000 * PLT;
	const STASH: Balance = 100 * 1_000_000_000_000_000_000; // 100 OCT with 18 decimals

	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|x| (x, ENDOWMENT)).collect(),
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
		babe: BabeConfig {
			authorities: vec![],
			epoch_config: Some(appchain_plats_runtime::BABE_GENESIS_EPOCH_CONFIG),
		},
		im_online: ImOnlineConfig { keys: vec![] },
		grandpa: GrandpaConfig { authorities: vec![] },
		transaction_payment: Default::default(),
		beefy: Default::default(),
		octopus_appchain: OctopusAppchainConfig {
			anchor_contract: "".to_string(),
			asset_id_by_name: vec![("usdc.testnet".to_string(), 0)],
			validators,
			premined_amount: 1024 * PLT,
		},
		octopus_lpos: OctopusLposConfig { era_payout: 2 * PLT, ..Default::default() },
		octopus_assets: Default::default(),
		sudo: SudoConfig {
			// Assign network admin rights.
			key: root_key,
		},
		task: Default::default(),
	}
}

pub fn staging_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Plats Testnet",
		// ID
		"plats_testnet",
		ChainType::Live,
		move || {
			plats_testnet_genesis(
				// WASM Binary
				wasm_binary,
				// Sudo account
				// 5HVgMkXJGoDGQdnTyah4shbhuaiNCmAUdqCyTdYAnr9T9Y1Q
				hex!["beeca6037f5762c78e372d4c6e9ad167b019dac7f16e4cff01d5dab1f242ad2e"].into(),
				// Initial PoA authorities
				vec![
					(
						// 5HNFfkmbzqcDawK6uxeM1oiu8FdioqmtGd4EQSHLaFU3o58b
						hex!["ea8f6d65fce81330127536293c4da64359ebb557f18dfabbefc167cf11d6b238"]
							.into(),
						// 5HNFfkmbzqcDawK6uxeM1oiu8FdioqmtGd4EQSHLaFU3o58b
						hex!["ea8f6d65fce81330127536293c4da64359ebb557f18dfabbefc167cf11d6b238"]
							.unchecked_into(),
						// 5ChNnq5HELidwoQ5wDtH6nZiEzMKuvDQG9qBxVsrxVWrFEdh
						hex!["1bf7e5beb600dfaa913cf5ae3ad7eb903f8774de050e793b6723217aee2a6824"]
							.unchecked_into(),
						// 5HNFfkmbzqcDawK6uxeM1oiu8FdioqmtGd4EQSHLaFU3o58b
						hex!["ea8f6d65fce81330127536293c4da64359ebb557f18dfabbefc167cf11d6b238"]
							.unchecked_into(),
						// KWD8NPNA1Y4MC6BqRtExptdMwLgo637TxFV2fFZ2AaomB4UUp
						hex!["03c34a38eb0c69afd06f21b40ad8e2fc76676fa1b3cdf9ec1ec07695afb3f7019a"]
							.unchecked_into(),
						// 5HNFfkmbzqcDawK6uxeM1oiu8FdioqmtGd4EQSHLaFU3o58b
						hex!["ea8f6d65fce81330127536293c4da64359ebb557f18dfabbefc167cf11d6b238"]
							.unchecked_into(),
					),
					(
						// 5DnhhLqqpY8TgncVhj47UWSTvzbNJ7nbA3zz9o7iTxFNpLTw
						hex!["4c43ffc97cd9b92db7ee3013a2d634bfc79ab779b51af66414929b132d0b8d1d"]
							.into(),
						// 5DnhhLqqpY8TgncVhj47UWSTvzbNJ7nbA3zz9o7iTxFNpLTw
						hex!["4c43ffc97cd9b92db7ee3013a2d634bfc79ab779b51af66414929b132d0b8d1d"]
							.unchecked_into(),
						// 5GvqE5huBZMpLuZrbst6fJjhpKwyxyPgNDaNPqkw5ByHYfia
						hex!["d72c02cd448732fac35c0424647ad08bb9a3610348f5b7e4181fb8be798f84b8"]
							.unchecked_into(),
						// 5DnhhLqqpY8TgncVhj47UWSTvzbNJ7nbA3zz9o7iTxFNpLTw
						hex!["4c43ffc97cd9b92db7ee3013a2d634bfc79ab779b51af66414929b132d0b8d1d"]
							.unchecked_into(),
						// KWAzFTVhzgrvYhVeE2gRKXvP2c7PNutcMLMot7DJwFPmXKwt4
						hex!["0364a077c26cc07d4c2db4abc15ff5fe1eba67f0c3c9aaabc3a22dc11ee1506ae4"]
							.unchecked_into(),
						// 5DnhhLqqpY8TgncVhj47UWSTvzbNJ7nbA3zz9o7iTxFNpLTw
						hex!["4c43ffc97cd9b92db7ee3013a2d634bfc79ab779b51af66414929b132d0b8d1d"]
							.unchecked_into(),
					),
				],
				// Pre-funded accounts
				vec![(
					// 5HVgMkXJGoDGQdnTyah4shbhuaiNCmAUdqCyTdYAnr9T9Y1Q
					hex!["beeca6037f5762c78e372d4c6e9ad167b019dac7f16e4cff01d5dab1f242ad2e"].into(),
					100_000_000 * PLT,
				)],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		Some("plats-staging-testnet"),
		// Properties
		Some(
			serde_json::from_str(
				"{\"tokenDecimals\": 18, \"tokenSymbol\": \"PLT\", \"SS58Prefix\": 42}",
			)
			.expect("Provided valid json map"),
		),
		// Extensions
		Default::default(),
	))
}

/// Configure initial storage state for FRAME modules.
fn plats_testnet_genesis(
	wasm_binary: &[u8],
	root_key: AccountId,
	initial_authorities: Vec<(AccountId, BabeId, GrandpaId, ImOnlineId, BeefyId, OctopusId)>,
	endowed_accounts: Vec<(AccountId, Balance)>,
	_enable_println: bool,
) -> GenesisConfig {
	let validators = initial_authorities.iter().map(|x| (x.0.clone(), STASH)).collect::<Vec<_>>();

	const STASH: Balance = 100 * UNITS; // 100 OCT with 18 decimals

	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|x| (x.0.clone(), x.1)).collect(),
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
		babe: BabeConfig {
			authorities: vec![],
			epoch_config: Some(appchain_plats_runtime::BABE_GENESIS_EPOCH_CONFIG),
		},
		im_online: ImOnlineConfig { keys: vec![] },
		grandpa: GrandpaConfig { authorities: vec![] },
		transaction_payment: Default::default(),
		beefy: Default::default(),
		octopus_appchain: OctopusAppchainConfig {
			anchor_contract: "".to_string(),
			asset_id_by_name: vec![("usdc.testnet".to_string(), 0)],
			validators,
			premined_amount: 1024 * PLT,
		},
		octopus_lpos: OctopusLposConfig { era_payout: 2 * PLT, ..Default::default() },
		octopus_assets: Default::default(),
		sudo: SudoConfig {
			// Assign network admin rights.
			key: root_key,
		},
		task: Default::default(),
	}
}
