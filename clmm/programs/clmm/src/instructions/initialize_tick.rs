use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::{
    state::{Pool, Tick},
    utils::tick_to_sqrt_price_x64,
};

pub fn initialize_tick(ctx: Context<InitializeTick>, tick_index: i32) -> Result<()> {
    let tick = &mut ctx.accounts.tick;

    let sqrt_price_x64 = tick_to_sqrt_price_x64(tick_index)?;

    tick.index = tick_index;
    tick.sqrt_price_x64 = sqrt_price_x64;
    tick.liquidity_net = 0;
    tick.bump = ctx.bumps.tick;

    Ok(())
}

#[derive(Accounts)]
#[instruction(tick_index: i32)]
pub struct InitializeTick<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub token_a_mint: Account<'info, Mint>,
    pub token_b_mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [b"pool", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump
    )]
    pub pool: AccountLoader<'info, Pool>,

    #[account(
        init,
        payer = payer,
        space = 8 + 16 + 16 + 4 + 1,
        seeds = [b"tick",pool.key().as_ref(), &tick_index.to_le_bytes()],
        bump
    )]
    pub tick: Account<'info, Tick>,
    pub system_program: Program<'info, System>,
}