use appchain_plats_runtime::{
	currency::{PLAT, UNITS},
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
		Some(
			serde_json::from_str(
				"{\"tokenDecimals\": 18, \"tokenSymbol\": \"PLAT\", \"SS58Prefix\": 42}",
			)
			.expect("Provided valid json map"),
		),
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

	const ENDOWMENT: Balance = 10_000_000 * PLAT;
	const STASH: Balance = 100 * UNITS; // 100 OCT with 18 decimals

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
			anchor_contract: "plats_network.near".to_string(),
			asset_id_by_name: vec![("usdc.testnet".to_string(), 0)],
			validators,
			premined_amount: 30_000_000 * PLAT,
		},
		octopus_lpos: OctopusLposConfig { era_payout: 34_246 * PLAT, ..Default::default() },
		octopus_assets: Default::default(),
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
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
				hex!["16c1e8c292b0ca968ee84d3f33de819dd3d1466ee4a5a025c4c714582e29fa26"].into(),
				// Initial PoA authorities
				vec![
					(
						hex!["622205c8f5b65af9a815c799af6fec1a866305a8d2a7821c7201a7f5150e4648"]
							.into(),
						// 5HNFfkmbzqcDawK6uxeM1oiu8FdioqmtGd4EQSHLaFU3o58b
						//BABE
						hex!["729bdde7aa98e9c8ea40a6e2998e290070c9fc40be783ff91cd670fcd66b375b"]
							.unchecked_into(),
						// 5ChNnq5HELidwoQ5wDtH6nZiEzMKuvDQG9qBxVsrxVWrFEdh
						//GRANPA
						hex!["62e6d42eb7054765973e10532746c3a478516b039bbc0970c006a3b51d957da3"]
							.unchecked_into(),
						// 5HNFfkmbzqcDawK6uxeM1oiu8FdioqmtGd4EQSHLaFU3o58b
						// IMONLINE
						hex!["1c05b439a50f231637caa475ad3e9359e2fdc8fd9e255f6f15c4dfac9d810b3a"]
							.unchecked_into(),
						// KWD8NPNA1Y4MC6BqRtExptdMwLgo637TxFV2fFZ2AaomB4UUp
						//BEEFY
						hex!["03fdec01e54a89a132d06b7ec4ceb248b3dee2b4f9e52e1dddf08e197c8ec1c1fd"]
							.unchecked_into(),
						// 5HNFfkmbzqcDawK6uxeM1oiu8FdioqmtGd4EQSHLaFU3o58b
						// OCTOPUS
						hex!["c40c3964691437304fde5e8f4e5fb58eb64f896d553328d58a0b54fc43678048"]
							.unchecked_into(),
					),
					(
						hex!["244a8d85303f048feaab2413d510c175f48e0ca8b2f73bc7ea261e42a4ca2334"]
							.into(),
						//BABE
						hex!["4e4c44263a0fc7c3d1ae0302920517246c075c6fb1218471873067413392c22b"]
							.unchecked_into(),
						// 5GvqE5huBZMpLuZrbst6fJjhpKwyxyPgNDaNPqkw5ByHYfia
						// GRANPA
						hex!["76c30dbf7dafd87bbe4d88f74fd45837832acb5553f60dc755845c0fc9a6bd7a"]
							.unchecked_into(),
						//IMONLINE
						hex!["9c9ff8a7d4c357e80648ec345d9d6c61f9dc16d0871a98743515433b45a71e47"]
							.unchecked_into(),
						// KWAzFTVhzgrvYhVeE2gRKXvP2c7PNutcMLMot7DJwFPmXKwt4
						//BEEFY
						hex!["03aad9b8428e57ac66303753569aca489120114105069959d410a90d256e8941a5"]
							.unchecked_into(),
						//OCT
						hex!["b654047ad9a94f104b8689f9f074c22bc06ee7f524a8a21cdc221ce311884b44"]
							.unchecked_into(),
					),
					(
						hex!["7c0040a6861a4592fb16fddd7de31a193a7daa3cb70bc84461373a80d5483814"]
							.into(),
						//BABE
						hex!["046933611026e8c51ea2fe75460ebf6f61103ab12bca9b096151ef1a3a529366"]
							.unchecked_into(),
						// 5GvqE5huBZMpLuZrbst6fJjhpKwyxyPgNDaNPqkw5ByHYfia
						// GRANPA
						hex!["2a516dfed3802d9fe46dc1b838a7c3fe0a52f8cb9346b8abab7ad7e99af82619"]
							.unchecked_into(),
						//IMONLINE
						hex!["28b2be9df1fe95b6b8966f5c88bf72c4a232baedb51f681aa65a33e61c454806"]
							.unchecked_into(),
						// KWAzFTVhzgrvYhVeE2gRKXvP2c7PNutcMLMot7DJwFPmXKwt4
						//BEEFY
						hex!["03682e0b13343b35f3c4d8536532f7bb39074f17943a51380d306b3ec80cbd7b4f"]
							.unchecked_into(),
						//OCT
						hex!["f6a0e5c18b33896017573e2c4c1baa9ff43d1d20b5a95d0e0f911ab9bb653566"]
							.unchecked_into(),
					),
					(
						hex!["0cd12803946630752e87cd472e695a69aaac023808bb5f932a52022d2a04047a"]
							.into(),
						//BABE
						hex!["62ebd8f4349d24f75b1ce8a9a4981ea62d2e2507bb689cc2da60fb03b5233f3d"]
							.unchecked_into(),
						// 5GvqE5huBZMpLuZrbst6fJjhpKwyxyPgNDaNPqkw5ByHYfia
						// GRANPA
						hex!["eedfe754dbe5d14765b911ea46b730fd3fe73c1ef2e71f1b2b4b90a11a82a240"]
							.unchecked_into(),
						//IMONLINE
						hex!["54e3962cd6f70d30aaf482c7a6e7704266157e52ab3735667cafef391367ef6e"]
							.unchecked_into(),
						// KWAzFTVhzgrvYhVeE2gRKXvP2c7PNutcMLMot7DJwFPmXKwt4
						//BEEFY
						hex!["02bcf03090f289f1cfbb45eb3809e13920d91a1033288cf840a69a39b52228f24b"]
							.unchecked_into(),
						//OCT
						hex!["aa6a10d892802063564801ecb37a10b011328b0f478085749107a94602b5ce1b"]
							.unchecked_into(),
					),
				],
				// Pre-funded accounts
				vec![
					(
						// 5HVgMkXJGoDGQdnTyah4shbhuaiNCmAUdqCyTdYAnr9T9Y1Q
						hex!["1abbfa43b14065a01ac7f9250b07977000c84a7caf97fbab5f26a9f21f49554f"]
							.into(),
						100_000_000 * PLAT,
					),
					(
						hex!["16c1e8c292b0ca968ee84d3f33de819dd3d1466ee4a5a025c4c714582e29fa26"]
							.into(),
						100_000_000 * PLAT,
					),
					(
						hex!["622205c8f5b65af9a815c799af6fec1a866305a8d2a7821c7201a7f5150e4648"]
							.into(),
						10 * PLAT,
					),
					(
						hex!["244a8d85303f048feaab2413d510c175f48e0ca8b2f73bc7ea261e42a4ca2334"]
							.into(),
						10 * PLAT,
					),
					(
						hex!["7c0040a6861a4592fb16fddd7de31a193a7daa3cb70bc84461373a80d5483814"]
							.into(),
						10 * PLAT,
					),
					(
						hex!["0cd12803946630752e87cd472e695a69aaac023808bb5f932a52022d2a04047a"]
							.into(),
						10 * PLAT,
					),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		Some("plats-staging-testnet"),
		None,
		// Properties
		Some(
			serde_json::from_str(
				"{\"tokenDecimals\": 18, \"tokenSymbol\": \"PLAT\", \"SS58Prefix\": 42}",
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

	const STASH: Balance = 10_000 * UNITS; // 10000 OCT with 18 decimals

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
			anchor_contract: "plats-network.registry.test_oct.testnet".to_string(),
			asset_id_by_name: vec![("usdc.testnet".to_string(), 0)],
			validators,
			premined_amount: 30_000_000 * PLAT,
		},
		octopus_lpos: OctopusLposConfig { era_payout: 34_246 * PLAT, ..Default::default() },
		octopus_assets: Default::default(),
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
		task: Default::default(),
	}
}
