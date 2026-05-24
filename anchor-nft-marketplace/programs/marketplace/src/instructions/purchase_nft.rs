use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer_checked, Token, TransferChecked},
    token_interface::{Mint, TokenAccount},
};

use crate::{
    errors::MarketplaceError,
    states::{Listing, Marketplace},
};

/// and collects marketplace fees
#[derive(Accounts)]
pub struct PurchaseNft<'info> {
    /// The NFT mint account being purchased
    pub nft: InterfaceAccount<'info, Mint>,

    /// The listing account being fulfilled
    /// - Contains price and seller information
    /// - Closed after successful purchase
    #[account(
        mut,
        seeds = [
            b"listing",
            marketplace.key().as_ref(),
            seller.key().as_ref(),
            nft.key().as_ref(),
        ],
        bump
    )]
    pub listing: Account<'info, Listing>,

    /// Token account holding the NFT during listing
    /// - Owned by the listing PDA
    /// - NFT transferred from here to buyer
    #[account(
        mut,
        associated_token::mint = nft,
        associated_token::authority = listing,
    )]
    pub listing_token_account: InterfaceAccount<'info, TokenAccount>,

    /// The buyer purchasing the NFT
    /// - Pays for the NFT plus marketplace fees
    /// - Receives the NFT in their token account
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// The buyer's token account to receive the NFT
    /// - Must be owned by the buyer
    /// - Will receive the purchased NFT
    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = nft,
        associated_token::authority = buyer
    )]
    pub buyer_token_account: InterfaceAccount<'info, TokenAccount>,

    /// The seller who listed the NFT
    /// - Receives payment minus marketplace fees
    /// - Validated against the listing's seller field
    /// CHECK: Seller account is validated in the instruction logic
    #[account(mut)]
    pub seller: AccountInfo<'info>,

    /// The marketplace state account
    /// - Contains fee percentage for calculations
    #[account(
        seeds = [b"marketplace"],
        bump = marketplace.bump,
    )]
    pub marketplace: Account<'info, Marketplace>,

    /// Treasury account for collecting marketplace fees
    /// - Receives the calculated fee percentage
    #[account(
        mut,
        seeds = [b"treasury", marketplace.key().as_ref()],
        bump
    )]
    pub treasury: SystemAccount<'info>,

    /// Required programs
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> PurchaseNft<'info> {
    /// Transfer the NFT from listing to buyer
    /// 
    /// # Returns
    /// * `Result<()>` - Success or error from the transfer
    pub fn transfer_nft(&mut self) -> Result<()> {
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
                to: self.buyer_token_account.to_account_info(),
                authority: self.listing.to_account_info(),
            },
            signer,
        );

        // Transfer the NFT to the buyer
        transfer_checked(cpi_ctx, 1, self.nft.decimals)
    }

    /// Transfer SOL payment from buyer to seller and treasury
    /// 
    /// # Returns
    /// * `Result<()>` - Success or error from the transfers
    pub fn transfer_sol(&mut self) -> Result<()> {
        // Calculate marketplace fee (percentage of listing price)
        let fee_lamports = (self.marketplace.fee_percentage as u64)
            .checked_mul(self.listing.price)
            .ok_or(MarketplaceError::MathOverflow)?
            .checked_div(100)
            .ok_or(MarketplaceError::MathOverflow)?;

        // Calculate seller payment (listing price minus fees)
        let seller_lamports = self
            .listing
            .price
            .checked_sub(fee_lamports)
            .ok_or(MarketplaceError::MathOverflow)?;

        // Transfer fee to treasury
        let treasury_transfer_ctx = CpiContext::new(
            self.system_program.to_account_info(),
            Transfer {
                from: self.buyer.to_account_info(),
                to: self.treasury.to_account_info(),
            },
        );
        transfer(treasury_transfer_ctx, fee_lamports)?;

        // Transfer remaining payment to seller
        let seller_transfer_ctx = CpiContext::new(
            self.system_program.to_account_info(),
            Transfer {
                from: self.buyer.to_account_info(),
                to: self.seller.to_account_info(),
            },
        );
        transfer(seller_transfer_ctx, seller_lamports)?;

        Ok(())
    }

    /// Mark the listing as inactive after successful purchase
    /// 
    /// # Returns
    /// * `Result<()>` - Always succeeds
    pub fn delist_nft(&mut self) -> Result<()> {
        // Mark listing as inactive (though it will be closed anyway)
        self.listing.is_active = false;
        Ok(())
    }
}