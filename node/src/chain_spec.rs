use appchain_plats_runtime::{
	currency::PLT,
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
use serde_json::json;
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
				hex!["f03941f93b990c271015d3b485f137e117aab80af0a03b557966927caaa7d44f"].into(),
				// Initial PoA authorities
				vec![
					(
						// 5GhTbhujpv3nZQx6idibYSwYeNCN7ddpqqjPjwZn43xdvYMT
						hex!["ccf90463ce9ae4cf881c549b09ddeac1960316930e390ca47eeba95741386e5b"]
							.into(),
						// 5GhTbhujpv3nZQx6idibYSwYeNCN7ddpqqjPjwZn43xdvYMT
						hex!["ccf90463ce9ae4cf881c549b09ddeac1960316930e390ca47eeba95741386e5b"]
							.unchecked_into(),
						// 5G5ghjBD9fkx9gR59LQLmQvFnayjaRhdBKqpujvNjYjmx4ks
						hex!["b1b04b436a8772b6429a549ae68d72fd88b8533462d03d83d9acaf9500b3ca00"]
							.unchecked_into(),
						// 5GhTbhujpv3nZQx6idibYSwYeNCN7ddpqqjPjwZn43xdvYMT
						hex!["ccf90463ce9ae4cf881c549b09ddeac1960316930e390ca47eeba95741386e5b"]
							.unchecked_into(),
						// KW8mwncjSVKxsCbACjDDk2bLHLsq2gkeVw5xjKW4vSLgWimn1
						hex!["0302c5928b0861672271346c29e30faa2cb5328e024d1c45f2689e886cb12b6de1"]
							.unchecked_into(),
						// 5GhTbhujpv3nZQx6idibYSwYeNCN7ddpqqjPjwZn43xdvYMT
						hex!["ccf90463ce9ae4cf881c549b09ddeac1960316930e390ca47eeba95741386e5b"]
							.unchecked_into(),
					),
					(
						// 5H9RP9sy2g9Jaj1GG2zGaytLdxoBHQnqMaKmqvtFPJpYiRV3
						hex!["e0c5efc09df70c2e236e32ebba4c89a5ae538dacf25412e2a23e6a175291453a"]
							.into(),
						// 5H9RP9sy2g9Jaj1GG2zGaytLdxoBHQnqMaKmqvtFPJpYiRV3
						hex!["e0c5efc09df70c2e236e32ebba4c89a5ae538dacf25412e2a23e6a175291453a"]
							.unchecked_into(),
						// 5Dvf9Qq8rmfFdSLACJwvcDEYJMYYq6wYiKkazZrUmWLqUDEE
						hex!["52556063e8c72431f643c8eb66ba172d5b0d2a095429a8a6e29b522208e26ccd"]
							.unchecked_into(),
						// 5H9RP9sy2g9Jaj1GG2zGaytLdxoBHQnqMaKmqvtFPJpYiRV3
						hex!["e0c5efc09df70c2e236e32ebba4c89a5ae538dacf25412e2a23e6a175291453a"]
							.unchecked_into(),
						// KW9uY45eZ65PpHxk21KiXvc8XiTse6amUPKpAWgvxmfhorryw
						hex!["0334cbe01d6db7bf3d0f4148c468a3a01a5a560f21244d9891c35de23d7c752c24"]
							.unchecked_into(),
						// 5H9RP9sy2g9Jaj1GG2zGaytLdxoBHQnqMaKmqvtFPJpYiRV3
						hex!["e0c5efc09df70c2e236e32ebba4c89a5ae538dacf25412e2a23e6a175291453a"]
							.unchecked_into(),
					),
				],
				// Pre-funded accounts
				vec![(
					// 5HVgMkXJGoDGQdnTyah4shbhuaiNCmAUdqCyTdYAnr9T9Y1Q
					hex!["f03941f93b990c271015d3b485f137e117aab80af0a03b557966927caaa7d44f"].into(),
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
				"{\"tokenDecimals\": 12, \"tokenSymbol\": \"PLT\", \"SS58Prefix\": 42}",
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

	const STASH: Balance = 100 * 1_000_000_000_000_000_000; // 100 OCT with 18 decimals

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
	}
}
