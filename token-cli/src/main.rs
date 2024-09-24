use std::error::Error;

use clap::{value_parser, Arg, Command};
macro_rules! get_program_id {
    ($netype:ident) => {
        match $netype {
            "test" => solana_program::pubkey!("9zuZUTkJBdrp6zZ6uEDjVjUHZT18rKcjchdqQYqyiy4C"),
            _ => panic!("unknown net type {}", $netype),
        }
    };
}
macro_rules! args_init {
    () => {
        [
            Arg::new("url").long("url").required(true),
            Arg::new("network").long("network").required(true),
        ]
    };
}
fn main() -> Result<(), Box<dyn Error>> {
    let cmd = Command::new("token-cli").subcommands([
        airdrop().args(args_init!()),
        balance().args(args_init!()),
        create_account().args(args_init!()),
        transfer().args(args_init!()),
        create_log_book_account().args(args_init!()),
    ]);
    let matches = cmd.get_matches();

    match matches.subcommand() {
        Some(("airdrop", am)) => cmd::airdrop(am),
        Some(("balance", am)) => cmd::balance(am),
        Some(("create-account", am)) => cmd::create_account(am),
        Some(("transfer", am)) => cmd::transfer(am),
        Some(("create-log-book-account", am)) => cmd::create_log_book_account(am),
        _ => Err(Box::new(token_cli::error::Error("not exist command"))),
    }
}
fn airdrop() -> Command {
    Command::new("airdrop")
        .arg(
            Arg::new("Address")
                .value_parser(value_parser!(String))
                .required(true),
        )
        .arg(
            Arg::new("Amount")
                .value_parser(value_parser!(u64))
                .required(true),
        )
}
fn balance() -> Command {
    Command::new("balance").arg(
        Arg::new("Address")
            .value_parser(value_parser!(String))
            .required(true),
    )
}
fn create_account() -> Command {
    Command::new("create-account").arg(
        Arg::new("Address")
            .value_parser(value_parser!(String))
            .required(true),
    )
}
fn transfer() -> Command {
    Command::new("transfer")
        .arg(
            Arg::new("Address")
                .value_parser(value_parser!(String))
                .required(true),
        )
        .arg(
            Arg::new("Amount")
                .value_parser(value_parser!(u64))
                .required(true),
        )
        .arg(
            Arg::new("Dest_Address")
                .value_parser(value_parser!(String))
                .required(true),
        )
}
fn create_log_book_account() -> Command {
    Command::new("create-log-book-account").arg(
        Arg::new("Address")
            .value_parser(value_parser!(String))
            .required(true),
    )
}
mod cmd {
    use std::{error::Error, str::FromStr};

    use clap::ArgMatches;
    use solana_sdk::{signature::Keypair, signer::Signer};
    use token_cli::logs;

    pub fn airdrop(arg_matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
        let address = arg_matches.get_one::<String>("Address").unwrap();
        let address = Keypair::from_base58_string(&address);
        let amount = arg_matches.get_one::<u64>("Amount").unwrap();
        let rpc_url = arg_matches.get_one::<String>("url").unwrap();
        let netype = arg_matches.get_one::<String>("network").unwrap();
        let nt = netype.as_str();
        let program_id = get_program_id!(nt);
        let pubkey = address.pubkey();
        let ix = token_cli::airdrop(program_id, &pubkey, *amount);
        let rpc_client = solana_client::rpc_client::RpcClient::new(rpc_url);
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[ix],
            Some(&pubkey),
            &[&address],
            rpc_client.get_latest_blockhash()?,
        );
        let sig = rpc_client.send_and_confirm_transaction(&tx)?;
        println!("{}", sig.to_string());
        Ok(())
    }
    pub fn balance(arg_matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
        let address = arg_matches.get_one::<String>("Address").unwrap();
        let pubkey = solana_program::pubkey::Pubkey::from_str(&address)?;
        let rpc_url = arg_matches.get_one::<String>("url").unwrap();
        let netype = arg_matches.get_one::<String>("network").unwrap();
        let nt = netype.as_str();
        let program_id = get_program_id!(nt);
        let rpc_client = solana_client::rpc_client::RpcClient::new(rpc_url);
        let balance = token_cli::get_balance(program_id, &pubkey, &rpc_client)?;
        println!("{balance}");
        Ok(())
    }
    pub fn create_account(arg_matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
        let address = arg_matches.get_one::<String>("Address").unwrap();
        let address = Keypair::from_base58_string(&address);
        let rpc_url = arg_matches.get_one::<String>("url").unwrap();
        let netype = arg_matches.get_one::<String>("network").unwrap();
        let nt = netype.as_str();
        let program_id = get_program_id!(nt);
        let pubkey = address.pubkey();
        let rpc_client = solana_client::rpc_client::RpcClient::new(rpc_url);
        let ix = token_cli::init_account(program_id, &pubkey);
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[ix],
            Some(&pubkey),
            &[&address],
            rpc_client.get_latest_blockhash()?,
        );
        let sig = rpc_client.send_and_confirm_transaction(&tx)?;
        println!("{}", sig.to_string());
        Ok(())
    }
    pub fn transfer(arg_matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
        let address = arg_matches.get_one::<String>("Address").unwrap();
        let address = Keypair::from_base58_string(&address);
        let amount = arg_matches.get_one::<u64>("Amount").unwrap();
        let dest_address = arg_matches.get_one::<String>("Dest_Address").unwrap();
        let dest = solana_program::pubkey::Pubkey::from_str(&dest_address)?;
        let rpc_url = arg_matches.get_one::<String>("url").unwrap();
        let rpc_client = solana_client::rpc_client::RpcClient::new(rpc_url);
        let netype = arg_matches.get_one::<String>("network").unwrap();
        let nt = netype.as_str();
        let program_id = get_program_id!(nt);
        let pubkey = address.pubkey();
        let ix = token_cli::transfer(program_id, &pubkey, &dest, *amount);
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[ix],
            Some(&pubkey),
            &[&address],
            rpc_client.get_latest_blockhash()?,
        );
        let sig = rpc_client.send_and_confirm_transaction(&tx)?;
        println!("{}", sig.to_string());
        Ok(())
    }
    pub fn create_log_book_account(arg_matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
        let address = arg_matches.get_one::<String>("Address").unwrap();
        let address = Keypair::from_base58_string(&address);
        let netype = arg_matches.get_one::<String>("network").unwrap();
        let nt = netype.as_str();
        let program_id = get_program_id!(nt);
        let payer = address.pubkey();
        let (book, ix) = logs::init_log_book_account(&program_id, &payer);
        let rpc_url = arg_matches.get_one::<String>("url").unwrap();
        let rpc_client = solana_client::rpc_client::RpcClient::new(rpc_url);
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer),
            &[address, book],
            rpc_client.get_latest_blockhash()?,
        );
        let sig = rpc_client.send_transaction(&tx)?;
        println!("{}", sig.to_string());
        Ok(())
    }
}
