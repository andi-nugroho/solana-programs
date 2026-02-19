use anchor_lang::prelude::*;
use anchor_spl::token::{burn, transfer, Burn, Mint, Token, TokenAccount, Transfer};

use crate::{
    error::CLMMError,
    state::{Pool, Tick},
    utils::{calculate_liquidity_amounts, tick_to_sqrt_price_x64, TICK_SPACING},
};

pub fn withdraw_liquidity(
    ctx: Context<WithdrawLiquidity>,
    tick_lower: i32,
    tick_upper: i32,
    liquidity_to_remove: u128,
) -> Result<()> {
    require!(tick_upper > tick_lower, CLMMError::TickMismatch);

    let mut pool = ctx.accounts.pool.load_mut()?;
    require!(liquidity_to_remove > 0, CLMMError::ZeroAmount);

    require!(pool.total_lp_issued > 0, CLMMError::PoolEmpty);
    require!(
        tick_lower % TICK_SPACING == 0 && tick_upper % TICK_SPACING as i32 == 0,
        CLMMError::UnalignedTick
    );

    let tick_upper_acc = &mut ctx.accounts.tick_upper_acc;
    let tick_lower_acc = &mut ctx.accounts.tick_lower_acc;
    // Calculate sqrt prices for the position bounds
    let sqrt_price_lower_x64 = tick_to_sqrt_price_x64(tick_lower)?;
    let sqrt_price_upper_x64 = tick_to_sqrt_price_x64(tick_upper)?;

    // LP token calculation: proportional to actual token value being withdrawn
    let (withdraw_amount_a, withdraw_amount_b) = calculate_liquidity_amounts(
        pool.sqrt_price_x64,
        sqrt_price_lower_x64,
        sqrt_price_upper_x64,
        liquidity_to_remove,
    )?;

    let pool_balance_a = ctx.accounts.vault_a.amount;
    let pool_balance_b = ctx.accounts.vault_b.amount;

    // Calculate LP tokens based on proportional value being withdrawn
    let lp_tokens_to_burn = if pool_balance_a > 0 && pool_balance_b > 0 {
        let share_from_a = (withdraw_amount_a as u128)
            .checked_mul(pool.total_lp_issued as u128)
            .ok_or(CLMMError::ArithmeticOverflow)?
            .checked_div(pool_balance_a as u128)
            .ok_or(CLMMError::ArithmeticOverflow)?;

        let share_from_b = (withdraw_amount_b as u128)
            .checked_mul(pool.total_lp_issued as u128)
            .ok_or(CLMMError::ArithmeticOverflow)?
            .checked_div(pool_balance_b as u128)
            .ok_or(CLMMError::ArithmeticOverflow)?;

        std::cmp::max(share_from_a, share_from_b) as u64
    } else if pool_balance_a > 0 {
        ((withdraw_amount_a as u128)
            .checked_mul(pool.total_lp_issued as u128)
            .ok_or(CLMMError::ArithmeticOverflow)?
            .checked_div(pool_balance_a as u128)
            .ok_or(CLMMError::ArithmeticOverflow)?) as u64
    } else if pool_balance_b > 0 {
        ((withdraw_amount_b as u128)
            .checked_mul(pool.total_lp_issued as u128)
            .ok_or(CLMMError::ArithmeticOverflow)?
            .checked_div(pool_balance_b as u128)
            .ok_or(CLMMError::ArithmeticOverflow)?) as u64
    } else {
        return Err(CLMMError::PoolEmpty.into());
    };
    require_eq!(
        tick_lower_acc.index,
        tick_lower,
        CLMMError::InvalidTickIndex
    );
    require_eq!(
        tick_upper_acc.index,
        tick_upper,
        CLMMError::InvalidTickIndex
    );
    require!(
        ctx.accounts.lp_token_account.amount >= lp_tokens_to_burn,
        CLMMError::InsufficientLPTokens
    );
    tick_lower_acc.liquidity_net = tick_lower_acc
        .liquidity_net
        .checked_sub(liquidity_to_remove as i128)
        .ok_or(CLMMError::ArithmeticOverflow)?;

    tick_upper_acc.liquidity_net = tick_upper_acc
        .liquidity_net
        .checked_add(liquidity_to_remove as i128)
        .ok_or(CLMMError::ArithmeticOverflow)?;

    if tick_lower <= pool.current_tick && pool.current_tick <= tick_upper {
        pool.active_liquidity = pool
            .active_liquidity
            .checked_sub(liquidity_to_remove)
            .ok_or(CLMMError::ArithmeticOverflow)?;
    }

    // Use the amounts already calculated above
    let (amount_a, amount_b) = (withdraw_amount_a, withdraw_amount_b);
    let token_a_mint = ctx.accounts.token_mint_a.key();
    let token_b_mint = ctx.accounts.token_mint_b.key();

    let seeds: &[&[u8]] = &[
        b"authority",
        token_a_mint.as_ref(),
        token_b_mint.as_ref(),
        &[ctx.bumps.authority],
    ];

    let signer = &[seeds];
    if amount_a != 0 {
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault_a.to_account_info(),
                    to: ctx.accounts.token_account_a.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
                signer,
            ),
            amount_a,
        )?;
    }

    if amount_b != 0 {
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault_b.to_account_info(),
                    to: ctx.accounts.token_account_b.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
                signer,
            ),
            amount_b,
        )?;
    }

    burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.lp_token_mint.to_account_info(),
                from: ctx.accounts.lp_token_account.to_account_info(),
                authority: ctx.accounts.liquidity_provider.to_account_info(),
            },
        ),
        lp_tokens_to_burn,
    )?;

    pool.total_lp_issued = pool
        .total_lp_issued
        .checked_sub(lp_tokens_to_burn)
        .ok_or(CLMMError::ArithmeticOverflow)?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(tick_lower:i32,tick_upper:i32)]
pub struct WithdrawLiquidity<'info> {
    #[account(mut)]
    pub liquidity_provider: Signer<'info>,

    pub token_mint_a: Account<'info, Mint>,
    pub token_mint_b: Account<'info, Mint>,

    /// CHECK: This holds the complete authority for vault A and B and lp_mint_token
    #[account(seeds = [b"authority", token_mint_a.key().as_ref(), token_mint_b.key().as_ref()], bump)]
    pub authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"pool", token_mint_a.key().as_ref(), token_mint_b.key().as_ref()],
        bump
    )]
    pub pool: AccountLoader<'info, Pool>,

    #[account(
        mut,
        seeds = [b"tick",pool.key().as_ref(), &tick_lower.to_le_bytes()],
        bump)]
    pub tick_lower_acc: Account<'info, Tick>,
    #[account(
        mut,
        seeds = [b"tick",pool.key().as_ref(), &tick_upper.to_le_bytes()],
        bump)]
    pub tick_upper_acc: Account<'info, Tick>,

    #[account(
        mut,
        associated_token::mint = token_mint_a,
        associated_token::authority = liquidity_provider
    )]
    pub token_account_a: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = token_mint_b,
        associated_token::authority = liquidity_provider
    )]
    pub token_account_b: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [b"vault_token", token_mint_a.key().as_ref(), token_mint_b.key().as_ref(), b"A"],
        bump,
        token::mint = token_mint_a,
        token::authority = authority
    )]
    pub vault_a: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [b"vault_token", token_mint_a.key().as_ref(), token_mint_b.key().as_ref(), b"B"],
        bump,
        token::mint = token_mint_b,
        token::authority = authority
    )]
    pub vault_b: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [b"lp_mint", token_mint_a.key().as_ref(), token_mint_b.key().as_ref()],
        bump,
    )]
    pub lp_token_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = lp_token_mint,
        associated_token::authority = liquidity_provider
    )]
    pub lp_token_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}