use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

use crate::{
    error::CLMMError,
    state::{Pool, Tick},
    utils::{compute_swap_step, tick_to_sqrt_price_x64},
};

pub fn swap(
    ctx: Context<Swap>,
    amount_in: u64,
    a_to_b: bool,
    sqrt_price_limit_x64: Option<u128>,
    min_amount_out: Option<u64>,
) -> Result<()> {
    require!(amount_in > 0, CLMMError::ZeroAmount);
    require!(
        !ctx.remaining_accounts.is_empty(),
        CLMMError::MissingTickAccounts
    );

    //assuming tick acc's are passed in order
    //ascending order for A to B and descending order for B to A
    let mut tick_infos = vec![];
    for account_info in ctx.remaining_accounts.iter() {
        let data = account_info.data.borrow_mut();
        let mut tick_data_slice = &data[8..]; // skip the discriminator
        let tick: Tick = Tick::try_deserialize(&mut tick_data_slice)?;
        tick_infos.push((account_info.clone(), tick));
    }

    let mut pool = ctx.accounts.pool.load_mut()?;
    let sqrt_price_limit =
        sqrt_price_limit_x64.unwrap_or_else(|| if a_to_b { 1 } else { u128::MAX });

    let mut curr_sqrt_price_x64 = pool.sqrt_price_x64;
    let mut curr_tick = pool.current_tick;
    let mut liquidity = pool.active_liquidity;

    let mut total_amount_in: u128 = 0;
    let mut total_amount_out: u128 = 0;
    let mut remaining_amount: u128 = amount_in as u128;
    let token_mint_a = ctx.accounts.token_mint_a.key();
    let token_mint_b = ctx.accounts.token_mint_b.key();

    require_keys_eq!(pool.mint_a, token_mint_a, CLMMError::InvalidTokenMint);
    require_keys_eq!(pool.mint_b, token_mint_b, CLMMError::InvalidTokenMint);
    require!(
        pool.active_liquidity > 0,
        CLMMError::InsufficientFundsInPool
    );

    for (_tick_acc_info, tick) in tick_infos {
        let next_sqrt_price_x64 = tick_to_sqrt_price_x64(tick.index)?;

        if (a_to_b && next_sqrt_price_x64 < sqrt_price_limit)
            || (!a_to_b && next_sqrt_price_x64 > sqrt_price_limit)
        {
            break;
        }

        let (new_sqrt_price, computed_amount_in, computed_amount_out) = compute_swap_step(
            curr_sqrt_price_x64,
            next_sqrt_price_x64,
            liquidity,
            remaining_amount as u128,
            a_to_b,
        )?;

        curr_sqrt_price_x64 = new_sqrt_price;
        remaining_amount = remaining_amount
            .checked_sub(computed_amount_in)
            .ok_or(CLMMError::ArithmeticOverflow)?;
        total_amount_in = total_amount_in
            .checked_add(computed_amount_in)
            .ok_or(CLMMError::ArithmeticOverflow)?;
        total_amount_out = total_amount_out
            .checked_add(computed_amount_out)
            .ok_or(CLMMError::ArithmeticOverflow)?;

        if curr_sqrt_price_x64 == next_sqrt_price_x64 {
            curr_tick = tick.index;
            if a_to_b {
                liquidity = liquidity
                    .checked_sub(tick.liquidity_net as u128)
                    .ok_or(CLMMError::ArithmeticOverflow)?;
            } else {
                liquidity = liquidity
                    .checked_add(tick.liquidity_net as u128)
                    .ok_or(CLMMError::ArithmeticOverflow)?;
            }
        }
    }

    let total_amount_in: u64 = total_amount_in
        .try_into()
        .map_err(|_| CLMMError::AmountTooLarge)?;
    let total_amount_out: u64 = total_amount_out
        .try_into()
        .map_err(|_| CLMMError::AmountTooLarge)?;

    if let Some(min_out) = min_amount_out {
        require!(total_amount_out >= min_out, CLMMError::SlippageExceeded);
    }

    require!(total_amount_out > 0, CLMMError::ZeroSwapOutput);
    pool.sqrt_price_x64 = curr_sqrt_price_x64;
    pool.current_tick = curr_tick;
    pool.active_liquidity = liquidity;

    let seeds = &[
        b"authority",
        token_mint_a.as_ref(),
        token_mint_b.as_ref(),
        &[ctx.bumps.authority],
    ];
    let signer = &[&seeds[..]];
    if a_to_b {
        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.token_account_a.to_account_info(),
                    to: ctx.accounts.vault_a.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            total_amount_in,
        )?;

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
            total_amount_out,
        )?;
    } else {
        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.token_account_b.to_account_info(),
                    to: ctx.accounts.vault_b.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            total_amount_in,
        )?;

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
            total_amount_out,
        )?;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct Swap<'info> {
    pub user: Signer<'info>,

    pub token_mint_a: Account<'info, Mint>,
    pub token_mint_b: Account<'info, Mint>,

    /// CHECK: vaults are owned by program authority
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
        associated_token::mint = token_mint_a,
        associated_token::authority = user
    )]
    pub token_account_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_mint_b,
        associated_token::authority = user
    )]
    pub token_account_b: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vault_token", token_mint_a.key().as_ref(), token_mint_b.key().as_ref(), b"A"],
        bump,
        token::mint = token_mint_a,
        token::authority = authority
    )]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vault_token", token_mint_a.key().as_ref(), token_mint_b.key().as_ref(), b"B"],
        bump,
        token::mint = token_mint_b,
        token::authority = authority
    )]
    pub vault_b: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    // tick accounts will be passed in as remaining_accounts
}