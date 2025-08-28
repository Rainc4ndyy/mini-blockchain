use crate::{blockchain::Blockchain, wallet::Wallet};
use anyhow::{Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};

const APP_DIR: &str = "mini-blockchain";
const CONFIG_FILE: &str = "config.json";
const CHAIN_FILE: &str = "chain.json";
const WALLETS_DIR: &str = "wallets";
const CONTACTS_FILE: &str = "contacts.json";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub active_wallet: Option<String>,
}

pub struct AppState {
    pub config: Config,
    pub blockchain: Blockchain,
    pub contacts: HashMap<String, String>,
}

pub fn get_app_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().context("Could not find the system's config directory.")?;
    let app_dir = config_dir.join(APP_DIR);
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)?;
    }
    Ok(app_dir)
}

pub fn load_app_state() -> Result<AppState> {
    let app_dir = get_app_dir()?;

    let config_path = app_dir.join(CONFIG_FILE);
    let config = match fs::read_to_string(config_path) {
        Ok(data) => serde_json::from_str(&data)?,
        Err(_) => Config::default(),
    };

    let chain_path = app_dir.join(CHAIN_FILE);
    let blockchain = match fs::read_to_string(chain_path) {
        Ok(data) => {
            println!("{}", "[INFO] Found saved blockchain data. Loading it now.".cyan());
            serde_json::from_str(&data)?
        }
        Err(_) => {
            println!("{}", "[INFO] No saved blockchain found. Creating a fresh one!".yellow());
            Blockchain::new()?
        }
    };

    let contacts_path = app_dir.join(CONTACTS_FILE);
    let contacts = match fs::read_to_string(contacts_path) {
        Ok(data) => serde_json::from_str(&data)?,
        Err(_) => HashMap::new(),
    };

    Ok(AppState {
        config,
        blockchain,
        contacts,
    })
}

pub fn save_app_state(state: &AppState) -> Result<()> {
    let app_dir = get_app_dir()?;

    let config_path = app_dir.join(CONFIG_FILE);
    let config_data = serde_json::to_string_pretty(&state.config)?;
    fs::write(config_path, config_data)?;

    let chain_path = app_dir.join(CHAIN_FILE);
    let chain_data = serde_json::to_string_pretty(&state.blockchain)?;
    fs::write(chain_path, chain_data)?;

    let contacts_path = app_dir.join(CONTACTS_FILE);
    let contacts_data = serde_json::to_string_pretty(&state.contacts)?;
    fs::write(contacts_path, contacts_data)?;

    Ok(())
}

pub fn get_wallets_dir() -> Result<PathBuf> {
    let app_dir = get_app_dir()?;
    let wallets_dir = app_dir.join(WALLETS_DIR);
    if !wallets_dir.exists() {
        fs::create_dir_all(&wallets_dir)?;
    }
    Ok(wallets_dir)
}

pub fn save_wallet(name: &str, wallet: &Wallet) -> Result<()> {
    let wallets_dir = get_wallets_dir()?;
    let wallet_path = wallets_dir.join(format!("{}.json", name));
    let json = serde_json::to_string_pretty(wallet)?;
    fs::write(wallet_path, json)?;
    Ok(())
}

pub fn load_wallet(name: &str) -> Result<Wallet> {
    let wallets_dir = get_wallets_dir()?;
    let wallet_path = wallets_dir.join(format!("{}.json", name));
    let json_data = fs::read_to_string(&wallet_path).context(format!(
        "Couldn't find wallet '{}'. Check the name or create a new one with `wallet new`.",
        name
    ))?;
    let wallet = serde_json::from_str(&json_data)?;
    Ok(wallet)
}

pub fn get_all_wallets() -> Result<Vec<(String, String)>> {
    let wallets_dir = get_wallets_dir()?;
    let mut wallets = Vec::new();
    for entry in fs::read_dir(wallets_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |e| e == "json") {
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                let wallet = load_wallet(name)?;
                let address = hex::encode(wallet.public_key.to_encoded_point(true));
                wallets.push((name.to_string(), address));
            }
        }
    }
    Ok(wallets)
}

pub fn clear_all_data() -> Result<()> {
    let app_dir = get_app_dir()?;
    if app_dir.exists() {
        fs::remove_dir_all(app_dir).context("Whoops, failed to delete the app data directory.")?;
    }
    Ok(())
}