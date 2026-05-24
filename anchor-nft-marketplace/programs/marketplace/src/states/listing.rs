use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Listing {
    /// The seller's public key who listed the NFT
    pub seller: Pubkey,
    
    /// The mint address of the NFT being sold
    pub mint: Pubkey,
    
    /// The listing price in lamports
    pub price: u64,
    
    /// PDA bump seed for this listing account
    /// Used for deterministic address generation
    pub bump: u8,
    
    /// Whether this listing is currently active
    /// Set to false when purchased or delisted
    pub is_active: bool,
}
