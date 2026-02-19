use crate::state::Tick;
use crate::{error::CLMMError, state::Pool, utils::*};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{mint_to, transfer, Mint, MintTo, Token, TokenAccount, Transfer};

pub fn add_liquidity(
    ctx: Context<AddLiquidity>,
    tick_lower: i32,
    tick_upper: i32,
    liquidity: u128,
) -> Result<()> {
    require!(tick_lower < tick_upper, CLMMError::TickMismatch);

    let mut pool = ctx.accounts.pool.load_mut()?;
    let tick_lower_acc = &mut ctx.accounts.tick_lower_acc;
    let tick_upper_acc = &mut ctx.accounts.tick_upper_acc;
    let token_a_mint = ctx.accounts.token_mint_a.key();
    let token_b_mint = ctx.accounts.token_mint_b.key();

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
    require!(pool.mint_a == token_a_mint, CLMMError::InvalidTokenMint);
    require!(pool.mint_b == token_b_mint, CLMMError::InvalidTokenMint);

    require!(
        tick_lower % TICK_SPACING as i32 == 0 && tick_upper % TICK_SPACING as i32 == 0,
        CLMMError::UnalignedTick
    );

    tick_lower_acc.liquidity_net = tick_lower_acc
        .liquidity_net
        .checked_add(liquidity as i128)
        .ok_or(CLMMError::ArithmeticOverflow)?;

    tick_upper_acc.liquidity_net = tick_upper_acc
        .liquidity_net
        .checked_sub(liquidity as i128)
        .ok_or(CLMMError::ArithmeticOverflow)?;

    if pool.current_tick >= tick_lower && pool.current_tick <= tick_upper {
        pool.active_liquidity = pool
            .active_liquidity
            .checked_add(liquidity)
            .ok_or(CLMMError::ArithmeticOverflow)?;
    }

    let authority_seeds = &[
        b"authority",
        token_a_mint.as_ref(),
        token_b_mint.as_ref(),
        &[ctx.bumps.authority],
    ];

    let sqrt_price_lower_x64 = tick_to_sqrt_price_x64(tick_lower)?;
    let sqrt_price_upper_x64 = tick_to_sqrt_price_x64(tick_upper)?;

    let (amount_a, amount_b) = calculate_liquidity_amounts(
        pool.sqrt_price_x64,
        sqrt_price_lower_x64,
        sqrt_price_upper_x64,
        liquidity,
    )?;

    if amount_a != 0 {
        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.token_account_a.to_account_info(),
                    to: ctx.accounts.vault_a.to_account_info(),
                    authority: ctx.accounts.liquidity_provider.to_account_info(),
                },
            ),
            amount_a,
        )?;
    }
    if amount_b != 0 {
        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.token_account_b.to_account_info(),
                    to: ctx.accounts.vault_b.to_account_info(),
                    authority: ctx.accounts.liquidity_provider.to_account_info(),
                },
            ),
            amount_b,
        )?;
    }

    // LP token calculation based on actual token value contributed
    let mint_amount = if pool.total_lp_issued == 0 {
        if amount_a > 0 && amount_b > 0 {
            // Use geometric mean for initial liquidity: sqrt(amount_a * amount_b)
            let product = (amount_a as u128)
                .checked_mul(amount_b as u128)
                .ok_or(CLMMError::ArithmeticOverflow)?;
            integer_sqrt(product)
        } else {
            std::cmp::max(amount_a, amount_b)
        }
    } else {
        // Calculate LP tokens based on proportional contribution to pool value
        let pool_balance_a = ctx.accounts.vault_a.amount;
        let pool_balance_b = ctx.accounts.vault_b.amount;

        if pool_balance_a == 0 && pool_balance_b == 0 {
            return Err(CLMMError::PoolEmpty.into());
        }

        let share_from_a = if pool_balance_a > 0 {
            (amount_a as u128)
                .checked_mul(pool.total_lp_issued as u128)
                .ok_or(CLMMError::ArithmeticOverflow)?
                .checked_div(pool_balance_a as u128)
                .ok_or(CLMMError::ArithmeticOverflow)?
        } else {
            0
        };

        let share_from_b = if pool_balance_b > 0 {
            (amount_b as u128)
                .checked_mul(pool.total_lp_issued as u128)
                .ok_or(CLMMError::ArithmeticOverflow)?
                .checked_div(pool_balance_b as u128)
                .ok_or(CLMMError::ArithmeticOverflow)?
        } else {
            0
        };

        std::cmp::min(share_from_a, share_from_b)
            .try_into()
            .map_err(|_| CLMMError::ArithmeticOverflow)?
    };

    pool.total_lp_issued = pool
        .total_lp_issued
        .checked_add(mint_amount)
        .ok_or(CLMMError::ArithmeticOverflow)?;
    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.lp_token_mint.to_account_info(),
                to: ctx.accounts.lp_token_account.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
            &[authority_seeds],
        ),
        mint_amount,
    )?;
    Ok(())
}

#[derive(Accounts)]
#[instruction(tick_lower:i32,tick_upper:i32)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub liquidity_provider: Signer<'info>,

    /// CHECK: This holds the complete authority for vault A and B and lp_mint_token
    #[account(seeds = [b"authority", token_mint_a.key().as_ref(), token_mint_b.key().as_ref()], bump)]
    pub authority: UncheckedAccount<'info>,

    pub token_mint_a: Account<'info, Mint>,
    pub token_mint_b: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"pool", token_mint_a.key().as_ref(), token_mint_b.key().as_ref()],
        bump
    )]
    pub pool: AccountLoader<'info, Pool>,

    #[account(
        mut,
        seeds = [b"tick",pool.key().as_ref(),&tick_lower.to_le_bytes()],
        bump
    )]
    pub tick_lower_acc: Account<'info, Tick>,

    #[account(
        mut,
        seeds = [b"tick",pool.key().as_ref(),&tick_upper.to_le_bytes()],
        bump
    )]
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
        init_if_needed,
        payer = liquidity_provider,
        associated_token::mint = lp_token_mint,
        associated_token::authority = liquidity_provider
    )]
    pub lp_token_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}