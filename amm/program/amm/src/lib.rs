use anchor_lang::prelude::*;

mod error;
mod instructions;
mod state;
mod utils;

use instructions::*;

declare_id!("8igYFZBtEYMLPmeeWNo1aFKwhMQfh7aEBJFVspu4vSff");

#[program]
pub mod amm {
    use super::*;

    pub fn initialize_pool(ctx: Context<InitPool>) -> Result<()> {
        instructions::initialize_pool::initialize_pool(ctx)
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        quantity_a: u64,
        quantity_b: u64,
    ) -> Result<()> {
        instructions::add_liquidity::add_liquidity(ctx, quantity_a, quantity_b)
    }

    pub fn withdraw_liquidity(
        ctx: Context<WithdrawLiquidity>,
        lp_token_quantity: u64,
    ) -> Result<()> {
        instructions::withdraw_liquidity::withdraw_liquidity(ctx, lp_token_quantity)
    }

    pub fn swap(
        ctx: Context<Swap>,
        quantity: u64,
        minimum_slippage_quantity: u64,
        is_a_to_b: bool,
    ) -> Result<()> {
        instructions::swap::swap(ctx, quantity, minimum_slippage_quantity, is_a_to_b)
    }
}