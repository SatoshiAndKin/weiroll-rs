use alloy::primitives::{Address, FixedBytes, address};
use alloy::providers::ProviderBuilder;
use alloy::signers::local::PrivateKeySigner;
use alloy::sol_types::SolEventInterface;
use alloy::{dyn_abi::DynSolType, node_bindings::Anvil};
// use ethers::abi::RawLog;
// use ethers::{abi::ParamType, prelude::*, utils::Anvil};
use weiroll::{
    Planner,
    bindings::{
        erc20::ERC20,
        events::Events,
        testable_vm::TestableVM,
    },
};

const WETH_ADDR: Address = address!("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2");
const VIT_ADDR: Address = address!("0xab5801a7d398351b8be11c439e05c5b3259aec9b");
const PROVIDER_URL: &str = "http://localhost:8545";

#[tokio::main]
pub async fn main() {
    println!("Spawning anvil..");
    let anvil = Anvil::new().fork(PROVIDER_URL).spawn();
    let wallet: PrivateKeySigner = anvil.keys().first().unwrap().clone().into();

    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .connect(&anvil.endpoint())
        .await
        .unwrap();

    println!("Deploying contracts..");
    let events = Events::deploy(&provider).await.unwrap();
    let vm = TestableVM::deploy(&provider).await.unwrap();

    println!("Planner..");
    let mut planner = Planner::default();
    planner
        .call::<Events::logStringCall>(
            *events.address(),
            vec![String::from("Checking balance..").into()],
            DynSolType::Uint(256),
        )
        .unwrap();
    let balance = planner
        .call::<ERC20::balanceOfCall>(WETH_ADDR, vec![VIT_ADDR.into()], DynSolType::Uint(256))
        .unwrap();
    planner
        .call::<Events::logUintCall>(
            *events.address(),
            vec![balance.into()],
            DynSolType::Uint(256),
        )
        .unwrap();
    let (commands, state) = planner.plan().unwrap();
    let commands: Vec<FixedBytes<32>> = commands.into_iter().map(Into::into).collect();

    println!("Executing..");
    let receipt = vm
        .execute(commands, state)
        .send()
        .await
        .unwrap()
        .get_receipt()
        .await
        .unwrap();

    println!("Logs:");
    for log in receipt.logs() {
        let topics: Vec<alloy::sol_types::Word> =
            log.data().topics().iter().copied().map(Into::into).collect();
        let call = Events::EventsEvents::decode_raw_log(&topics, log.data().data.as_ref()).unwrap();
        println!("{:?}", call);
    }
}
