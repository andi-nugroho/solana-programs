
use anchor_lang::prelude::*;

#[account]
pub struct AMMPool {
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub vault_a: Pubkey,
    pub vault_b: Pubkey,
    pub lp_mint: Pubkey,
    pub pool_authority: Pubkey,
    pub total_lp_issued: u64,
    pub bump: u8,
}
