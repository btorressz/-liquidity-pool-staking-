use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Token};


//Program ID
declare_id!("DjY4jUtRVQnw57KfRhwbSH1FCgrN8iu5ruhDGePfif64");


#[program]
pub mod liquidity_pool_staking {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, _bump: u8, reward_rate: u64, reward_multiplier: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.reward_rate = reward_rate;
        pool.reward_multiplier = reward_multiplier;
        pool.last_update_time = Clock::get()?.unix_timestamp;
        pool.total_staked = 0;
        emit!(InitializeEvent {
            pool: ctx.accounts.pool.key(),
            reward_rate,
            reward_multiplier,
        });
        Ok(())
    }

    pub fn stake_lp_tokens(ctx: Context<StakeLpTokens>, amount: u64, lockup_period: i64) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let lp_staking_account = &mut ctx.accounts.lp_staking_account;
        let pool = &mut ctx.accounts.pool;

        update_pool(pool)?;

        lp_staking_account.user = ctx.accounts.user.key();
        lp_staking_account.lp_amount += amount;
        lp_staking_account.last_stake_time = now;
        lp_staking_account.lockup_end_time = now + lockup_period;
        lp_staking_account.reward_debt = calculate_rewards(lp_staking_account.lp_amount, now - lp_staking_account.last_stake_time, pool.reward_multiplier);

        pool.total_staked += amount;

        // Transfer LP tokens to staking account
        let cpi_accounts = token::Transfer {
            from: ctx.accounts.user_lp_token_account.to_account_info(),
            to: ctx.accounts.lp_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        emit!(StakeEvent {
            user: ctx.accounts.user.key(),
            amount,
            lockup_period,
        });

        Ok(())
    }

    pub fn unstake_lp_tokens(ctx: Context<UnstakeLpTokens>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let lp_staking_account = &mut ctx.accounts.lp_staking_account;
        let pool = &mut ctx.accounts.pool;

        require!(now >= lp_staking_account.lockup_end_time, CustomError::LockupPeriodNotEnded);

        update_pool(pool)?;

        let amount = lp_staking_account.lp_amount;
        lp_staking_account.lp_amount = 0;

        pool.total_staked -= amount;

        // Transfer LP tokens back to user
        let cpi_accounts = token::Transfer {
            from: ctx.accounts.lp_vault.to_account_info(),
            to: ctx.accounts.user_lp_token_account.to_account_info(),
            authority: ctx.accounts.lp_vault.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx.with_signer(&[&[b"lp_vault", &[ctx.bumps["lp_vault"]]]]), amount)?;

        emit!(UnstakeEvent {
            user: ctx.accounts.user.key(),
            amount,
        });

        Ok(())
    }

    pub fn claim_lp_rewards(ctx: Context<ClaimLpRewards>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let lp_staking_account = &mut ctx.accounts.lp_staking_account;
        let pool = &mut ctx.accounts.pool;

        update_pool(pool)?;

        let rewards = calculate_rewards(lp_staking_account.lp_amount, now - lp_staking_account.last_stake_time, pool.reward_multiplier);
        lp_staking_account.reward_debt += rewards;
        lp_staking_account.last_stake_time = now;

        // Transfer governance tokens as rewards to user
        let cpi_accounts = token::Transfer {
            from: ctx.accounts.rewards_vault.to_account_info(),
            to: ctx.accounts.user_rewards_token_account.to_account_info(),
            authority: ctx.accounts.rewards_vault.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx.with_signer(&[&[b"rewards_vault", &[ctx.bumps["rewards_vault"]]]]), rewards)?;

        emit!(ClaimRewardsEvent {
            user: ctx.accounts.user.key(),
            rewards,
        });

        Ok(())
    }

    pub fn update_pool(ctx: Context<UpdatePool>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let now = Clock::get()?.unix_timestamp;

        if pool.total_staked > 0 {
            let elapsed = now - pool.last_update_time;
            pool.accumulated_reward_per_share += pool.reward_rate * elapsed as u64 / pool.total_staked;
        }

        pool.last_update_time = now;
        emit!(UpdatePoolEvent {
            pool: ctx.accounts.pool.key(),
            accumulated_reward_per_share: pool.accumulated_reward_per_share,
        });
        Ok(())
    }

    pub fn set_reward_rate(ctx: Context<SetRewardRate>, new_rate: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.reward_rate = new_rate;
        emit!(SetRewardRateEvent {
            pool: ctx.accounts.pool.key(),
            new_rate,
        });
        Ok(())
    }

    pub fn set_reward_multiplier(ctx: Context<SetRewardMultiplier>, new_multiplier: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.reward_multiplier = new_multiplier;
        emit!(SetRewardMultiplierEvent {
            pool: ctx.accounts.pool.key(),
            new_multiplier,
        });
        Ok(())
    }

    fn calculate_rewards(lp_amount: u64, staking_duration: i64, reward_multiplier: u64) -> u64 {
        // Implement reward calculation logic
        lp_amount * staking_duration as u64 * reward_multiplier / 1000  // Example reward calculation
    }

    fn update_pool(pool: &mut Account<Pool>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;

        if pool.total_staked > 0 {
            let elapsed = now - pool.last_update_time;
            pool.accumulated_reward_per_share += pool.reward_rate * elapsed as u64 / pool.total_staked;
        }

        pool.last_update_time = now;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + Pool::MAX_SIZE, seeds = [b"pool"], bump)]
    pub pool: Account<'info, Pool>,
    #[account(init, payer = user, space = 8 + TokenAccount::LEN, seeds = [b"lp_vault"], bump)]
    pub lp_vault: Account<'info, TokenAccount>,
    #[account(init, payer = user, space = 8 + TokenAccount::LEN, seeds = [b"rewards_vault"], bump)]
    pub rewards_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct StakeLpTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub lp_staking_account: Account<'info, LpStakingAccount>,
    #[account(mut)]
    pub user_lp_token_account: Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"lp_vault"], bump)]
    pub lp_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UnstakeLpTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub lp_staking_account: Account<'info, LpStakingAccount>,
    #[account(mut)]
    pub user_lp_token_account: Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"lp_vault"], bump)]
    pub lp_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ClaimLpRewards<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub lp_staking_account: Account<'info, LpStakingAccount>,
    #[account(mut, seeds = [b"rewards_vault"], bump)]
    pub rewards_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_rewards_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UpdatePool<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
}

#[derive(Accounts)]
pub struct SetRewardRate<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetRewardMultiplier<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    pub authority: Signer<'info>,
}

#[account]
pub struct Pool {
    pub reward_rate: u64,
    pub reward_multiplier: u64,
    pub accumulated_reward_per_share: u64,
    pub last_update_time: i64,
    pub total_staked: u64,
}

impl Pool {
    pub const MAX_SIZE: usize = 8 * 5;  // u64 and i64 fields
}

#[account]
pub struct LpStakingAccount {
    pub user: Pubkey,
    pub lp_amount: u64,
    pub reward_debt: u64,
    pub last_stake_time: i64,
    pub lockup_end_time: i64,
}

impl LpStakingAccount {
    pub const MAX_SIZE: usize = 32 + 8 * 4;  // Pubkey and u64/i64 fields
}

#[error_code]
pub enum CustomError {
    #[msg("Lock-up period has not ended yet")]
    LockupPeriodNotEnded,
}

// Event Definitions
#[event]
pub struct InitializeEvent {
    pub pool: Pubkey,
    pub reward_rate: u64,
    pub reward_multiplier: u64,
}

#[event]
pub struct StakeEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub lockup_period: i64,
}

#[event]
pub struct UnstakeEvent {
    pub user: Pubkey,
    pub amount: u64,
}

#[event]
pub struct ClaimRewardsEvent {
    pub user: Pubkey,
    pub rewards: u64,
}

#[event]
pub struct UpdatePoolEvent {
    pub pool: Pubkey,
    pub accumulated_reward_per_share: u64,
}

#[event]
pub struct SetRewardRateEvent {
    pub pool: Pubkey,
    pub new_rate: u64,
}

#[event]
pub struct SetRewardMultiplierEvent {
    pub pool: Pubkey,
    pub new_multiplier: u64,
}
