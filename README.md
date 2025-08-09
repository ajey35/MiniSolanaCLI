# 🦋 Mini Solana CLI
A simple, lightweight command-line tool for quick interactions with the Solana blockchain.  

![Preview](https://github.com/user-attachments/assets/0ba37770-759f-41ab-8fe4-0d68609529ce)  

###  Features  
- **Create a new Solana keypair** in seconds  
- **View your public address** from a keypair or given pubkey  
- **Check account balance** instantly  
- **Airdrop SOL** for development and testing  
- **Send SOL** to another account  
- **Multiple cluster support** (devnet, localnet, mainnet)  

---

## Installation  

### Install directly from source
```bash
# Clone the repository
git clone https://github.com/your-username/minsol.git
cd minsol

# Build the project
cargo build --release

# Run the CLI
./target/release/minsol --help
```

### Install from Cargo Crates
```bash
#  published to crates.io
cargo install minsol
```

---

## 📖 Usage  

```bash
minsol [OPTIONS] <COMMAND>
```

### **Commands**
| Command   | Description |
|-----------|-------------|
| `create`  | Create a new keypair file |
| `address` | Show the public address from keypair or given pubkey |
| `balance` | Check account balance |
| `airdrop` | Request an airdrop (devnet/localnet only) |
| `send`    | Send SOL to another account |
| `help`    | Show help message for commands |

---

### **Options**
| Option   | Description |
|----------|-------------|
| `-c, --cluster <CLUSTER>` | Cluster to use (**default:** `devnet`) <br> Possible values: `devnet`, `localnet`, `mainnet` |
| `-h, --help` | Show help message |

---

##  Examples  

```bash
# Create a new keypair
minsol create --outfile my-keypair.json

# Get public address from keypair
minsol address --keypair my-keypair.json

# Check balance
minsol balance --keypair my-keypair.json

# Airdrop 2 SOL (devnet)
minsol airdrop --keypair my-keypair.json --amount 2

# Send 0.5 SOL to another address
minsol send --keypair my-keypair.json --to <RECIPIENT_ADDRESS> --amount 0.5
```

---

##  Cluster Support  
- **Devnet** → For development & testing  
- **Localnet** → For local blockchain testing  
- **Mainnet** → For real transactions  

---

## 📄 License  
This project is licensed under the **MIT License** – feel free to use and modify.


