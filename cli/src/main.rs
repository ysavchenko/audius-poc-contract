use audius::{
    instruction::{
        clear_valid_signer, init_signer_group, init_valid_signer, validate_signature, SignatureData,
    },
    state::SecpSignatureOffsets,
};
use clap::{
    crate_description, crate_name, crate_version, value_t, value_t_or_exit, App, AppSettings, Arg,
    SubCommand,
};
use hex;
use hex::FromHex;
use solana_clap_utils::{
    input_parsers::pubkey_of,
    input_validators::{is_keypair, is_parsable, is_pubkey, is_url},
    keypair::signer_from_path,
};
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::{
    commitment_config::CommitmentConfig, native_token::lamports_to_sol, signature::Signer,
    transaction::Transaction,
};
use std::process::exit;

struct Config {
    rpc_client: RpcClient,
    verbose: bool,
    owner: Box<dyn Signer>,
    fee_payer: Box<dyn Signer>,
    commitment_config: CommitmentConfig,
}

type Error = Box<dyn std::error::Error>;
type CommandResult = Result<Option<Transaction>, Error>;

fn is_hex(s: String) -> Result<(), String> {
    if hex::decode(s).is_err() {
        return Err(String::from("Wrong address format"));
    } else {
        return Ok(());
    }
}

fn check_fee_payer_balance(config: &Config, required_balance: u64) -> Result<(), Error> {
    let balance = config.rpc_client.get_balance(&config.fee_payer.pubkey())?;
    if balance < required_balance {
        Err(format!(
            "Fee payer, {}, has insufficient balance: {} required, {} available",
            config.fee_payer.pubkey(),
            lamports_to_sol(required_balance),
            lamports_to_sol(balance)
        )
        .into())
    } else {
        Ok(())
    }
}

fn command_init_signer_group(config: &Config, signer_group: &Pubkey) -> CommandResult {
    let mut transaction = Transaction::new_with_payer(
        &[init_signer_group(&audius::id(), signer_group, &config.owner.pubkey()).unwrap()],
        Some(&config.fee_payer.pubkey()),
    );

    let (recent_blockhash, fee_calculator) = config.rpc_client.get_recent_blockhash()?;
    check_fee_payer_balance(config, fee_calculator.calculate_fee(&transaction.message()))?;

    transaction.sign(&[config.fee_payer.as_ref()], recent_blockhash);
    Ok(Some(transaction))
}

fn command_init_valid_signer(
    config: &Config,
    valid_signer: &Pubkey,
    signer_group: &Pubkey,
    eth_address: String,
) -> CommandResult {
    let decoded_address = <[u8; SecpSignatureOffsets::ETH_ADDRESS_SIZE]>::from_hex(eth_address)
        .expect("Ethereum address decoding failed");

    let mut transaction = Transaction::new_with_payer(
        &[init_valid_signer(
            &audius::id(),
            valid_signer,
            signer_group,
            &config.owner.pubkey(),
            decoded_address,
        )
        .unwrap()],
        Some(&config.fee_payer.pubkey()),
    );

    let (recent_blockhash, fee_calculator) = config.rpc_client.get_recent_blockhash()?;
    check_fee_payer_balance(config, fee_calculator.calculate_fee(&transaction.message()))?;

    transaction.sign(
        &[config.fee_payer.as_ref(), config.owner.as_ref()],
        recent_blockhash,
    );
    Ok(Some(transaction))
}

fn command_clear_valid_signer(
    config: &Config,
    valid_signer: &Pubkey,
    signer_group: &Pubkey,
) -> CommandResult {
    let mut transaction = Transaction::new_with_payer(
        &[clear_valid_signer(
            &audius::id(),
            valid_signer,
            signer_group,
            &config.owner.pubkey(),
        )
        .unwrap()],
        Some(&config.fee_payer.pubkey()),
    );

    let (recent_blockhash, fee_calculator) = config.rpc_client.get_recent_blockhash()?;
    check_fee_payer_balance(config, fee_calculator.calculate_fee(&transaction.message()))?;

    transaction.sign(
        &[config.fee_payer.as_ref(), config.owner.as_ref()],
        recent_blockhash,
    );
    Ok(Some(transaction))
}

fn command_validate_signature(
    config: &Config,
    valid_signer: &Pubkey,
    signer_group: &Pubkey,
    signature: String,
    recovery_id: u8,
    message: String,
) -> CommandResult {
    let decoded_signature = <[u8; SecpSignatureOffsets::SECP_SIGNATURE_SIZE]>::from_hex(signature)
        .expect("Secp256k1 signature decoding failed");

    let signature_data = SignatureData {
        signature: decoded_signature,
        recovery_id,
        message: message.as_bytes().to_vec(),
    };

    let mut transaction = Transaction::new_with_payer(
        &[validate_signature(&audius::id(), valid_signer, signer_group, signature_data).unwrap()],
        Some(&config.fee_payer.pubkey()),
    );

    let (recent_blockhash, fee_calculator) = config.rpc_client.get_recent_blockhash()?;
    check_fee_payer_balance(config, fee_calculator.calculate_fee(&transaction.message()))?;

    transaction.sign(&[config.fee_payer.as_ref()], recent_blockhash);
    Ok(Some(transaction))
}

fn main() {
    let matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg({
            let arg = Arg::with_name("config_file")
                .short("C")
                .long("config")
                .value_name("PATH")
                .takes_value(true)
                .global(true)
                .help("Configuration file to use");
            if let Some(ref config_file) = *solana_cli_config::CONFIG_FILE {
                arg.default_value(&config_file)
            } else {
                arg
            }
        })
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .short("v")
                .takes_value(false)
                .global(true)
                .help("Show additional information"),
        )
        .arg(
            Arg::with_name("json_rpc_url")
                .long("url")
                .value_name("URL")
                .takes_value(true)
                .validator(is_url)
                .help("JSON RPC URL for the cluster.  Default from the configuration file."),
        )
        .arg(
            Arg::with_name("owner")
                .long("owner")
                .value_name("KEYPAIR")
                .validator(is_keypair)
                .takes_value(true)
                .help(
                    "Specify the signer group's owner. \
                     This may be a keypair file, the ASK keyword. \
                     Defaults to the client keypair.",
                ),
        )
        .arg(
            Arg::with_name("fee_payer")
                .long("fee-payer")
                .value_name("KEYPAIR")
                .validator(is_keypair)
                .takes_value(true)
                .help(
                    "Specify the fee-payer account. \
                     This may be a keypair file, the ASK keyword. \
                     Defaults to the client keypair.",
                ),
        )
        .subcommand(
            SubCommand::with_name("init-signer-group")
                .about("Create a new signer group")
                .arg(
                    Arg::with_name("signer_group")
                        .long("signer-group")
                        .validator(is_pubkey)
                        .value_name("ADDRESS")
                        .takes_value(true)
                        .required(true)
                        .help("Signer group to be created."),
                ),
        )
        .subcommand(
            SubCommand::with_name("init-valid-signer")
                .about("Add valid signer to the signer group")
                .arg(
                    Arg::with_name("valid_signer")
                        .long("valid-signer-account")
                        .validator(is_pubkey)
                        .value_name("ADDRESS")
                        .takes_value(true)
                        .required(true)
                        .help("Account of valid signer."),
                )
                .arg(
                    Arg::with_name("signer_group")
                        .long("signer-group")
                        .validator(is_pubkey)
                        .value_name("ADDRESS")
                        .takes_value(true)
                        .required(true)
                        .help("Group for Valid Signer to join with."),
                )
                .arg(
                    Arg::with_name("eth_address")
                        .long("ethereum-address")
                        .validator(is_hex)
                        .value_name("ADDRESS")
                        .takes_value(true)
                        .required(true)
                        .help("Ethereum address of valid signer's private key."),
                ),
        )
        .subcommand(
            SubCommand::with_name("clear-valid-signer")
                .about("Remove valid signer from the signer group")
                .arg(
                    Arg::with_name("valid_signer")
                        .long("valid-signer-account")
                        .validator(is_pubkey)
                        .value_name("ADDRESS")
                        .takes_value(true)
                        .required(true)
                        .help("Account of valid signer to be removed."),
                )
                .arg(
                    Arg::with_name("signer_group")
                        .long("signer-group")
                        .validator(is_pubkey)
                        .value_name("ADDRESS")
                        .takes_value(true)
                        .required(true)
                        .help("Signer group to remove from."),
                ),
        )
        .subcommand(
            SubCommand::with_name("validate-signature")
                .about("Validate signer's signature")
                .arg(
                    Arg::with_name("valid_signer")
                        .long("valid-signer-account")
                        .validator(is_pubkey)
                        .value_name("ADDRESS")
                        .takes_value(true)
                        .required(true)
                        .help("Account of valid signer."),
                )
                .arg(
                    Arg::with_name("signer_group")
                        .long("signer-group")
                        .validator(is_pubkey)
                        .value_name("ADDRESS")
                        .takes_value(true)
                        .required(true)
                        .help("Signer group signer belongs to."),
                )
                .arg(
                    Arg::with_name("signature")
                        .long("secp256k1-signature")
                        .validator(is_hex)
                        .value_name("SIGNATURE")
                        .takes_value(true)
                        .required(true)
                        .help("Secp256k1 signature."),
                )
                .arg(
                    Arg::with_name("recovery_id")
                        .long("recovery-id")
                        .validator(is_parsable::<u8>)
                        .value_name("RECOVERY_ID")
                        .takes_value(true)
                        .required(true)
                        .help("Recovery id required to reconstruct address from signature."),
                )
                .arg(
                    Arg::with_name("message")
                        .long("message")
                        .value_name("MESSAGE")
                        .takes_value(true)
                        .required(true)
                        .help("Signed message."),
                ),
        )
        .get_matches();

    let mut wallet_manager = None;
    let config = {
        let cli_config = if let Some(config_file) = matches.value_of("config_file") {
            solana_cli_config::Config::load(config_file).unwrap_or_default()
        } else {
            solana_cli_config::Config::default()
        };
        let json_rpc_url = value_t!(matches, "json_rpc_url", String)
            .unwrap_or_else(|_| cli_config.json_rpc_url.clone());

        let owner = signer_from_path(
            &matches,
            &cli_config.keypair_path,
            "owner",
            &mut wallet_manager,
        )
        .unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            exit(1);
        });
        let fee_payer = signer_from_path(
            &matches,
            &cli_config.keypair_path,
            "fee_payer",
            &mut wallet_manager,
        )
        .unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            exit(1);
        });
        let verbose = matches.is_present("verbose");

        Config {
            rpc_client: RpcClient::new(json_rpc_url),
            verbose,
            owner,
            fee_payer,
            commitment_config: CommitmentConfig::confirmed(),
        }
    };

    solana_logger::setup_with_default("solana=info");

    let _ = match matches.subcommand() {
        ("init-signer-group", Some(arg_matches)) => {
            let signer_group: Pubkey = pubkey_of(arg_matches, "signer_group").unwrap();
            command_init_signer_group(&config, &signer_group)
        }
        ("init-valid-signer", Some(arg_matches)) => {
            let valid_signer: Pubkey = pubkey_of(arg_matches, "valid_signer").unwrap();
            let signer_group: Pubkey = pubkey_of(arg_matches, "signer_group").unwrap();
            let eth_address: String = value_t_or_exit!(arg_matches, "eth_address", String);
            command_init_valid_signer(&config, &valid_signer, &signer_group, eth_address)
        }
        ("clear-valid-signer", Some(arg_matches)) => {
            let valid_signer: Pubkey = pubkey_of(arg_matches, "valid_signer").unwrap();
            let signer_group: Pubkey = pubkey_of(arg_matches, "signer_group").unwrap();
            command_clear_valid_signer(&config, &valid_signer, &signer_group)
        }
        ("validate-signature", Some(arg_matches)) => {
            let valid_signer: Pubkey = pubkey_of(arg_matches, "valid_signer").unwrap();
            let signer_group: Pubkey = pubkey_of(arg_matches, "signer_group").unwrap();
            let signature: String = value_t_or_exit!(arg_matches, "signature", String);
            let recovery_id: u8 = value_t_or_exit!(arg_matches, "recovery_id", u8);
            let message: String = value_t_or_exit!(arg_matches, "message", String);
            command_validate_signature(
                &config,
                &valid_signer,
                &signer_group,
                signature,
                recovery_id,
                message,
            )
        }
        _ => unreachable!(),
    }
    .and_then(|transaction| {
        if let Some(transaction) = transaction {
            let signature = config
                .rpc_client
                .send_and_confirm_transaction_with_spinner_and_commitment(
                    &transaction,
                    config.commitment_config,
                )?;
            println!("Signature: {}", signature);
        }
        Ok(())
    })
    .map_err(|err| {
        eprintln!("{}", err);
        exit(1);
    });
}
