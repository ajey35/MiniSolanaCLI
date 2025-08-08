use std::{ path::PathBuf, time::Duration};
use colored::*;
use figlet_rs::FIGfont;
use anyhow::{anyhow, Ok};
use clap::{ Parser, Subcommand};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, signature::{ read_keypair_file, write_keypair_file, Keypair }, signer::Signer, system_instruction, transaction::Transaction
};
#[derive(Parser)]
#[command(name="Mini-Solana-CLI")]
#[command(about="It is Mini SOlana CLI for Blockchain Interations",long_about=None)]
struct Cli{
    #[command(subcommand)]
    command:Commands
}


#[derive(Subcommand)]
enum Commands{

    CreateAccount{
        #[arg(short,long,default_value="my-keypair.json")]
        outfile:PathBuf,
    },
    Address{
        #[arg(short,long,default_value="my-keypair.json")]
        key:String
    },
    Balance{
        #[arg(short,long,default_value="my-keypair.json")]
        key:String,
    },
    Airdrop{
        #[arg(short,long)]
        key:String,
        #[arg(short,long)]
        amount:f64
    },
    Send{
        #[arg(short,long,default_value="my-keypair.json")]
        from:PathBuf,
        #[arg(short,long)]
        to:String,
        #[arg(short,long)]
        amount:f64,
        
    }
}
#[tokio::main]
async fn main()->anyhow::Result<()>{

    let standard_font = FIGfont::standard().unwrap();
    if let Some(figure) = standard_font.convert("MINISOLCLI") {
        println!("{}", figure.to_string().red().bold());
    }
    println!("{}", "Your Mini Solana CLI for blockchain interactions".bright_red());


    let cli = Cli::parse();

    let url = String::from("https://api.devnet.solana.com");
    let rpc = RpcClient::new_with_timeout(url.clone(), Duration::from_secs(60));

    match cli.command {
        Commands::CreateAccount { outfile }=>{
            create_account(outfile)?;
        }
        Commands::Address { key }=>{
            let pubkey = parse_or_pubkey(&key)?;
            println!("{}",pubkey);
        }
        Commands::Balance { key }=>{
            let pubkey = parse_or_pubkey(&key)?;
            let lamps = rpc.get_balance(&pubkey).await?;
            println!("Balance :{} SOL",(lamps as f64/LAMPORTS_PER_SOL as f64));
        }
        Commands::Airdrop { key, amount }=>{
            let pubkey = parse_or_pubkey(&key)?;
            request_airdrop_sol(&pubkey,amount,&rpc).await?;   
        }
        Commands::Send { from, to, amount }=>{
            let to = parse_or_pubkey(&to)?;
            send_sol(&from,&to,amount,&rpc).await?;
        }
    }

    Ok(())

}



fn create_account(outfile:PathBuf)->anyhow::Result<()>{

    let kp = Keypair::new();

    let _ = write_keypair_file(&kp, &outfile);
    println!("Wrote the New Keypair to {}",outfile.display());

    println!("Pubkey :{}",kp.pubkey());

    Ok(())
}

fn parse_or_pubkey(s:&str)->anyhow::Result<Pubkey>{
    if std::path::Path::new(s).exists(){
        let kp = read_keypair_file(s).unwrap();
        Ok(kp.pubkey())
    }
    else {
        let pk =s.parse::<Pubkey>()?;
        Ok(pk)
    }
}


async  fn request_airdrop_sol(pubkey:&Pubkey,amount:f64,rpc:&RpcClient)->anyhow::Result<()>{

    let lamps = (amount * LAMPORTS_PER_SOL as f64) as u64;
    let sig = rpc.request_airdrop(&*pubkey, lamps).await?;

    println!("Airdrop Transaction Signature {:?}",sig);

    match rpc.confirm_transaction(&sig).await? {
        true =>{
            println!("Airdrop Confirmed");
            let balance = rpc.get_balance(&pubkey).await?;
            println!("Now the Balance : {} SOL",balance as f64 / LAMPORTS_PER_SOL as f64);
        }
        false =>return Err(anyhow!("Airdrop is Not Confirmed"))
    }

    Ok(())

}


async fn send_sol(from:&PathBuf,to:&Pubkey,amount:f64,rpc:&RpcClient)->anyhow::Result<()>{

    let from_kp: Keypair = read_keypair_file(from).unwrap();

    let lamps = (amount * LAMPORTS_PER_SOL as f64) as u64;

    let ix = system_instruction::transfer(&from_kp.pubkey(), &to, lamps);


    let recent_hash = rpc.get_latest_blockhash().await?;
    let tx = Transaction::new_signed_with_payer(
        &[ix], 
        Some(&from_kp.pubkey()),
        &[&from_kp],
        recent_hash
    );

    let sig = rpc.send_and_confirm_transaction(&tx).await?;

    println!("Transfer Complete : {:?}",sig);
    Ok(())
}