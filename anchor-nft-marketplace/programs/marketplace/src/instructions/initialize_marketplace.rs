use anchor_lang::prelude::*;

use crate::{errors::MarketplaceError, states::Marketplace};

#[derive(Accounts)]
pub struct InitializeMarketplace<'info> {
    /// The admin account that will manage the marketplace
    /// Must be mutable to pay for account creation
    #[account(mut)]
    pub admin: Signer<'info>,

    /// The marketplace state account
    /// - Initialized with a PDA using "marketplace" seed
    /// - Stores admin pubkey, fee percentage, and bump values
    #[account(
        init,
        payer = admin,
        space = 8 + Marketplace::INIT_SPACE,
        seeds = [b"marketplace"],
        bump
    )]
    pub marketplace: Account<'info, Marketplace>,

    /// Treasury account for collecting marketplace fees
    /// - Uses PDA with "treasury" seed and marketplace key
    /// - Not initialized here, just validated
    #[account(
        seeds = [b"treasury", marketplace.key().as_ref()],
        bump
    )]
    pub treasury: SystemAccount<'info>,

    /// Required system program for account creation
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeMarketplace<'info> {
    /// Initialize the marketplace with admin and fee configuration
    ///
    /// # Arguments
    /// * `fee_percentage` - The percentage fee (0-100) charged on each sale
    /// * `bumps` - PDA bump values for deterministic addresses
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub fn initialize_marketplace(
        &mut self,
        fee_percentage: u8,
        bumps: InitializeMarketplaceBumps,
    ) -> Result<()> {
        // Validate fee percentage is reasonable (0-100%)
        require!(
            fee_percentage <= 100,
            MarketplaceError::InvalidFeePercentage
        );

        // Initialize marketplace state with provided parameters
        self.marketplace.set_inner(Marketplace {
            admin: self.admin.key(),
            fee_percentage,
            bump: bumps.marketplace,
            treasury_bump: bumps.treasury, // Fixed: should be treasury bump, not marketplace bump
        });

        Ok(())
    }
}
