use crate::error::AMMError;
use crate::state::AMMPool;
use crate::utils::integer_sqrt;
use anchor_lang::prelude::*;
use anchor_spl::token::{mint_to, transfer, Mint, MintTo, Token, TokenAccount, Transfer};

pub fn add_liquidity(ctx: Context<AddLiquidity>, quantity_a: u64, quantity_b: u64) -> Result<()> {
    require!(
        ctx.accounts.token_a_mint.key() == ctx.accounts.amm_pool.mint_a,
        AMMError::InvalidTokenMint
    );
    require!(
        ctx.accounts.token_b_mint.key() == ctx.accounts.amm_pool.mint_b,
        AMMError::InvalidTokenMint
    );

    require!(quantity_a > 0 && quantity_b > 0, AMMError::ZeroAmount);

    let liquidity_provider = &mut ctx.accounts.liquidity_provider;
    let token_account_a = &mut ctx.accounts.token_a_account;
    let token_account_b = &mut ctx.accounts.token_b_account;
    let vault_a = &mut ctx.accounts.vault_a;
    let vault_b = &mut ctx.accounts.vault_b;
    let amm_pool = &mut &mut ctx.accounts.amm_pool;
    let lp_token_mint = &mut ctx.accounts.lp_token_mint;

    let tokens_to_issue: u64;
    if vault_a.amount == 0 && vault_b.amount == 0 {
        let scaling_factor = 10u128.pow(lp_token_mint.decimals as u32);
        let total = (quantity_a as u128) * (quantity_b as u128) * scaling_factor;
        tokens_to_issue = integer_sqrt(total);
    } else {
        require!(
            (quantity_a * vault_b.amount == quantity_b * vault_a.amount),
            AMMError::InvalidLiquidity
        );

        let lp_tokens_to_issue_based_on_token_a = amm_pool
            .total_lp_issued
            .checked_mul(quantity_a)
            .and_then(|result| result.checked_div(vault_a.amount))
            .ok_or(AMMError::ArithmeticOverflow)?;

        let lp_tokens_to_issue_based_on_token_b = amm_pool
            .total_lp_issued
            .checked_mul(quantity_b)
            .and_then(|result| result.checked_div(vault_b.amount))
            .ok_or(AMMError::ArithmeticOverflow)?;

        tokens_to_issue = std::cmp::min(
            lp_tokens_to_issue_based_on_token_a,
            lp_tokens_to_issue_based_on_token_b,
        );
    }

    let transfer_quantiy_a_to_vault_ix = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: token_account_a.to_account_info(),
            to: vault_a.to_account_info(),
            authority: liquidity_provider.to_account_info(),
        },
    );

    let transfer_quantiy_b_to_vault_ix = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: token_account_b.to_account_info(),
            to: vault_b.to_account_info(),
            authority: liquidity_provider.to_account_info(),
        },
    );
    msg!("hi");
    transfer(transfer_quantiy_a_to_vault_ix, quantity_a)?;
    transfer(transfer_quantiy_b_to_vault_ix, quantity_b)?;

    let seeds: &[&[u8]] = &[
        b"authority",
        amm_pool.mint_a.as_ref(),
        amm_pool.mint_b.as_ref(),
        &[ctx.bumps.authority],
    ];
    let signer = &[seeds];

    let mint_lp_tokens_ix = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: lp_token_mint.to_account_info(),
            to: ctx.accounts.lp_token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        },
        signer,
    );

    mint_to(mint_lp_tokens_ix, tokens_to_issue)?;
    amm_pool.total_lp_issued += tokens_to_issue;

    Ok(())
}

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
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