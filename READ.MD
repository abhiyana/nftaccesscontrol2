
### **NFT License Smart Contract Setup Guide**

#### **Prerequisites**

- **Rust**: Ensure Rust is installed. If not, [install it here](https://rustup.rs/).
- **Anchor**: Install the Anchor framework by running:
  ```bash
  cargo install --git https://github.com/coral-xyz/anchor anchor-cli --locked
  ```
- **Solana CLI**: Install the Solana CLI and configure the network (e.g., Devnet) by running:
  ```bash
  sh -c "$(curl -sSfL https://release.solana.com/v1.9.4/install)"
  solana config set --url https://api.devnet.solana.com
  ```

#### **1. Clone the Project**

Clone the repository containing the smart contract code:
```bash
git clone <repository-url>
cd nftaccesscontrol2
```

#### **2. Configure Anchor**

Set up Anchor for deployment by editing `Anchor.toml` if necessary.

#### **3. Compile the Smart Contract**

Compile the contract to check for errors:
```bash
anchor build
```

#### **4. Deploy the Contract**

Deploy the contract on the Solana Devnet:
```bash
anchor deploy
```

After deploying, note the program ID generated.

#### **5. Testing the Contract**

To test the contract, ensure test scripts are in place and run:
```bash
anchor test
```
