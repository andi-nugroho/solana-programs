use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Marketplace {
    /// The admin public key who can manage the marketplace
    pub admin: Pubkey,
    
    /// Fee percentage charged on each sale (0-100)
    /// This percentage is taken from the sale price and sent to treasury
    pub fee_percentage: u8,
    
    /// PDA bump seed for the marketplace account
    /// Used for deterministic address generation
    pub bump: u8,
    
    /// PDA bump seed for the treasury account
    /// Used for deterministic address generation of the treasury
    pub treasury_bump: u8,
}
