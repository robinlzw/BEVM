// Copyright 2023 BEVM Project Authors. Licensed under GPL-3.0.

use crate::{
	chain_spec, service,
	service::{new_partial, FullClient},
	Cli, Subcommand,
};
use frame_benchmarking_cli::*;
use bevm_runtime::{ExistentialDeposit, RuntimeApi};
use bevm_executor::BevmExecutor;
use bevm_primitives::Block;
use sc_cli::{Result, SubstrateCli};
use sc_service::PartialComponents;
use sp_keyring::Sr25519Keyring;

use std::sync::Arc;

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"BEVM Node".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn executable_name() -> String {
		"bevm".into()
	}

	fn description() -> String {
		env!("CARGO_PKG_DESCRIPTION").into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/btclayer2/BEVM/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2023
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		Ok(match id {
			"" => return Err("Please specify which chain you want to run, e.g. --dev".into()),
			"dev" => Box::new(chain_spec::development_config()),
			path =>
				Box::new(chain_spec::BevmChainSpec::from_json_file(std::path::PathBuf::from(path))?),
		})
	}
}

/// Parse command line arguments into service configuration.
pub fn run() -> Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		None => {
			let runner = cli.create_runner(&cli.run)?;
			runner.run_node_until_exit(|config| async move {
				service::new_full(config).map_err(sc_cli::Error::Service)
			})
		},
		Some(Subcommand::Benchmark(_cmd)) => {
			todo!()
			// let runner = cli.create_runner(cmd)?;
			//
			// runner.sync_run(|config| {
			// 	// This switch needs to be in the client, since the client decides
			// 	// which sub-commands it wants to support.
			// 	match cmd {
			// 		BenchmarkCmd::Pallet(cmd) => {
			// 			if !cfg!(feature = "runtime-benchmarks") {
			// 				return Err(
			// 					"Runtime benchmarking wasn't enabled when building the node. \
			// 				You can enable it with `--features runtime-benchmarks`."
			// 						.into(),
			// 				)
			// 			}
			//
			// 			cmd.run::<Block, sp_statement_store::runtime_api::HostFunctions>(config)
			// 		},
			// 		BenchmarkCmd::Block(cmd) => {
			// 			// ensure that we keep the task manager alive
			// 			let partial = new_partial(&config)?;
			// 			cmd.run(partial.client)
			// 		},
			// 		#[cfg(not(feature = "runtime-benchmarks"))]
			// 		BenchmarkCmd::Storage(_) => Err(
			// 			"Storage benchmarking can be enabled with `--features runtime-benchmarks`."
			// 				.into(),
			// 		),
			// 		#[cfg(feature = "runtime-benchmarks")]
			// 		BenchmarkCmd::Storage(cmd) => {
			// 			// ensure that we keep the task manager alive
			// 			let partial = new_partial(&config)?;
			// 			let db = partial.backend.expose_db();
			// 			let storage = partial.backend.expose_storage();
			//
			// 			cmd.run(config, partial.client, db, storage)
			// 		},
			// 		BenchmarkCmd::Overhead(cmd) => {
			// 			// ensure that we keep the task manager alive
			// 			let partial = new_partial(&config)?;
			// 			let ext_builder = RemarkBuilder::new(partial.client.clone());
			//
			// 			cmd.run(
			// 				config,
			// 				partial.client,
			// 				inherent_benchmark_data()?,
			// 				Vec::new(),
			// 				&ext_builder,
			// 			)
			// 		},
			// 	}
			// })
		},
		Some(Subcommand::Key(cmd)) => cmd.run(&cli),
		Some(Subcommand::Sign(cmd)) => cmd.run(),
		Some(Subcommand::Verify(cmd)) => cmd.run(),
		Some(Subcommand::Vanity(cmd)) => cmd.run(),
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		},
		Some(Subcommand::CheckBlock(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, import_queue, .. } =
					new_partial(&config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		},
		Some(Subcommand::ExportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, .. } = new_partial(&config)?;
				Ok((cmd.run(client, config.database), task_manager))
			})
		},
		Some(Subcommand::ExportState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, .. } = new_partial(&config)?;
				Ok((cmd.run(client, config.chain_spec), task_manager))
			})
		},
		Some(Subcommand::ImportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, import_queue, .. } =
					new_partial(&config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		},
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.database))
		},
		Some(Subcommand::Revert(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, backend, .. } =
					new_partial(&config)?;
				let aux_revert = Box::new(|client: Arc<FullClient>, backend, blocks| {
					sc_consensus_babe::revert(client.clone(), backend, blocks)?;
					sc_consensus_grandpa::revert(client, blocks)?;
					Ok(())
				});
				Ok((cmd.run(client, backend, Some(aux_revert)), task_manager))
			})
		},
		#[cfg(feature = "try-runtime")]
		Some(Subcommand::TryRuntime) => Err(try_runtime_cli::DEPRECATION_NOTICE.into()),
		#[cfg(not(feature = "try-runtime"))]
		Some(Subcommand::TryRuntime) => Err("TryRuntime wasn't enabled when building the node. \
				You can enable it with `--features try-runtime`."
			.into()),
		Some(Subcommand::ChainInfo(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run::<Block>(&config))
		},
	}
}
