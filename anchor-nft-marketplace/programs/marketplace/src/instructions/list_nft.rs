use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{MasterEditionAccount, Metadata, MetadataAccount},
    token::{transfer_checked, Token, TransferChecked},
    token_interface::{Mint, TokenAccount},
};

use crate::{
    errors::MarketplaceError,
    states::{Listing, Marketplace},
};

#[derive(Accounts)]
pub struct ListNft<'info> {
    /// The NFT mint account to be listed
    pub nft: InterfaceAccount<'info, Mint>,

    /// The listing state account
    /// - Stores seller, mint, price, and status information
    /// - Uses PDA with marketplace, seller, and NFT mint as seeds
    #[account(
        init,
        payer = seller,
        space = 8 + Listing::INIT_SPACE,
        seeds = [
            b"listing",
            marketplace.key().as_ref(),
            seller.key().as_ref(),
            nft.key().as_ref(),
        ],
        bump,
    )]
    pub listing: Account<'info, Listing>,

    /// Token account that will hold the NFT during listing
    /// - Owned by the listing PDA for security
    /// - Created as associated token account
    #[account(
        init,
        payer = seller,
        associated_token::mint = nft,
        associated_token::authority = listing,
    )]
    pub listing_token_account: InterfaceAccount<'info, TokenAccount>,

    /// The seller who owns the NFT and is creating the listing
    #[account(mut)]
    pub seller: Signer<'info>,

    /// The seller's token account containing the NFT
    /// - Must be owned by the seller
    /// - Must have the NFT to be listed
    #[account(
        mut,
        associated_token::mint = nft,
        associated_token::authority = seller,
        constraint = seller_token_account.owner == seller.key()
    )]
    pub seller_token_account: InterfaceAccount<'info, TokenAccount>,

    /// The marketplace state account
    /// - Validates this is the correct marketplace instance
    #[account(
        seeds = [b"marketplace"],
        bump = marketplace.bump,
    )]
    pub marketplace: Account<'info, Marketplace>,

    /// The collection mint that this NFT belongs to
    /// - Used for collection verification
    pub collection_mint: InterfaceAccount<'info, Mint>,

    /// The metadata account for the NFT
    /// - Contains collection information and verification status
    /// - Must be from a verified collection
    #[account(
        seeds = [
            b"metadata",
            metadata_program.key().as_ref(),
            nft.key().as_ref(),
        ],
        seeds::program = metadata_program.key(),
        bump,
        constraint = metadata.collection.as_ref().unwrap().key.as_ref() == collection_mint.key().as_ref(),
        constraint = metadata.collection.as_ref().unwrap().verified == true,
    )]
    pub metadata: Account<'info, MetadataAccount>,

    /// The master edition account for the NFT
    /// - Proves this is a valid NFT (not just a token)
    #[account(
        seeds = [
            b"metadata", 
            metadata_program.key().as_ref(),
            nft.key().as_ref(),
            b"edition"
        ],
        seeds::program = metadata_program.key(),
        bump,
    )]
    pub master_edition: Account<'info, MasterEditionAccount>,

    /// Required programs for the instruction
    pub metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> ListNft<'info> {
    /// Transfer the NFT from seller to the listing account
    ///
    /// # Returns
    /// * `Result<()>` - Success or error from the transfer
    pub fn transfer_nft(&mut self) -> Result<()> {
        // Create CPI context for token transfer
        let cpi_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.seller_token_account.to_account_info(),
                mint: self.nft.to_account_info(),
                to: self.listing_token_account.to_account_info(),
                authority: self.seller.to_account_info(),
            },
        );

        // Transfer exactly 1 NFT (decimals from mint account)
        transfer_checked(cpi_ctx, 1, self.nft.decimals)
    }

    /// Initialize the listing state with seller and price information
    ///
    /// # Arguments
    /// * `price` - The listing price in lamports
    /// * `bumps` - PDA bump values for the listing account
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub fn initialize_listing(&mut self, price: u64, bumps: ListNftBumps) -> Result<()> {
        // Validate price is greater than 0
        require!(price > 0, MarketplaceError::InvalidPrice);

        // Initialize listing state
        self.listing.set_inner(Listing {
            seller: self.seller.key(),
            mint: self.nft.key(),
            price,
            bump: bumps.listing,
            is_active: true,
        });

        Ok(())
    }
}
