import BN from "bn.js";
import * as web3 from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider, BN } from "@coral-xyz/anchor";
import { LiquidityPoolStaking } from "../target/types/liquidity_pool_staking";
import { TOKEN_PROGRAM_ID, createMint, createAccount, mintTo, getAccount } from "@solana/spl-token";
import { expect } from "chai";
import type { LiquidityPoolStaking } from "../target/types/liquidity_pool_staking";

describe("liquidity_pool_staking", () => {
  // Configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.LiquidityPoolStaking as anchor.Program<LiquidityPoolStaking>;
  
  const provider = AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.LiquidityPoolStaking as Program<LiquidityPoolStaking>;

  let lpMint = null as any;
  let rewardMint = null as any;
  let userLpTokenAccount = null as any;
  let userRewardTokenAccount = null as any;
  let poolPDA = null as any;
  let lpVaultPDA = null as any;
  let rewardsVaultPDA = null as any;

  it("Initializes the staking pool", async () => {
    lpMint = await createMint(
      provider.connection,
      provider.wallet.payer,
      provider.wallet.publicKey,
      null,
      9 // Decimals
    );

    rewardMint = await createMint(
      provider.connection,
      provider.wallet.payer,
      provider.wallet.publicKey,
      null,
      9 // Decimals
    );

    userLpTokenAccount = await createAccount(provider.connection, provider.wallet.payer, lpMint, provider.wallet.publicKey);
    userRewardTokenAccount = await createAccount(provider.connection, provider.wallet.payer, rewardMint, provider.wallet.publicKey);

    await mintTo(provider.connection, provider.wallet.payer, lpMint, userLpTokenAccount, provider.wallet.payer, 1000000000);

    [poolPDA] = await web3.PublicKey.findProgramAddress([Buffer.from("pool")], program.programId);
    [lpVaultPDA] = await web3.PublicKey.findProgramAddress([Buffer.from("lp_vault")], program.programId);
    [rewardsVaultPDA] = await web3.PublicKey.findProgramAddress([Buffer.from("rewards_vault")], program.programId);

    await program.rpc.initialize(new BN(1), new BN(1), new BN(1), {
      accounts: {
        pool: poolPDA,
        lpVault: lpVaultPDA,
        rewardsVault: rewardsVaultPDA,
        user: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    });

    const pool = await program.account.pool.fetch(poolPDA);
    expect(pool.rewardRate.toNumber()).to.equal(1);
    expect(pool.rewardMultiplier.toNumber()).to.equal(1);
    console.log("Staking pool initialized.");
  });

  it("Stakes LP tokens", async () => {
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

    const lpStakingAccount = await getAccount(provider.connection, userLpTokenAccount);
    expect(lpStakingAccount.amount.toNumber()).to.equal(0);

    const lpVaultAccount = await getAccount(provider.connection, lpVaultPDA);
    expect(lpVaultAccount.amount.toNumber()).to.equal(1000000);

    console.log("Staked LP tokens.");
  });

  it("Unstakes LP tokens", async () => {
    // Move time forward to simulate the end of the lockup period
    await provider.connection.sendTransaction(
      new anchor.web3.Transaction().add(anchor.web3.SystemProgram.advanceClockBy(60 * 60 * 24 * 7 + 1)), // Advance time by one week and one second
      []
    );

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

    const lpStakingAccount = await getAccount(provider.connection, userLpTokenAccount);
    expect(lpStakingAccount.amount.toNumber()).to.equal(1000000);

    const lpVaultAccount = await getAccount(provider.connection, lpVaultPDA);
    expect(lpVaultAccount.amount.toNumber()).to.equal(0);

    console.log("Unstaked LP tokens.");
  });

  it("Claims rewards", async () => {
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

    const userRewardAccount = await getAccount(provider.connection, userRewardTokenAccount);
    expect(userRewardAccount.amount.toNumber()).to.be.greaterThan(0);

    console.log("Claimed rewards.");
  });
});
