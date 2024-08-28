import BN from "bn.js";
import * as web3 from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
// Client

import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider, web3, BN } from "@coral-xyz/anchor";
import { LiquidityPoolStaking } from "../target/types/liquidity_pool_staking";
import { TOKEN_PROGRAM_ID, createMint, createAccount, mintTo } from "@solana/spl-token";
import type { LiquidityPoolStaking } from "../target/types/liquidity_pool_staking";

// Configure the client to use the local cluster
anchor.setProvider(anchor.AnchorProvider.env());

const program = anchor.workspace.LiquidityPoolStaking as anchor.Program<LiquidityPoolStaking>;


const { SystemProgram } = web3;

async function main() {
  const provider = AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.LiquidityPoolStaking as Program<LiquidityPoolStaking>;

  // Create a new mint for the LP tokens
  const lpMint = await createMint(
    provider.connection,
    provider.wallet.payer,
    provider.wallet.publicKey,
    null,
    9 // Decimals
  );

  // Create a new mint for the reward tokens (governance tokens)
  const rewardMint = await createMint(
    provider.connection,
    provider.wallet.payer,
    provider.wallet.publicKey,
    null,
    9 // Decimals
  );

  // Create associated token accounts for the user
  const userLpTokenAccount = await createAccount(provider.connection, provider.wallet.payer, lpMint, provider.wallet.publicKey);
  const userRewardTokenAccount = await createAccount(provider.connection, provider.wallet.payer, rewardMint, provider.wallet.publicKey);

  // Mint some LP tokens to the user's account
  await mintTo(provider.connection, provider.wallet.payer, lpMint, userLpTokenAccount, provider.wallet.payer, 1000000000);

  // Initialize the staking pool
  const [poolPDA, poolBump] = await web3.PublicKey.findProgramAddress(
    [Buffer.from("pool")],
    program.programId
  );

  const [lpVaultPDA, lpVaultBump] = await web3.PublicKey.findProgramAddress(
    [Buffer.from("lp_vault")],
    program.programId
  );

  const [rewardsVaultPDA, rewardsVaultBump] = await web3.PublicKey.findProgramAddress(
    [Buffer.from("rewards_vault")],
    program.programId
  );

  await program.rpc.initialize(poolBump, new BN(1), new BN(1), {
    accounts: {
      pool: poolPDA,
      lpVault: lpVaultPDA,
      rewardsVault: rewardsVaultPDA,
      user: provider.wallet.publicKey,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
    },
  });

  console.log("Staking pool initialized.");

  // Stake LP tokens
  await program.rpc.stakeLpTokens(new BN(1000000), new BN(60 * 60 * 24 * 7), { // 1 week lockup
    accounts: {
      user: provider.wallet.publicKey,
      lpStakingAccount: userLpTokenAccount,
      userLpTokenAccount: userLpTokenAccount,
      lpVault: lpVaultPDA,
      pool: poolPDA,
      tokenProgram: TOKEN_PROGRAM_ID,
    },
  });

  console.log("Staked LP tokens.");

  // Unstake LP tokens (after lockup period)
  // Note: Ensure lockup period has ended before calling this in practice
  await program.rpc.unstakeLpTokens({
    accounts: {
      user: provider.wallet.publicKey,
      lpStakingAccount: userLpTokenAccount,
      userLpTokenAccount: userLpTokenAccount,
      lpVault: lpVaultPDA,
      pool: poolPDA,
      tokenProgram: TOKEN_PROGRAM_ID,
    },
  });

  console.log("Unstaked LP tokens.");

  // Claim rewards
  await program.rpc.claimLpRewards({
    accounts: {
      user: provider.wallet.publicKey,
      lpStakingAccount: userLpTokenAccount,
      rewardsVault: rewardsVaultPDA,
      userRewardsTokenAccount: userRewardTokenAccount,
      pool: poolPDA,
      tokenProgram: TOKEN_PROGRAM_ID,
    },
  });

  console.log("Claimed rewards.");
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});

/*
console.log("My address:", program.provider.publicKey.toString());
const balance = await program.provider.connection.getBalance(program.provider.publicKey);
console.log(`My balance: ${balance / web3.LAMPORTS_PER_SOL} SOL`);*'/
