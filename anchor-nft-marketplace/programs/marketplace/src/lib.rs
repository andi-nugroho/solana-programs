use anchor_lang::prelude::*;

declare_id!("broQPt5f3vtMniWxwJLeHK5X56pGgor4Qmpd9jMVLYT");

pub mod instructions;
pub mod states;
pub mod constants;
pub mod errors;

pub use instructions::*;

#[program]
pub mod marketplace {
    use super::*;

    pub fn initialize_marketplace(ctx: Context<InitializeMarketplace>, fee_percentage: u8) -> Result<()> {
        ctx.accounts.initialize_marketplace(fee_percentage, ctx.bumps)?;
        Ok(())
    }


    pub fn list_nft(ctx: Context<ListNft>, price: u64) -> Result<()> {
        ctx.accounts.initialize_listing(price, ctx.bumps)?;
        ctx.accounts.transfer_nft()
    }


    pub fn delist_nft(ctx: Context<DelistNft>) -> Result<()> {
        ctx.accounts.transfer_back_nft()
    }

    pub fn purchase_nft(ctx: Context<PurchaseNft>) -> Result<()> {
        ctx.accounts.transfer_nft()?;
        ctx.accounts.transfer_sol()?;
        ctx.accounts.delist_nft()
    }
}