use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Escrow {
    pub seed: u64,      // For indexing multiple escrows
    pub maker: Pubkey,  // Creator of the escrow
    pub mint_a: Pubkey, // Token the maker is offering
    pub mint_b: Pubkey, // Token the maker expects
    pub receive: u64,   // Amount of Token B expected
    pub bump: u8,       // PDA bump
}