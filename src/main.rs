use mini_blockchain::{
    config,
    transaction::{PublicKey, Transaction},
    wallet::Wallet,
};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use comfy_table::{presets::UTF8_FULL, Table};
use p256::ecdsa::VerifyingKey;

#[derive(Parser, Debug)]
#[command(name = "mini-blockchain", version, about = "A fun little blockchain, written in Rust, now with all the bells and whistles!")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum WalletCommands {
    New { name: String },
    List,
    Use { name: String },
}

#[derive(Subcommand, Debug)]
enum ContactCommands {
    Add { name: String, address: String },
    List,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(subcommand)]
    Wallet(WalletCommands),
    #[command(subcommand)]
    Contact(ContactCommands),
    AddTx {
        #[arg(short, long)]
        receiver: String,
        #[arg(short, long)]
        amount: u64,
    },
    Mine,
    Balance {
        #[arg(short, long)]
        address: Option<String>,
    },
    Pending,
    List,
    Validate,
    Clear,
}

fn main() -> Result<()> {
    let mut state = config::load_app_state()?;
    let cli = Cli::parse();
    let mut state_changed = false;

    match cli.command {
        Commands::Wallet(wallet_cmd) => {
            state_changed = true;
            match wallet_cmd {
                WalletCommands::New { name } => {
                    let wallet = Wallet::new();
                    let address = hex::encode(wallet.public_key.to_encoded_point(true));
                    config::save_wallet(&name, &wallet)?;
                    println!("{} New wallet '{}' created.", "[SUCCESS]".green(), name.bold());
                    println!("   Your public address is: {}", address.cyan());
                    if state.config.active_wallet.is_none() {
                        state.config.active_wallet = Some(name.clone());
                        println!("{} This has been set as your active wallet.", "[INFO]".cyan());
                    }
                }
                WalletCommands::List => {
                    state_changed = false;
                    let wallets = config::get_all_wallets()?;
                    let mut table = Table::new();
                    table.set_header(vec!["Active", "Name", "Public Address"]);
                    for (name, address) in wallets {
                        let is_active = if state.config.active_wallet.as_deref() == Some(&name) {
                            "*".green().to_string()
                        } else {
                            "".to_string()
                        };
                        table.add_row(vec![is_active, name.bold().to_string(), address]);
                    }
                    println!("{}", table);
                }
                WalletCommands::Use { name } => {
                    config::load_wallet(&name)?;
                    state.config.active_wallet = Some(name.clone());
                    println!(
                        "{} Your active wallet is now '{}'.",
                        "[SUCCESS]".green(),
                        name.bold()
                    );
                }
            }
        }
        Commands::Contact(contact_cmd) => {
            state_changed = true;
            match contact_cmd {
                ContactCommands::Add { name, address } => {
                    state.contacts.insert(name.clone(), address);
                    println!("{} Contact '{}' saved.", "[SUCCESS]".green(), name.bold());
                }
                ContactCommands::List => {
                    state_changed = false;
                    let mut table = Table::new();
                    table.set_header(vec!["Nickname", "Address"]);
                    for (name, address) in &state.contacts {
                        table.add_row(vec![name.bold().to_string(), address.to_string()]);
                    }
                    println!("{}", table);
                }
            }
        }
        Commands::AddTx { receiver, amount } => {
            let active_wallet_name = state.config.active_wallet.clone().context(
                "You don't have an active wallet. Use `wallet use <name>` to set one.",
            )?;
            let wallet = config::load_wallet(&active_wallet_name)?;

            let final_receiver_addr = state.contacts.get(&receiver).unwrap_or(&receiver);

            let receiver_pk_bytes =
                hex::decode(final_receiver_addr).context("The receiver's address isn't valid hex.")?;
            let receiver_pk = VerifyingKey::from_sec1_bytes(&receiver_pk_bytes)
                .context("That's not a valid public key.")?;

            let tx = Transaction::new(&wallet, PublicKey(receiver_pk), amount);
            state.blockchain.add_transaction(tx)?;
            state_changed = true;
            println!(
                "{} Transaction added to the mempool. It'll be in the next block.",
                "[SUCCESS]".green()
            );
        }
        Commands::Mine => {
            let active_wallet_name = state.config.active_wallet.clone()
                .context("You need an active wallet to receive the mining reward!")?;
            let wallet = config::load_wallet(&active_wallet_name)?;

            println!("[INFO] Starting the miner... This might take a moment.");
            state
                .blockchain
                .mine_pending_transactions(PublicKey(wallet.public_key))?;
            state_changed = true;
            println!(
                "{} A new block has been successfully mined!",
                "[SUCCESS]".green()
            );
        }
        Commands::Balance { address } => {
            let target_address_str = match address {
                Some(addr) => state.contacts.get(&addr).cloned().unwrap_or(addr),
                None => {
                    let active_wallet_name = state.config.active_wallet.as_ref()
                        .context("No active wallet. Specify an address with `-a <address>`.")?;
                    let wallet = config::load_wallet(active_wallet_name)?;
                    hex::encode(wallet.public_key.to_encoded_point(true))
                }
            };

            let pk_bytes = hex::decode(&target_address_str)?;
            let public_key = VerifyingKey::from_sec1_bytes(&pk_bytes)?;
            let balance = state.blockchain.get_balance(&PublicKey(public_key));
            println!(
                "Balance for {}: {} coins.",
                target_address_str.yellow(),
                balance.to_string().bold()
            );
        }
        Commands::Pending => {
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .set_header(vec!["From", "To", "Amount"]);
            if state.blockchain.mempool.is_empty() {
                println!("{}", "The mempool is currently empty. No pending transactions.".italic());
            } else {
                for tx in &state.blockchain.mempool {
                    let from = tx.source.as_ref().map(|s| hex::encode(s.0.to_encoded_point(true))).unwrap_or_else(|| "COINBASE".to_string());
                    let to = hex::encode(tx.destination.0.to_encoded_point(true));
                    table.add_row(vec![
                        format!("{}...", &from[..10]),
                        format!("{}...", &to[..10]),
                        tx.amount.to_string().green().to_string(),
                    ]);
                }
                println!("Pending Transactions in the Mempool:\n{}", table);
            }
        }
        Commands::List => {
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .set_header(vec!["Index", "Hash", "# Txs", "Difficulty"]);
            for block in &state.blockchain.chain {
                table.add_row(vec![
                    block.index.to_string().cyan().to_string(),
                    format!("{}...", &block.hash[..10]),
                    block.transactions.len().to_string().yellow().to_string(),
                    block.difficulty.to_string(),
                ]);
            }
            println!("Full Blockchain History:\n{}", table);
        }
        Commands::Validate => {
            if state.blockchain.is_chain_valid() {
                println!(
                    "{} The blockchain is valid and its integrity is intact!",
                    "[VALID]".green()
                );
            } else {
                println!(
                    "{} DANGER: The blockchain has been tampered with or is corrupted!",
                    "[INVALID]".red()
                );
            }
        }
        Commands::Clear => {
            println!("{}", "This will delete ALL your data (wallets, contacts, blockchain). Are you sure? (y/n)".red().bold());
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim().eq_ignore_ascii_case("y") {
                config::clear_all_data()?;
                println!(
                    "{} All blockchain and wallet data has been wiped clean.",
                    "[SUCCESS]".green()
                );
            } else {
                println!("Operation cancelled.");
            }
        }
    }

    if state_changed {
        config::save_app_state(&state)?;
    }

    Ok(())
}