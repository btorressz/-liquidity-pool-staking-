# liquidity-pool-staking

Prototype in Progress

This project is currently a prototype and is under active development. It is not yet fully functional, and there are known issues and incomplete features. Contributions and feedback are welcome!

## Overview

This project implements a Liquidity Pool Staking program on the Solana blockchain using the Anchor framework. Users can stake their LP tokens, earn rewards, and claim governance tokens based on their staked amounts and staking durations.

Note: This is a prototype project and is not intended for production use yet. The code and contracts are subject to change and should not be used on mainnet or for any real transactions.



## Features

- **Staking Functionality**: Users can stake LP tokens with a specified lock-up period.
- **Lock-Up Period**: Tokens cannot be unstaked until the lock-up period has expired.
- **Reward Mechanism**: Rewards are calculated based on the amount of staked tokens and a reward multiplier.
- **Governance Token Rewards**: Distributes governance tokens as rewards for staking.
- **Event Logging**: Logs various staking-related events for tracking and debugging.
- **Security Checks**: Ensures only authorized accounts can change reward rates and multipliers.

## Project Structure

- `programs/`: Contains the Solana program written in Rust using the Anchor framework.
  - `src/lib.rs`: Main program logic and account structures.
- `tests/`: Contains the integration tests for the program.
  - `anchor.test.ts`: Integration tests using Anchor's testing framework.
- `client/`: Contains the client script for interacting with the deployed program.
  - `client.ts`: Example script to interact with the program.
 
## Tech Stack

Rust

TypeScript

Anchor

Solana

## Tools
Solana Playground IDE: Used for initial development and testing of Solana programs. It provides an easy-to-use interface to write, deploy, and test Solana smart contracts without needing extensive local setup.(This project was 100% developed in solana playground IDE and coverted and exported for VSCode)

VSCode: Used for advanced development and editing. VSCode provides powerful extensions for Rust, TypeScript, and Solana development, allowing for a more integrated and efficient coding environment.
