use std::{path::PathBuf, time::Duration};

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use figlet_rs::FIGfont;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{read_keypair_file, write_keypair_file, Keypair},
    signer::Signer,
    system_instruction,
    transaction::Transaction,
};

const SEPARATOR: &str = "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━";

#[derive(ValueEnum, Clone, Debug)]
enum Cluster {
    Devnet,
    Localnet,
    Mainnet,
}

impl Cluster {
    fn url(&self) -> String {
        match self {
            Cluster::Devnet => "https://api.devnet.solana.com",
            Cluster::Localnet => "http://localhost:8899",
            Cluster::Mainnet => "https://api.mainnet-beta.solana.com",
        }
        .to_string()
    }
}

#[derive(Parser)]
#[command(name = "Mini-Solana-CLI")]
#[command(about = "🦋 Mini Solana CLI for quick blockchain interactions", long_about = None)]
struct Cli {
    #[arg(short, long, value_enum, default_value_t = Cluster::Devnet)]
    cluster: Cluster,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new keypair file
    Create {
        #[arg(short, long, default_value = "my-keypair.json")]
        outfile: PathBuf,
    },
    /// Show the public address from keypair or given pubkey
    Address {
        #[arg(short, long, default_value = "my-keypair.json")]
        key: String,
    },
    /// Check account balance
    Balance {
        #[arg(short, long, default_value = "my-keypair.json")]
        key: String,
    },
    Airdrop {
        #[arg(short, long)]
        key: String,
        #[arg(short, long)]
        amount: f64,
    },
    /// Send SOL to another account
    Send {
        #[arg(short, long, default_value = "my-keypair.json")]
        from: PathBuf,
        #[arg(short, long)]
        to: String,
        #[arg(short, long)]
        amount: f64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    print_banner_with_cluster(&cli.cluster);

    let rpc_url = cli.cluster.url();
    let rpc = RpcClient::new_with_timeout(rpc_url.clone(), Duration::from_secs(60));

    match cli.command {
        Commands::Create { outfile } => create_account(outfile)?,
        Commands::Address { key } => {
            let pubkey = parse_or_pubkey(&key)?;
            print_success(&format!("Public Key: {}", pubkey));
        }
        Commands::Balance { key } => {
            let pubkey = parse_or_pubkey(&key)?;
            let lamps = rpc.get_balance(&pubkey).await?;
            print_success(&format!("Balance: {:.4} SOL", lamps as f64 / LAMPORTS_PER_SOL as f64));
        }
        Commands::Airdrop { key, amount } => {
            let pubkey = parse_or_pubkey(&key)?;
            request_airdrop_sol(&pubkey, amount, &rpc).await?;
        }
        Commands::Send { from, to, amount } => {
            let to = parse_or_pubkey(&to)?;
            send_sol(&from, &to, amount, &rpc).await?;
        }
    }

    Ok(())
}

fn print_banner_with_cluster(cluster: &Cluster) {
    let font = FIGfont::standard().unwrap();
    if let Some(fig) = font.convert("MINISOLCLI") {
        println!("\n{}", fig.to_string().bright_blue().bold());
    }
    println!("{}", SEPARATOR.dimmed());

    print_success(&format!(
        "Cluster set to {:?} ({})",
        cluster,
        cluster.url()
    ));

    println!("{}", SEPARATOR.dimmed());
}

fn print_success(msg: &str) {
    println!("{} {}", "✔ Success:".bright_green().bold(), msg.bright_white());
}

//
// ==== CORE FUNCTIONS ====
//
fn create_account(outfile: PathBuf) -> Result<()> {
    let kp = Keypair::new();
    write_keypair_file(&kp, &outfile)
        .map_err(|e| anyhow!("Failed to save keypair: {}", e))?;
    print_success(&format!("New keypair saved to: {}", outfile.display()));
    print_success(&format!("Public Key: {}", kp.pubkey()));
    println!("\n");
    Ok(())
}

fn parse_or_pubkey(s: &str) -> Result<Pubkey> {
    if std::path::Path::new(s).exists() {
        let kp = read_keypair_file(s)
            .map_err(|e| anyhow!("Failed to read keypair: {}", e))?;
        Ok(kp.pubkey())
    } else {
        Ok(s.parse::<Pubkey>().map_err(|e| anyhow!("Invalid public key: {}", e))?)
    }
}

async fn request_airdrop_sol(pubkey: &Pubkey, amount: f64, rpc: &RpcClient) -> Result<()> {
    let lamps = (amount * LAMPORTS_PER_SOL as f64) as u64;
    print_success(&format!("Requesting airdrop of {:.2} SOL", amount));

    let sig = rpc.request_airdrop(pubkey, lamps).await?;
    print_success(&format!("Transaction Signature: {:?}", sig));

    if rpc.confirm_transaction(&sig).await? {
        print_success(&format!("Airdrop confirmed to: {}", pubkey));
        let balance = rpc.get_balance(pubkey).await?;
        print_success(&format!("Balance: {:.4} SOL", balance as f64 / LAMPORTS_PER_SOL as f64));
        println!("\n");
    } else {
        return Err(anyhow!("Airdrop not confirmed"));
    }
    Ok(())
}

async fn send_sol(from: &PathBuf, to: &Pubkey, amount: f64, rpc: &RpcClient) -> Result<()> {
    let from_kp = read_keypair_file(from)
        .map_err(|e| anyhow!("Failed to read sender keypair: {}", e))?;
    let lamps = (amount * LAMPORTS_PER_SOL as f64) as u64;
    let ix = system_instruction::transfer(&from_kp.pubkey(), to, lamps);
    let recent_hash = rpc.get_latest_blockhash().await?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&from_kp.pubkey()),
        &[&from_kp],
        recent_hash,
    );

    let sig = rpc.send_and_confirm_transaction(&tx).await?;
    print_success(&format!("Transfer complete: Sent {:.2} SOL to {}", amount, to));
    print_success(&format!("Transaction Signature: {:?}", sig));
    println!("\n");
    Ok(())
}
