use anchor_lang::{
    prelude::*,
    solana_program::{
        native_token::LAMPORTS_PER_SOL,
        program::{invoke, invoke_signed},
        system_instruction::transfer,
    },
};
use anchor_spl::token::{self as token, Mint, MintTo, Token, TokenAccount};

declare_id!("Fn5jnsXvawRwwBHDMMkAAHoiaRpesE6zLVVXdhKarH1d");

const SECONDS_PER_DAY: u64 = 86_400;
const POINTS_PER_SOL_PER_DAY: u64 = 1_000_000; // 1 point = 10^6 micro points
const POINTS_PER_REWARD: u64 = 10; // 10 points = 1 reward token
const REWARD_TOKEN_DECIMALS: u64 = 6; // change it according to spl token

#[derive(Accounts)]
pub struct CreateStakeAccount<'info> {
    #[account(mut)]
    user: Signer<'info>,
    #[account(init, payer = user, space = 8 + 32 + 8 + 8 + 8 + 1, seeds = [b"staked_account", user.key().as_ref()], bump)]
    user_stake_account: Account<'info, StakeAccount>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    user: Signer<'info>,
    #[account(mut, seeds = [b"staked_account", user.key().as_ref()], bump = user_stake_account.bump)]
    user_stake_account: Account<'info, StakeAccount>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    user: Signer<'info>,
    #[account(mut, seeds = [b"staked_account", user.key().as_ref()], bump = user_stake_account.bump)]
    user_stake_account: Account<'info, StakeAccount>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(mut)]
    user: Signer<'info>,
    #[account(mut, seeds = [b"staked_account", user.key().as_ref()], bump = user_stake_account.bump)]
    user_stake_account: Account<'info, StakeAccount>,
    #[account(mut)]
    reward_mint: Account<'info, Mint>,
    /// CHECK: authority of the mint to transfer amount
    #[account(seeds = [b"mint_authority"], bump)]
    mint_authority: UncheckedAccount<'info>,
    #[account(mut)]
    user_token_account: Account<'info, TokenAccount>,
    token_program: Program<'info, Token>,
}

#[account]
pub struct StakeAccount {
    pub owner: Pubkey,
    pub staked_amount: u64,
    pub total_points: u64,
    pub stake_timestamp: i64,
    pub bump: u8,
}

#[error_code]
pub enum StakeError {
    #[msg("Amount must be greater than 0")]
    InvalidAmount,
    #[msg("Insufficient staked amount")]
    InsufficientStake,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Arithmetic underflow")]
    Underflow,
    #[msg("Invalid timestamp")]
    InvalidTimestamp,
    #[msg("Insufficient points for rewards")]
    InsufficientPoints,
}

#[program]
pub mod staking {

    use super::*;

    pub fn create_stake_account(ctx: Context<CreateStakeAccount>) -> Result<()> {
        let user_new_stake_acc = &mut ctx.accounts.user_stake_account;
        user_new_stake_acc.owner = ctx.accounts.user.key();
        user_new_stake_acc.staked_amount = 0;
        user_new_stake_acc.total_points = 0;
        user_new_stake_acc.stake_timestamp = 0;
        user_new_stake_acc.bump = ctx.bumps.user_stake_account;
        msg!("User Stake account created successfully");
        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, quantity: u64) -> Result<()> {
        require!(quantity > 0, StakeError::InvalidAmount);

        let user_staked_account = &mut ctx.accounts.user_stake_account;
        let clock = Clock::get()?;

        update_points(user_staked_account, clock.unix_timestamp)?;

        anchor_lang::system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.user.to_account_info(),
                    to: user_staked_account.to_account_info(),
                },
            ),
            quantity,
        )?;

        user_staked_account.staked_amount = user_staked_account
            .staked_amount
            .checked_add(quantity)
            .ok_or(StakeError::Overflow)?;
        user_staked_account.stake_timestamp = clock.unix_timestamp;

        msg!(
            "Staked {} lamports. Total staked: {}, Total points: {}",
            quantity,
            user_staked_account.staked_amount,
            user_staked_account.total_points / 1_000_000
        );
        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>, quantity: u64) -> Result<()> {
        require!(quantity > 0, StakeError::InvalidAmount);

        let user_staked_acc = &mut ctx.accounts.user_stake_account;
        let user_acc = &mut ctx.accounts.user;
        let clock = Clock::get()?;

        require!(
            user_staked_acc.staked_amount >= quantity,
            StakeError::InsufficientStake
        );

        update_points(user_staked_acc, clock.unix_timestamp)?;

        let user_key = user_acc.key();

        **user_staked_acc
            .to_account_info()
            .try_borrow_mut_lamports()? -= quantity;
        **user_acc.to_account_info().try_borrow_mut_lamports()? += quantity;

        user_staked_acc.staked_amount = user_staked_acc
            .staked_amount
            .checked_sub(quantity)
            .ok_or(StakeError::Underflow)?;
        user_staked_acc.stake_timestamp = clock.unix_timestamp;

        msg!(
            "Unstaked {} lamports. Remaining staked: {}, Total points: {}",
            quantity,
            user_staked_acc.staked_amount,
            user_staked_acc.total_points / 1_000_000
        );
        Ok(())
    }

    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        let user_staked_acc = &mut ctx.accounts.user_stake_account;
        let clock = Clock::get()?;

        update_points(user_staked_acc, clock.unix_timestamp)?;

        let available_rewards = user_staked_acc.total_points / POINTS_PER_REWARD;
        require!(available_rewards > 0, StakeError::InsufficientPoints);

        let reward_amount = available_rewards * 10u64.pow(REWARD_TOKEN_DECIMALS as u32);

        let authority_bump = [ctx.bumps.mint_authority];
        let authority_seeds = &[b"mint_authority".as_ref(), &authority_bump];

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.reward_mint.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
                &[authority_seeds],
            ),
            reward_amount,
        )?;

        user_staked_acc.total_points = user_staked_acc.total_points % POINTS_PER_REWARD;
        user_staked_acc.stake_timestamp = clock.unix_timestamp;

        msg!("Minted {} reward tokens to user", available_rewards);
        Ok(())
    }
}

pub fn update_points(user_staked_account: &mut StakeAccount, current_time: i64) -> Result<()> {
    let time_elapsed = current_time
        .checked_sub(user_staked_account.stake_timestamp)
        .ok_or(StakeError::Overflow)?;

    if time_elapsed > 0 && user_staked_account.staked_amount > 0 {
        let points = calculate_points(user_staked_account.staked_amount, time_elapsed)?;
        user_staked_account.total_points = user_staked_account
            .total_points
            .checked_add(points)
            .ok_or(StakeError::Overflow)?;
    }

    user_staked_account.stake_timestamp = current_time;
    Ok(())
}

pub fn calculate_points(staked_quantity: u64, elapsed_seconds: i64) -> Result<u64> {
    //Points = (staked_amount_in_sol * days * points_per_day in sol)
    let points = (staked_quantity as u128)
        .checked_mul(elapsed_seconds as u128)
        .ok_or(StakeError::Overflow)?
        .checked_mul(POINTS_PER_SOL_PER_DAY as u128)
        .ok_or(StakeError::Overflow)?
        .checked_div(LAMPORTS_PER_SOL as u128)
        .ok_or(StakeError::Overflow)?
        .checked_div(SECONDS_PER_DAY as u128)
        .ok_or(StakeError::Overflow)?;

    Ok(points as u64)
}
