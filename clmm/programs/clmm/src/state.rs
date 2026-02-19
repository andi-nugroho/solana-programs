use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

#[account(zero_copy)]
#[derive(Debug, Default)]
#[repr(C)]
pub struct Pool {
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub vault_a: Pubkey,
    pub vault_b: Pubkey,
    pub lp_mint: Pubkey,
    pub pool_authority: Pubkey,
    pub sqrt_price_x64: u128,
    pub active_liquidity: u128,
    pub total_lp_issued: u64,
    pub current_tick: i32,
    pub bump: u8,
    pub _padding: [u8; 3],
}

#[account]
pub struct Tick {
    pub sqrt_price_x64: u128,
    pub liquidity_net: i128,
    pub index: i32,
    pub bump: u8,
}