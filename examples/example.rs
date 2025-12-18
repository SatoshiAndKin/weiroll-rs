use alloy::node_bindings::Anvil;
use alloy::primitives::{Address, address};
use alloy::providers::ProviderBuilder;
use alloy::signers::local::PrivateKeySigner;
use alloy::sol_types::SolEventInterface;
// use ethers::abi::RawLog;
// use ethers::{abi::ParamType, prelude::*, utils::Anvil};
use weiroll::{
    Planner,
    bindings::{erc20::ERC20, events::Events, testable_vm::TestableVM},
};

const WETH_ADDR: Address = address!("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2");
const AAVE_ADDR: Address = address!("0x4d5F47FA6A74757f35C14fD3a6Ef8E3C9BC514E8");
const DEFAULT_PROVIDER_URL: &str = "http://localhost:8545";

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider_url =
        std::env::var("PROVIDER_URL").unwrap_or_else(|_| DEFAULT_PROVIDER_URL.to_string());

    println!("Spawning anvil..");
    let anvil = Anvil::new().fork(provider_url).spawn();
    let wallet = anvil
        .keys()
        .first()
        .cloned()
        .ok_or_else(|| std::io::Error::other("anvil returned no keys"))?;
    let wallet: PrivateKeySigner = wallet.into();

    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .connect(&anvil.endpoint())
        .await?;

    println!("Deploying contracts..");
    let events = Events::deploy(&provider).await?;
    let vm = TestableVM::deploy(&provider).await?;
    let weth = ERC20::new(WETH_ADDR, &provider);

    let direct_balance = weth.balanceOf(AAVE_ADDR).call().await?;
    println!("Direct WETH balanceOf(AAVE_ADDR): {direct_balance}");

    println!("Planner..");
    let mut planner = Planner::default();
    weiroll::call_contract!(
        &mut planner,
        &events,
        (Events::logStringCall {
            message: String::from("Checking balance.."),
        })
    )?;
    let balance = weiroll::call_contract!(&mut planner, &weth, ERC20::balanceOfCall[AAVE_ADDR])?;
    weiroll::call_contract!(&mut planner, &events, Events::logUintCall[balance])?;
    let (commands, state) = planner.plan()?;

    println!("Executing..");
    let receipt = vm
        .execute(commands, state)
        .send()
        .await?
        .get_receipt()
        .await?;

    // dbg!(&receipt);

    println!("Logs:");
    for log in receipt.logs() {
        let topics: Vec<alloy::sol_types::Word> = log.data().topics().to_vec();
        let call = Events::EventsEvents::decode_raw_log(&topics, log.data().data.as_ref())?;
        println!("{:?}", call);
    }

    Ok(())
}
