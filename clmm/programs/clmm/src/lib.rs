use anchor_lang::prelude::*;
mod error;
mod instructions;
mod state;
mod utils;

use instructions::*;

declare_id!("9sfBz349EJEWpqrtFS7KJsgusGfiQBn5UbEJq58DSXvN");

#[program]
mod clmm {

    use super::*;
    pub fn initialize_pool(ctx: Context<InitializePool>, current_price: u64) -> Result<()> {
        instructions::initialize_pool::initialize_pool(ctx, current_price)
    }

    pub fn initialize_tick(ctx: Context<InitializeTick>, tick_index: i32) -> Result<()> {
        instructions::initialize_tick(ctx, tick_index)
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        tick_lower: i32,
        tick_upper: i32,
        liquidity: u128,
    ) -> Result<()> {
        instructions::add_liquidity(ctx, tick_lower, tick_upper, liquidity)
    }

    pub fn withdraw_liquidity(
        ctx: Context<WithdrawLiquidity>,
        tick_lower: i32,
        tick_upper: i32,
        liquidity_to_remove: u128,
    ) -> Result<()> {
        instructions::withdraw_liquidity(ctx, tick_lower, tick_upper, liquidity_to_remove)
    }

    pub fn swap(
        ctx: Context<Swap>,
        amount_in: u64,
        a_to_b: bool,
        sqrt_price_limit_x64: Option<u128>,
        min_amount_out: Option<u64>,
    ) -> Result<()> {
        instructions::swap(ctx, amount_in, a_to_b, sqrt_price_limit_x64, min_amount_out)
    }
}