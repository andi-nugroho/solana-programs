use anchor_lang::prelude::*;
use anchor_spl::token::{
    burn, close_account, transfer, Burn, CloseAccount, Mint, Token, TokenAccount, Transfer,
};

use crate::{amm, error::AMMError, state::AMMPool};

pub fn withdraw_liquidity(ctx: Context<WithdrawLiquidity>, lp_token_quantity: u64) -> Result<()> {
    let liquidity_provider = &mut ctx.accounts.liquidity_provider;
    let token_account_a = &mut ctx.accounts.token_a_account;
    let token_account_b = &mut ctx.accounts.token_b_account;
    let vault_a = &mut ctx.accounts.vault_a;
    let vault_b = &mut ctx.accounts.vault_b;
    let amm_pool = &mut ctx.accounts.amm_pool;
    let lp_token_account = &mut ctx.accounts.lp_token_account;

    require!(
        vault_a.amount > 0 && vault_b.amount > 0,
        AMMError::PoolEmpty
    );
    require!(
        lp_token_quantity <= lp_token_account.amount,
        AMMError::InsufficientLPTokens
    );
    require!(
        ctx.accounts.token_a_mint.key() == amm_pool.mint_a,
        AMMError::InvalidTokenMint
    );
    require!(
        ctx.accounts.token_b_mint.key() == amm_pool.mint_b,
        AMMError::InvalidTokenMint
    );

    let token_a_quantity_to_release = lp_token_quantity
        .checked_mul(vault_a.amount)
        .and_then(|v| v.checked_div(amm_pool.total_lp_issued))
        .ok_or(AMMError::ArithmeticOverflow)?;

    let token_b_quantity_to_release = lp_token_quantity
        .checked_mul(vault_b.amount)
        .and_then(|v| v.checked_div(amm_pool.total_lp_issued))
        .ok_or(AMMError::ArithmeticOverflow)?;

    let burn_redeemed_lp_tokens = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Burn {
            mint: ctx.accounts.lp_token_mint.to_account_info(),
            from: lp_token_account.to_account_info(),
            authority: liquidity_provider.to_account_info(),
        },
    );

    burn(burn_redeemed_lp_tokens, lp_token_quantity)?;
    amm_pool.total_lp_issued -= lp_token_quantity;

    let seeds: &[&[u8]] = &[
        b"authority",
        amm_pool.mint_a.as_ref(),
        amm_pool.mint_b.as_ref(),
        &[ctx.bumps.authority],
    ];
    let signer_seeds = &[seeds];

    let transfer_quantiy_a_to_user_ix = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: vault_a.to_account_info(),
            to: token_account_a.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        },
        signer_seeds,
    );

    let transfer_quantiy_b_to_user_ix = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: vault_b.to_account_info(),
            to: token_account_b.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        },
        signer_seeds,
    );

    transfer(transfer_quantiy_a_to_user_ix, token_a_quantity_to_release)?;
    transfer(transfer_quantiy_b_to_user_ix, token_b_quantity_to_release)?;

    if lp_token_quantity == lp_token_account.amount {
        let close_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            CloseAccount {
                account: lp_token_account.to_account_info(),
                destination: ctx.accounts.liquidity_provider.to_account_info(),
                authority: ctx.accounts.liquidity_provider.to_account_info(),
            },
        );
        close_account(close_ctx)?;
    }
    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawLiquidity<'info> {
    pub liquidity_provider: Signer<'info>,

    #[account()]
    pub token_a_mint: Account<'info, Mint>,
    #[account()]
    pub token_b_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"pool",token_a_mint.key().as_ref(),token_b_mint.key().as_ref()],
        bump)]
    pub amm_pool: Account<'info, AMMPool>,

    /// CHECK: This holds the complete authority for vault A and B and lp_mint_token
    #[account(seeds = [b"authority", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()], bump)]
    pub authority: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = token_a_mint,
        associated_token::authority = liquidity_provider)]
    pub token_a_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_b_mint,
        associated_token::authority = liquidity_provider)]
    pub token_b_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vault_token", token_a_mint.key().as_ref(), token_b_mint.key().as_ref(), b"A"],
        bump,
        token::mint = token_a_mint,
        token::authority = authority
    )]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vault_token", token_a_mint.key().as_ref(), token_b_mint.key().as_ref(), b"B"],
        bump,
        token::mint = token_b_mint,
        token::authority = authority
    )]
    pub vault_b: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"lp_mint", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
    )]
    pub lp_token_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = lp_token_mint,
        associated_token::authority = liquidity_provider)]
    pub lp_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}