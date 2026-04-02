use env_logger::fmt::style::Style;
use env_logger::{Builder, Env};
use log::{Level, error, warn};
use std::env::var;
use std::process::exit;

use deterministic_deployer_evm::client::wallet_client::create_deployers;
use deterministic_deployer_evm::types::config::{ContractSpec, WalletClient, CliArgs, PrecheckResult};
use deterministic_deployer_evm::data::contracts::create_contract_spec_from_args;
use deterministic_deployer_evm::helpers::contract_searcher::resolve_contract;
use deterministic_deployer_evm::helpers::pre_conditions::{check_before, log_info, run_prechecks};
use deterministic_deployer_evm::types::constants::Constants;
use deterministic_deployer_evm::types::errors::CliError;
use deterministic_deployer_evm::utils::create_keystore::load_or_create_keystore;
use deterministic_deployer_evm::utils::deploy::run_deployments;
use deterministic_deployer_evm::utils::read_buf::parse_args;
use deterministic_deployer_evm::utils::verifier::run_verifications;

fn parse_pk() -> String {
    var(Constants::PRIVATE_KEY_ENV).unwrap_or_else(|_| {
        error!(
            "Error: {} environment variable not set",
            Constants::PRIVATE_KEY_ENV
        );
        exit(1);
    })
}

fn init_logger() {
    Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            use std::io::Write;
            let level: log::Level = record.level();
            let level_style: Style = buf.default_level_style(level);
            let style: Style = match level {
                Level::Error | Level::Warn | Level::Debug => buf.default_level_style(level),
                _ => Style::new(),
            };
            writeln!(
                buf,
                "[{} {level_style}{level}{level_style:#} {}] {style}{}{style:#}",
                buf.timestamp(),
                record.module_path().unwrap_or(""),
                record.args(),
            )
        })
        .init();
}

#[tokio::main]
async fn main() {
    init_logger();
    dotenv::dotenv().ok();

    let args: CliArgs = parse_args().unwrap_or_else(|e: CliError| {
        error!("Error: {e}");
        exit(1);
    });

    log_info(&args);

    let private_key: String = if args.keystore {
        load_or_create_keystore().unwrap_or_else(|e| {
            error!("Keystore error: {e}");
            exit(1);
        })
    } else {
        parse_pk()
    };

    let registry_spec: Option<&ContractSpec> = resolve_contract(&args);

    let dynamic_spec: Option<ContractSpec> = if registry_spec.is_some() {
        None
    } else {
        create_contract_spec_from_args(&args)
    };

    let contract_to_deploy: Option<&ContractSpec> = registry_spec.or(dynamic_spec.as_ref());

    check_before(contract_to_deploy, &args);

    let deployers: Vec<WalletClient> = create_deployers(&args.chains, &private_key);

    let Some(spec) = contract_to_deploy.copied() else {
        warn!("No contract specified — nothing to deploy");
        return;
    };

    let PrecheckResult {
        needs_deploy,
        mut ready_for_verify,
    } = run_prechecks(deployers, &spec).await;

    let deployed: Vec<WalletClient> = run_deployments(needs_deploy, spec).await;
    ready_for_verify.extend(deployed);

    if args.verify {
        run_verifications(&ready_for_verify, &spec).await;
    }
}
