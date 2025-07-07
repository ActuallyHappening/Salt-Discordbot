use crate::prelude::*;
use alloy::primitives::utils::{ParseUnits, Unit};
use alloy::primitives::{Address, U256, address, utils::parse_ether};
use alloy::sol_types::SolCall as _;
use color_eyre::eyre::Context as _;
use or_poisoned::OrPoisoned as _;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
	application::interaction::Interaction,
	http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::{
	chains::{BlockchainListing, SomniaShannon},
	commands::faucet::DiscordInfo,
	common::GlobalStateRef,
	ratelimits,
};

/// Faucet 0.05 PING on Somnia Shannon (an ERC20 token)
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "somnia-shannon-ping")]
pub struct SomniaShannonPing {
	/// Your personal wallet address
	address: String,
}

impl BlockchainListing for SomniaShannonPing {
	fn chain_id(&self) -> u64 {
		self.plain().chain_id()
	}

	fn chain_name(&self) -> &'static str {
		self.plain().chain_name()
	}

	fn native_token_name(&self) -> &'static str {
		self.plain().native_token_name()
	}
}

impl SomniaShannonPing {
	fn plain(&self) -> SomniaShannon {
		SomniaShannon {
			address: self.address.clone(),
		}
	}

	fn amount() -> U256 {
		parse_ether("0.05").unwrap()
	}

	fn erc20_token_name() -> &'static str {
		"PING"
	}

	const SMART_CONTRACT_ADDR: Address = address!("0x33E7fAB0a8a5da1A923180989bD617c9c2D1C493");
}

// Generate the contract bindings for the ERC20 interface.
alloy::sol! {
	// The `rpc` attribute enables contract interaction via the provider.
	#[sol(rpc, abi)]
	contract ERC20 {
		function name() public view returns (string);
		function symbol() public view returns (string);
		function decimals() public view returns (uint8);
		function totalSupply() public view returns (uint256);
		function balanceOf(address account) public view returns (uint256);
		function transfer(address recipient, uint256 amount) public returns (bool);
		function allowance(address owner, address spender) public view returns (uint256);
		function approve(address spender, uint256 amount) public returns (bool);
		function transferFrom(address sender, address recipient, uint256 amount) public returns (bool);

		event Transfer(address indexed from, address indexed to, uint256 value);
		event Approval(address indexed owner, address indexed spender, uint256 value);
	}
}

impl SomniaShannonPing {
	pub async fn handle(
		&self,
		state: GlobalStateRef<'_>,
		interaction: Interaction,
		discord_info: DiscordInfo,
	) -> color_eyre::Result<()> {
		let address = &self.address;
		let chain_id = self.chain_id();
		let chain_name = self.chain_name();
		let token_name = Self::erc20_token_name();
		let amount = Self::amount();
		let amount_eth = ParseUnits::from(amount).format_units(Unit::ETHER);
		let rpc_url = &state.env.somnia_shannon_rpc_endpoint;
		let DiscordInfo {
			discord_id,
			has_expanded_limits,
		} = discord_info;

		let respond = async |msg: &str| {
			state
				.client
				.interaction(interaction.application_id)
				.create_response(
					interaction.id,
					&interaction.token,
					&InteractionResponse {
						kind: InteractionResponseType::ChannelMessageWithSource,
						data: Some(InteractionResponseDataBuilder::new().content(msg).build()),
					},
				)
				.await
				.wrap_err("Couldn't initially respond to a discord interaction")
		};
		let follow_up = async |msg: &str| {
			state
				.client
				.interaction(interaction.application_id)
				.create_followup(&interaction.token)
				.content(msg.as_ref())
				.await
				.wrap_err("Couldn't followup a discord interaction")
		};

		let address = match address.parse() {
			Ok(address) => address,
			Err(err) => {
				respond(&format!(
					"Invalid Etherium wallet address {:?}: {}",
					self.address, err
				))
				.await?;
				return Ok(());
			}
		};

		// check ratelimiting if not expanded limits
		let ratelimit_key = ratelimits::Key {
			address,
			discord_id,
			chain_id,
			chain_name,
		};
		if !has_expanded_limits {
			let ratelimit = state.ratelimits.lock().await?.check(&ratelimit_key);
			if let Err(msg) = ratelimit {
				let msg =
					format!("Couldn't faucet you any tokens because you are ratelimited!\n{msg}");
				respond(&msg).await?;
				return Ok(());
			}
		} else {
			info!(%discord_id, "This person has expanded limits");
		}

		let calldata = ERC20::transferCall {
			amount,
			recipient: address,
		}
		.abi_encode();

		// initial response
		respond(&format!(
			"Starting faucet of {amount_eth}{token_name}, an ERC20 token ({chain_name}), to {address} ..."
		))
		.await?;

		// do transaction
		let (send_logs, mut live_logs) = tokio::sync::mpsc::channel(10);
		let salt_config = salt_sdk::SaltConfig {
			private_key: state.env.private_key.clone(),
			orchestration_network_rpc_node: state.env.sepolia_arbitrum_rpc_endpoint.clone(),
			broadcasting_network_rpc_node: rpc_url.clone(),
			broadcasting_network_id: chain_id,
		};
		let salt = salt_sdk::Salt::new(salt_config)?;
		let transaction = salt.transaction(salt_sdk::TransactionInfo {
			amount: U256::from(0),
			vault_address: state.env.faucet_testnet_salt_account_address,
			recipient_address: SomniaShannonPing::SMART_CONTRACT_ADDR,
			data: calldata,
			logging: salt_sdk::LiveLogging::from_sender(send_logs),
			gas: salt_sdk::GasEstimator::Mul(100.0),
			confirm_publish: true,
		});
		let logging = async move {
			while let Some(log) = live_logs.recv().await {
				info!(%log, "Sending live log");
				follow_up(&log)
					.await
					.wrap_err("Live logging failed to send")?;
			}
			Result::<(), color_eyre::Report>::Ok(())
		};

		let (res, logging_err) = tokio::join!(transaction, logging);

		if let Err(err) = logging_err {
			error!("Failed to send live logs:\n{}", err);
		}

		if let Err(err) = res {
			error!("Failed to do salt transaction:\n{}", err);
			let mut err_string = err.to_string();

			if let salt_sdk::Error::SubprocessExitedBadlyWithOutput(output) = err {
				err_string = output.stderr;
			}

			if err_string.len() > 1900 {
				// only keeps first 1900 bytes, avoiding a panic if using String.split_off
				// https://doc.rust-lang.org/stable/std/string/struct.String.html#method.split_off
				let truncated = err_string
					.into_bytes()
					.into_iter()
					.take(1900)
					.collect::<Vec<u8>>();
				let truncated = String::from_utf8_lossy(&truncated);
				err_string = format!("{truncated}...<truncated>");
			}
			err_string = format!(
				"Error transacting {amount_eth}{token_name} ({chain_name}) to {address}:\n{err_string}"
			);
			follow_up(&err_string)
				.await
				.wrap_err("Couldn't follow up on a failed transaction with an error message")?;
		} else {
			// still registers even if expanded limits
			state
				.ratelimits
				.lock()
				.await?
				.register(&ratelimit_key)
				.await
				.wrap_err("Couldn't register successful bot transaction")?;
			follow_up(&format!(
				"Successful faucet of {amount_eth}{token_name} ({chain_name}) to {address}"
			))
			.await?;
			info!("Finished handling the discord interaction");
		}

		Ok(())
	}
}
