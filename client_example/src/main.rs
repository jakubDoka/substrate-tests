use std::time::Instant;
use subxt::{OnlineClient, PolkadotConfig};
use subxt_signer::sr25519::{dev, Keypair};

#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod polkadot {}

async fn get_balance(
	api: &OnlineClient<PolkadotConfig>,
	kp: &Keypair,
) -> Result<u128, Box<dyn std::error::Error>> {
	let account = kp.public_key().into();
	let storage_query = polkadot::storage().system().account(&account);

	Ok(api.storage().at_latest().await?.fetch(&storage_query).await?.unwrap().data.free)
}

async fn transfer(
	api: &OnlineClient<PolkadotConfig>,
	from_kp: &Keypair,
	dest_kp: &Keypair,
	amount: u128,
	nonce_plus: u64,
) -> Result<(), Box<dyn std::error::Error>> {
	let dest = dest_kp.public_key().into();
	let balance_transfer_tx = polkadot::tx().balances().transfer_allow_death(dest, amount);

	println!("\n ---------------------- initiating transaction...");

	let start = Instant::now();
	let nonce = api.tx().account_nonce(&from_kp.public_key().to_account_id()).await?;
	println!("     --- The nonce is {nonce}");
	let	tx_in_block = api.tx().create_signed_with_nonce(&balance_transfer_tx, from_kp, nonce + nonce_plus, Default::default())?.submit_and_watch().await?
//	let tx_in_block = api
//		.tx()
//		.sign_and_submit_then_watch_default(&balance_transfer_tx, from_kp)
//		.await?
		.wait_for_in_block()
		.await?;
	println!("-------------------- tx submitted {:?}",  start.elapsed());

	// Find a Transfer event and print it.
	let events = tx_in_block.fetch_events().await?;

	let event = events.find_first::<polkadot::system::events::ExtrinsicSuccess>()?;
	if let Some(event) = event {
		println!("extrinsic success: {:?}", event);
	}

	let event = events.find_first::<polkadot::system::events::ExtrinsicFailed>()?;
	if let Some(event) = event {
		println!("extrinsic failed: {:?}", event);
	}

	println!("-------------------- events processed {:?}",  start.elapsed());


	Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let api = OnlineClient::<PolkadotConfig>::from_url("ws://localhost:9944").await?;
	let alice = dev::alice();
	let bob = dev::bob();
	// let charlie = dev::charlie();

	println!("Alice has free balance: {}", get_balance(&api, &alice).await.unwrap());
	println!("Bob has free balance: {}", get_balance(&api, &bob).await.unwrap());
	// println!("Charlie has free balance: {}", get_balance(&api, &charlie).await.unwrap());

	println!(
		"\nTransfering Alice to Bob: \n{:?}",
		transfer(&api, &alice, &bob, 200_000_000_000, 0).await
	);
	println!(
		"\nTransfering Alice to Bob: \n{:?}",
		transfer(&api, &alice, &bob, 200_000_000_000, 1).await
	);
	//	println!(
	//		"\nTransfering Bob to Charlie: \n{:?}",
	//		transfer(&api, &bob, &charlie, 10_000_000_000).await
	//	);

	println!("\nAlice has free balance: {}", get_balance(&api, &alice).await.unwrap());
	println!("Bob has free balance: {}", get_balance(&api, &bob).await.unwrap());
	// println!("Charlie has free balance: {}", get_balance(&api, &charlie).await.unwrap());

	Ok(())
}
