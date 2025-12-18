use alloy::node_bindings::Anvil;
use alloy::primitives::{Address, FixedBytes, address};
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
const VIT_ADDR: Address = address!("0xab5801a7d398351b8be11c439e05c5b3259aec9b");
const PROVIDER_URL: &str = "http://ski-nuc-3a:8545";

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Spawning anvil..");
    let anvil = Anvil::new().fork(PROVIDER_URL).spawn();
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

    println!("Planner..");
    let mut planner = Planner::default();
    planner.call_contract::<Events::logStringCall, _>(
        &events,
        (String::from("Checking balance.."),),
    )?;
    let balance = planner.call_contract::<ERC20::balanceOfCall, _>(&weth, (VIT_ADDR,))?;
    planner.call_contract::<Events::logUintCall, _>(&events, (balance,))?;
    let (commands, state) = planner.plan()?;
    let commands: Vec<FixedBytes<32>> = commands.into_iter().map(Into::into).collect();

    println!("Executing..");
    let receipt = vm
        .execute(commands, state)
        .send()
        .await?
        .get_receipt()
        .await?;

    println!("Logs:");
    for log in receipt.logs() {
        let topics: Vec<alloy::sol_types::Word> = log.data().topics().to_vec();
        let call = Events::EventsEvents::decode_raw_log(&topics, log.data().data.as_ref())?;
        println!("{:?}", call);
    }

    Ok(())
}
