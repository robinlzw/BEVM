// Copyright 2023 BEVM Project Authors. Licensed under GPL-3.0.

mod chain_spec;
#[macro_use]
mod service;
mod benchmarking;
mod cli;
mod command;
mod rpc;

fn main() -> sc_cli::Result<()> {
	command::run()
}
