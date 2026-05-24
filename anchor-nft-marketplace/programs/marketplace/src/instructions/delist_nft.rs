use anchor_lang::prelude::*;
use anchor_spl::{
    token::{transfer_checked, Token, TransferChecked},
    token_interface::{Mint, TokenAccount},
};

use crate::{
    errors::MarketplaceError,
    states::{Listing, Marketplace},
};

#[derive(Accounts)]
pub struct DelistNft<'info> {
    /// The NFT mint account being delisted
    pub nft: InterfaceAccount<'info, Mint>,

    /// The listing account to be closed
    /// - Must match the PDA derived from marketplace, seller, and NFT
    /// - Closed and rent refunded to seller after successful delisting
    #[account(
        mut,
        seeds = [
            b"listing",
            marketplace.key().as_ref(),
            seller.key().as_ref(),
            nft.key().as_ref(),
        ],
        bump,
        close = seller
    )]
    pub listing: Account<'info, Listing>,

    /// Token account holding the NFT during listing
    /// - Owned by the listing PDA
    /// - Will be emptied during delisting
    #[account(
        mut,
        associated_token::mint = nft,
        associated_token::authority = listing
    )]
    pub listing_token_account: InterfaceAccount<'info, TokenAccount>,

    /// The seller who originally listed the NFT
    /// - Must be the same as the seller in the listing
    /// - Receives the NFT back and rent refund
    #[account(mut)]
    pub seller: Signer<'info>,

    /// The seller's token account to receive the NFT
    /// - Must be owned by the seller
    /// - Will receive the NFT back
    #[account(
        mut,
        associated_token::mint = nft,
        associated_token::authority = seller,
    )]
    pub seller_token_account: InterfaceAccount<'info, TokenAccount>,

    /// The marketplace state account for validation
    #[account(
        seeds = [b"marketplace"],
        bump = marketplace.bump,
    )]
    pub marketplace: Account<'info, Marketplace>,

    /// Required programs
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> DelistNft<'info> {
    /// Transfer the NFT back to the seller and validate listing state
    /// 
    /// # Returns
    /// * `Result<()>` - Success or error from the transfer
    pub fn transfer_back_nft(&mut self) -> Result<()> {
        // Validate listing is active and seller matches
        require!(
            self.listing.is_active && self.listing.seller == self.seller.key(),
            MarketplaceError::ListingNotActive
        );

        // Create seeds for PDA signing
        let marketplace = self.marketplace.key();
        let seller = self.seller.key();
        let nft = self.nft.key();
        let listing_seeds: &[&[u8]] = &[
            b"listing",
            marketplace.as_ref(),
            seller.as_ref(),
            nft.as_ref(),
            &[self.listing.bump],
        ];
        let signer = &[listing_seeds];

        // Create CPI context with PDA signer
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.listing_token_account.to_account_info(),
                mint: self.nft.to_account_info(),
                to: self.seller_token_account.to_account_info(),
                authority: self.listing.to_account_info(),
            },
            signer,
        );

        // Transfer the NFT back to seller
        transfer_checked(cpi_ctx, 1, self.nft.decimals)
    }
}