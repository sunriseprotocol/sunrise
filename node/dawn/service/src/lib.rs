// Disable the following lints
#![allow(clippy::type_complexity)]

//! Sunrise service. Specialized wrapper over substrate service.

use cumulus_client_network::build_block_announce_validator;
use cumulus_client_service::{
	prepare_node_config, start_collator, start_full_node, StartCollatorParams, StartFullNodeParams,
};

#[cfg(feature = "with-sunrise-runtime")]
pub use sunrise_runtime;
#[cfg(feature = "with-dawm-runtime")]
pub use dawn_runtime;

use sunrise_primitives::Block;
use polkadot_primitives::v0::CollatorPair;
use sc_executor::native_executor_instance;
use sc_service::{error::Error as ServiceError, Configuration, PartialComponents, Role, TFullClient, TaskManager};
use sp_core::Pair;
use sp_runtime::traits::BlakeTwo256;
use sp_trie::PrefixedMemoryDB;
use std::sync::Arc;

pub use client::*;

pub use sc_executor::NativeExecutionDispatch;
pub use sc_service::{
	config::{DatabaseConfig, PrometheusConfig},
	ChainSpec,
};
pub use sp_api::ConstructRuntimeApi;

pub mod chain_spec;
mod client;
mod mock_timestamp_data_provider;

#[cfg(feature = "with-dawn-runtime")]
native_executor_instance!(
	pub DawnExecutor,
	dawn_runtime::api::dispatch,
	dawn_runtime::native_version,
	frame_benchmarking::benchmarking::HostFunctions,
);

#[cfg(feature = "with-sunrise-runtime")]
native_executor_instance!(
	pub SunriseExecutor,
	sunrise_runtime::api::dispatch,
	sunrise_runtime::native_version,
	frame_benchmarking::benchmarking::HostFunctions,
);

/// Can be called for a `Configuration` to check if it is a configuration for
/// the `Sunrise` network.
pub trait IdentifyVariant {
	/// Returns if this is a configuration for the `Sunrise` network.
	fn is_sunrise(&self) -> bool;

	/// Returns if this is a configuration for the `Dawn` network.
	fn is_dawn(&self) -> bool;
}

impl IdentifyVariant for Box<dyn ChainSpec> {
	fn is_sunrise(&self) -> bool {
		self.id().starts_with("sunrise") || self.id().starts_with("aca")
	}

	fn is_dawn(&self) -> bool {
		self.id().starts_with("dawn") || self.id().starts_with("man")
	}
}

/// Sunrise's full backend.
type FullBackend = sc_service::TFullBackend<Block>;

/// Sunrise's full client.
type FullClient<RuntimeApi, Executor> = sc_service::TFullClient<Block, RuntimeApi, Executor>;

pub fn new_partial<RuntimeApi, Executor>(
	config: &Configuration,
	_test: bool,
) -> Result<
	PartialComponents<
		FullClient<RuntimeApi, Executor>,
		FullBackend,
		(),
		sp_consensus::import_queue::BasicQueue<Block, PrefixedMemoryDB<BlakeTwo256>>,
		sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi, Executor>>,
		(),
	>,
	sc_service::Error,
>
where
	RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
	Executor: NativeExecutionDispatch + 'static,
{
	let inherent_data_providers = sp_inherents::InherentDataProviders::new();

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, Executor>(&config)?;
	let client = Arc::new(client);

	// TODO: custom registry with `sunrise` prefix?
	let registry = config.prometheus_registry();

	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_handle(),
		client.clone(),
	);

	let import_queue = cumulus_client_consensus::import_queue::import_queue(
		client.clone(),
		client.clone(),
		inherent_data_providers.clone(),
		&task_manager.spawn_handle(),
		registry.clone(),
	)?;

	Ok(PartialComponents {
		backend,
		client,
		import_queue,
		keystore_container,
		task_manager,
		transaction_pool,
		inherent_data_providers,
		select_chain: (),
		other: (),
	})
}

/// Start a node with the given parachain `Configuration` and relay chain
/// `Configuration`.
///
/// This is the actual implementation that is abstract over the executor and the
/// runtime api.
#[sc_tracing::logging::prefix_logs_with("Parachain")]
async fn start_node_impl<RuntimeApi, Executor>(
	parachain_config: Configuration,
	collator_key: CollatorPair,
	polkadot_config: Configuration,
	id: polkadot_primitives::v0::Id,
	validator: bool,
) -> sc_service::error::Result<(TaskManager, Arc<TFullClient<Block, RuntimeApi, Executor>>)>
where
	RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
	Executor: NativeExecutionDispatch + 'static,
{
	if matches!(parachain_config.role, Role::Light) {
		return Err("Light client not supported!".into());
	}

	let parachain_config = prepare_node_config(parachain_config);

	let polkadot_full_node = cumulus_client_service::build_polkadot_full_node(polkadot_config, collator_key.public())
		.map_err(|e| match e {
		polkadot_service::Error::Sub(x) => x,
		s => format!("{}", s).into(),
	})?;

	let params = new_partial(&parachain_config, false)?;
	params
		.inherent_data_providers
		.register_provider(sp_timestamp::InherentDataProvider)
		.unwrap();

	let client = params.client.clone();
	let backend = params.backend.clone();
	let block_announce_validator = build_block_announce_validator(
		polkadot_full_node.client.clone(),
		id,
		Box::new(polkadot_full_node.network.clone()),
		polkadot_full_node.backend.clone(),
	);

	let prometheus_registry = parachain_config.prometheus_registry().cloned();
	let transaction_pool = params.transaction_pool.clone();
	let mut task_manager = params.task_manager;
	let import_queue = params.import_queue;
	let (network, network_status_sinks, system_rpc_tx, start_network) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &parachain_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			on_demand: None,
			block_announce_validator_builder: Some(Box::new(|_| block_announce_validator)),
		})?;

	let rpc_extensions_builder = {
		let client = client.clone();
		let transaction_pool = transaction_pool.clone();

		Box::new(move |deny_unsafe, _| -> sunrise_rpc::RpcExtension {
			let deps = sunrise_rpc::FullDeps {
				client: client.clone(),
				pool: transaction_pool.clone(),
				deny_unsafe,
			};

			sunrise_rpc::create_full(deps)
		})
	};

	if parachain_config.offchain_worker.enabled {
		sc_service::build_offchain_workers(
			&parachain_config,
			backend.clone(),
			task_manager.spawn_handle(),
			client.clone(),
			network.clone(),
		);
	};

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		on_demand: None,
		remote_blockchain: None,
		rpc_extensions_builder,
		client: client.clone(),
		transaction_pool: transaction_pool.clone(),
		task_manager: &mut task_manager,
		config: parachain_config,
		keystore: params.keystore_container.sync_keystore(),
		backend: backend.clone(),
		network: network.clone(),
		network_status_sinks,
		system_rpc_tx,
	})?;

	let announce_block = {
		let network = network.clone();
		Arc::new(move |hash, data| network.announce_block(hash, Some(data)))
	};

	if validator {
		let proposer_factory = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool,
			prometheus_registry.as_ref(),
		);
		let spawner = task_manager.spawn_handle();

		let polkadot_backend = polkadot_full_node.backend.clone();

		let params = StartCollatorParams {
			para_id: id,
			block_import: client.clone(),
			proposer_factory,
			inherent_data_providers: params.inherent_data_providers,
			block_status: client.clone(),
			announce_block,
			client: client.clone(),
			task_manager: &mut task_manager,
			collator_key,
			polkadot_full_node,
			spawner,
			backend,
			polkadot_backend,
		};

		start_collator(params).await?;
	} else {
		let params = StartFullNodeParams {
			client: client.clone(),
			announce_block,
			task_manager: &mut task_manager,
			para_id: id,
			polkadot_full_node,
		};

		start_full_node(params)?;
	}

	start_network.start_network();

	Ok((task_manager, client))
}

/// Start a normal parachain node.
pub async fn start_node<RuntimeApi, Executor>(
	parachain_config: Configuration,
	collator_key: CollatorPair,
	polkadot_config: Configuration,
	id: polkadot_primitives::v0::Id,
	validator: bool,
) -> sc_service::error::Result<(TaskManager, Arc<TFullClient<Block, RuntimeApi, Executor>>)>
where
	RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
	Executor: NativeExecutionDispatch + 'static,
{
	start_node_impl(parachain_config, collator_key, polkadot_config, id, validator).await
}

/// Builds a new object suitable for chain operations.
pub fn new_chain_ops(
	mut config: &mut Configuration,
) -> Result<
	(
		Arc<Client>,
		Arc<FullBackend>,
		sp_consensus::import_queue::BasicQueue<Block, PrefixedMemoryDB<BlakeTwo256>>,
		TaskManager,
	),
	ServiceError,
> {
	config.keystore = sc_service::config::KeystoreConfig::InMemory;
	if config.chain_spec.is_dawn() {
		#[cfg(feature = "with-dawn-runtime")]
		{
			let PartialComponents {
				client,
				backend,
				import_queue,
				task_manager,
				..
			} = new_partial::<dawn_runtime::RuntimeApi, DawnExecutor>(config, false)?;
			Ok((Arc::new(Client::Dawn(client)), backend, import_queue, task_manager))
		}
		#[cfg(not(feature = "with-dawn-runtime"))]
		Err("Dawn runtime is not available. Please compile the node with `--features with-dawn-runtime` to enable it.".into())
	} else {
		#[cfg(feature = "with-sunrise-runtime")]
		{
			let PartialComponents {
				client,
				backend,
				import_queue,
				task_manager,
				..
			} = new_partial::<sunrise_runtime::RuntimeApi, SunriseExecutor>(config, false)?;
			Ok((Arc::new(Client::Sunrise(client)), backend, import_queue, task_manager))
		}
		#[cfg(not(feature = "with-sunrise-runtime"))]
		Err("Sunrise runtime is not available. Please compile the node with `--features with-sunrise-runtime` to enable it.".into())
	}
}