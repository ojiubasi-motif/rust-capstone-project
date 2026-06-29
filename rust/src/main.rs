#![allow(unused)]
use bitcoin::hex::DisplayHex;
use bitcoincore_rpc::bitcoin::Amount;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use serde::Deserialize;
use serde_json::json;
use std::fs::File;
use std::io::Write;

// Node access params
const RPC_URL: &str = "http://127.0.0.1:18443"; // Default regtest RPC port
const RPC_USER: &str = "alice";
const RPC_PASS: &str = "password";

// You can use calls not provided in RPC lib API using the generic `call` function.
// An example of using the `send` RPC call, which doesn't have exposed API.
// You can also use serde_json `Deserialize` derivation to capture the returned json result.
fn send(rpc: &Client, addr: &str) -> bitcoincore_rpc::Result<String> {
    let args = [
        json!([{addr : 100 }]), // recipient address
        json!(null),            // conf target
        json!(null),            // estimate mode
        json!(null),            // fee rate in sats/vb
        json!(null),            // Empty option object
    ];

    #[derive(Deserialize)]
    struct SendResult {
        complete: bool,
        txid: String,
    }
    let send_result = rpc.call::<SendResult>("send", &args)?;
    assert!(send_result.complete);
    Ok(send_result.txid)
}

fn main() -> bitcoincore_rpc::Result<()> {
    // Connect to Bitcoin Core RPC
    let rpc = Client::new(
        RPC_URL,
        Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
    )?;

    // Get blockchain info
    let blockchain_info = rpc.get_blockchain_info()?;
    println!("Blockchain Info: {:?}", blockchain_info);

    // Create/Load the wallets, named 'Miner' and 'Trader'. Have logic to optionally create/load them if they do not exist or not loaded already.
     let base_auth = Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned());
    //  ==>closure used to create/load wallets
     let get_or_create_wallet = |wallet_name: &str| -> bitcoincore_rpc::Result<Client> {
        let loaded_wallets = rpc.list_wallets()?;
        if !loaded_wallets.contains(&wallet_name.to_string()) {
            let wallet_dir = rpc.list_wallet_dir()?;
            if wallet_dir.contains(&wallet_name.to_string()) {
                rpc.load_wallet(wallet_name)?;
                println!("Loaded wallet: {}", wallet_name);
            } else {
                rpc.create_wallet(wallet_name, None, None, None, None)?;
                println!("Created wallet: {}", wallet_name);
            }
        } else {
            println!("Wallet {} is already loaded", wallet_name);
        }
        // Return a wallet-specific client pointing to the RPC wallet endpoint
        Client::new(
            format!("{}/wallet/{}", RPC_URL, wallet_name),
            base_auth.clone(),
        )
    };
    // ===> call the closure to create/load wallets
    let miner_rpc = get_or_create_wallet("Miner")?;
    let trader_rpc = get_or_create_wallet("Trader")?;

    // Generate spendable balances in the Miner wallet. How many blocks needs to be mined?
     let miner_address = miner_rpc.get_new_address(Some("Mining Reward"), None)?.assume_checked();
    //==> In Bitcoin (and especially regtest), coinbase block rewards are subject to a 100-block 
    // maturity rule. They cannot be spent until at least 100 blocks have been mined on top of them.
    // Therefore, mining 101 blocks is required to mature the very first block reward,
    // which then registers as a positive wallet balance of 50 BTC.
    let blocks_to_mine = 101;
    miner_rpc.generate_to_address(blocks_to_mine, &miner_address)?;
    // Print the balance of the Miner wallet
    let miner_balance = miner_rpc.get_balance(None, None)?;
    println!("Miner Wallet Balance: {}", miner_balance);

    
    // Load Trader wallet and generate a new address
    // ==>Create a receiving address labeled "Received" from the Trader wallet
    let trader_address = trader_rpc.get_new_address(Some("Received"), None)?.assume_checked();
    println!("Trader Address: {}", trader_address);

    // Send 20 BTC from Miner to Trader
    let amount_to_send = Amount::from_btc(20.0).expect("Invalid amount");
    let txid = miner_rpc.send_to_address(
        &trader_address,
        amount_to_send,
        None,
        None,
        None,
        None,
        None,
        None,
    )?;
    println!("Sent 20 BTC. Transaction ID (txid): {}", txid);
    // Check transaction in mempool

    // Mine 1 block to confirm the transaction

    // Extract all required transaction details

    // Write the data to ../out.txt in the specified format given in readme.md

    Ok(())
}
