use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

use crate::error::AMMError;
use crate::state::AMMPool;

pub fn swap(
    ctx: Context<Swap>,
    quantity: u64,
    min_slippage_quantity: u64,
    is_a_to_b: bool,
) -> Result<()> {
    require!(quantity > 0, AMMError::ZeroAmount);

    // Constant Product Formula: x * y = k
    // When a user swaps dx amount of token x (quantity), he should recieve dy back token y:
    // - dx: amount of input tokens user provides
    // - dy: amount of output tokens user receives
    // After the swap
    // (x + dx) * (y - dy) = x * y
    // Solving for dy:
    //     (x + dx)(y - dy) = x * y
    //     y - dy = (x * y) / (x + dx)
    //     dy = y - (x * y) / (x + dx)
    //     dy = (y * dx) / (x + dx)

    require!(
        ctx.accounts.token_a_mint.key() == ctx.accounts.amm_pool.mint_a,
        AMMError::InvalidTokenMint
    );
    require!(
        ctx.accounts.token_b_mint.key() == ctx.accounts.amm_pool.mint_b,
        AMMError::InvalidTokenMint
    );

    require!(
        ctx.accounts.vault_a.key() == ctx.accounts.amm_pool.vault_a,
        AMMError::InvalidVault
    );
    require!(
        ctx.accounts.vault_b.key() == ctx.accounts.amm_pool.vault_b,
        AMMError::InvalidVault
    );

    let (vault_in, vault_out, user_account_in, user_account_out) = if is_a_to_b {
        require!(
            ctx.accounts.vault_b.amount > min_slippage_quantity,
            AMMError::InsufficientFundsInPool
        );
        (
            &ctx.accounts.vault_a,
            &ctx.accounts.vault_b,
            &ctx.accounts.user_token_account_a,
            &ctx.accounts.user_token_account_b,
        )
    } else {
        require!(
            ctx.accounts.vault_a.amount > min_slippage_quantity,
            AMMError::InsufficientFundsInPool
        );
        (
            &ctx.accounts.vault_b,
            &ctx.accounts.vault_a,
            &ctx.accounts.user_token_account_b,
            &ctx.accounts.user_token_account_a,
        )
    };

    let token_user_receives = vault_out
        .amount
        .checked_mul(quantity)
        .and_then(|p| p.checked_div(vault_in.amount.checked_add(quantity)?))
        .ok_or(AMMError::ArithmeticOverflow)?;

    require!(
        token_user_receives >= min_slippage_quantity,
        AMMError::SlippageExceeded
    );

    let transfer_tokens_from_user_to_vault_ix = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: user_account_in.to_account_info(),
            to: vault_in.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    transfer(transfer_tokens_from_user_to_vault_ix, quantity)?;

    let token_a_mint_key = ctx.accounts.token_a_mint.key();
    let token_b_mint_key = ctx.accounts.token_b_mint.key();
    let seeds: &[&[u8]] = &[
        b"authority",
        token_a_mint_key.as_ref(),
        token_b_mint_key.as_ref(),
        &[ctx.bumps.authority],
    ];
    let signer = &[seeds];

    let transfer_tokens_from_vault_to_user_ix = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: vault_out.to_account_info(),
            to: user_account_out.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        },
        signer,
    );
    transfer(transfer_tokens_from_vault_to_user_ix, token_user_receives)?;

    Ok(())
}

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account()]
    pub user: Signer<'info>,

    #[account()]
    pub token_a_mint: Account<'info, Mint>,

    #[account()]
    pub token_b_mint: Account<'info, Mint>,

    /// CHECK: PDA authority for signing vault transfers
    #[account(
        seeds = [b"authority", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump
    )]
    pub authority: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = token_a_mint,
        associated_token::authority = user
    )]
    pub user_token_account_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_b_mint,
        associated_token::authority = user
    )]
    pub user_token_account_b: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vault_token", token_a_mint.key().as_ref(), token_b_mint.key().as_ref(), b"A"],
        bump
    )]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vault_token", token_a_mint.key().as_ref(), token_b_mint.key().as_ref(), b"B"],
        bump
    )]
    pub vault_b: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"pool", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump
    )]
    pub amm_pool: Account<'info, AMMPool>,

    pub token_program: Program<'info, Token>,
}