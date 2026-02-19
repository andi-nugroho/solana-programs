use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::error::CLMMError;
use crate::state::Pool;
use crate::utils::{price_to_sqrt_price_x64, sqrt_price_x64_to_tick};

//current price is the current price of a wrt b while creating the pool.
pub fn initialize_pool(ctx: Context<InitializePool>, current_price: u64) -> Result<()> {
    require!(
        ctx.accounts.token_a_mint.key() != ctx.accounts.token_b_mint.key(),
        CLMMError::SameTokenMint
    );

    let curr_sqrt_price_x64 = price_to_sqrt_price_x64(current_price)?;
    let current_tick = sqrt_price_x64_to_tick(curr_sqrt_price_x64)?;

    let mut pool = ctx.accounts.pool.load_init()?;
    pool.mint_a = ctx.accounts.token_a_mint.key();
    pool.mint_b = ctx.accounts.token_b_mint.key();
    pool.vault_a = ctx.accounts.vault_a.key();
    pool.vault_b = ctx.accounts.vault_b.key();
    pool.lp_mint = ctx.accounts.lp_token_mint.key();
    pool.total_lp_issued = 0;
    pool.bump = ctx.bumps.pool;
    pool.pool_authority = ctx.accounts.authority.key();
    pool.sqrt_price_x64 = curr_sqrt_price_x64;
    pool.current_tick = current_tick;
    pool.active_liquidity = 0;

    Ok(())
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,

    /// CHECK: This holds the complete authority for vault A and B and lp_mint_token
    #[account(seeds = [b"authority", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()], bump)]
    pub authority: UncheckedAccount<'info>,

    pub token_a_mint: Account<'info, Mint>,
    pub token_b_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = initializer,
        seeds = [b"vault_token", token_a_mint.key().as_ref(), token_b_mint.key().as_ref(), b"A"],
        bump,
        token::mint = token_a_mint,
        token::authority = authority
    )]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = initializer,
        seeds = [b"vault_token", token_a_mint.key().as_ref(), token_b_mint.key().as_ref(), b"B"],
        bump,
        token::mint = token_b_mint,
        token::authority = authority
    )]
    pub vault_b: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = initializer,
        seeds = [b"lp_mint", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = authority,
        mint::freeze_authority = authority
    )]
    pub lp_token_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = initializer,
        space = 8 + 32 * 6 + 16 * 2 + 8 + 4 + 1 + 3,
        seeds = [b"pool", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump
    )]
    pub pool: AccountLoader<'info, Pool>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}