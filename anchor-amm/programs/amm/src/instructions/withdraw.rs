use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{burn, transfer_checked, Burn, Mint, Token, TokenAccount, TransferChecked},
};
use constant_product_curve::ConstantProduct;

use crate::errors::AmmError;
use crate::states::Config;

/// Accounts required for withdrawing liquidity from the AMM pool
/// This struct defines all the accounts needed to perform a liquidity withdrawal operation
#[derive(Accounts)]
pub struct Withdraw<'info> {
    /// The user who is withdrawing liquidity (must sign the transaction)
    #[account(mut)]
    pub user: Signer<'info>,

    /// The mint account for token X in the trading pair
    /// Immutable as we only need to read mint information for transfers
    #[account(mint::token_program = token_program)]
    pub mint_x: Account<'info, Mint>,

    /// The mint account for token Y in the trading pair
    /// Immutable as we only need to read mint information for transfers
    #[account(mint::token_program = token_program)]
    pub mint_y: Account<'info, Mint>,

    /// The AMM pool configuration account
    /// Contains pool settings and references to the token mints
    /// Uses PDA derived from "config" seed and config.seed
    #[account(
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
        has_one = mint_x,  // Ensures mint_x matches the one in config
        has_one = mint_y,  // Ensures mint_y matches the one in config
    )]
    pub config: Account<'info, Config>,

    /// The LP (Liquidity Provider) token mint
    /// Mutable because we need to burn LP tokens from the user
    /// Uses PDA derived from "lp" seed and config pubkey
    #[account(
        mut,
        seeds = [b"lp", config.key().as_ref()],
        bump = config.lp_bump
    )]
    pub mint_lp: Account<'info, Mint>,

    /// The vault that holds all deposited token X
    /// Mutable because we're withdrawing tokens from it
    /// Associated token account owned by the config PDA
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
        associated_token::token_program = token_program,
    )]
    pub vault_x: Account<'info, TokenAccount>,

    /// The vault that holds all deposited token Y
    /// Mutable because we're withdrawing tokens from it
    /// Associated token account owned by the config PDA
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
        associated_token::token_program = token_program,
    )]
    pub vault_y: Account<'info, TokenAccount>,

    /// User's token account for token X
    /// Mutable because we're transferring tokens to it
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_ata_x: Account<'info, TokenAccount>,

    /// User's token account for token Y
    /// Mutable because we're transferring tokens to it
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_ata_y: Account<'info, TokenAccount>,

    /// User's token account for LP tokens
    /// Will be created if it doesn't exist, user pays for creation
    /// Mutable because we're burning LP tokens from it
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_lp,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_ata_lp: Account<'info, TokenAccount>,

    /// SPL Token program for token operations
    pub token_program: Program<'info, Token>,
    /// Associated Token program for ATA operations
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// System program for account creation
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    /// Main withdrawal function that handles liquidity removal
    /// 
    /// The process follows this sequence:
    /// 1. Read current LP supply and vault balances
    /// 2. Calculate proportional token amounts based on LP tokens being burned
    /// 3. Validate slippage protection (amounts meet minimum requirements)
    /// 4. Burn the LP tokens from user's account
    /// 5. Transfer proportional amounts of both tokens to user
    /// 
    /// # Arguments
    /// * `amount` - Amount of LP tokens to burn
    /// * `min_x` - Minimum amount of token X user expects to receive
    /// * `min_y` - Minimum amount of token Y user expects to receive
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if successful, error otherwise
    pub fn withdraw(&mut self, amount: u64, min_x: u64, min_y: u64) -> Result<()> {
        // Ensure the pool is not locked for withdrawals
        require!(self.config.locked == false, AmmError::PoolLocked);
        // Ensure user is requesting to burn some LP tokens
        require!(amount != 0, AmmError::InvalidAmount);

        // Calculate token amounts to withdraw based on current pool state
        let (x, y) = match self.mint_lp.supply == 0
            && self.vault_x.amount == 0
            && self.vault_y.amount == 0
        {
            // Edge case: if pool is completely empty, use minimum amounts
            // This shouldn't happen in normal operation but provides safety
            true => (min_x, min_y),
            // Normal case: calculate proportional amounts based on LP token share
            false => {
                let amounts = ConstantProduct::xy_withdraw_amounts_from_l(
                    self.vault_x.amount,   // Current vault X balance
                    self.vault_y.amount,   // Current vault Y balance
                    self.mint_lp.supply,   // Current LP token supply
                    amount,                // LP tokens being burned
                    6,                     // Precision for calculations
                )
                .unwrap();
                (amounts.x, amounts.y)
            }
        };

        // Slippage protection: ensure calculated amounts meet user's minimum requirements
        require!(x >= min_x && y >= min_y, AmmError::SlippageExceeded);

        // Burn LP tokens from user's account first
        self.burn_lp_tokens(amount)?;

        // Transfer calculated amounts of both tokens to user
        self.withdraw_tokens(x, true)?;   // Transfer token X
        self.withdraw_tokens(y, false)    // Transfer token Y
    }

    /// Burns LP tokens from the user's account
    /// This reduces the total LP supply and removes the user's claim on pool liquidity
    /// 
    /// # Arguments
    /// * `amount` - Amount of LP tokens to burn
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if successful, error otherwise
    pub fn burn_lp_tokens(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        // Set up burn instruction accounts
        let cpi_accounts = Burn {
            mint: self.mint_lp.to_account_info(),
            from: self.user_ata_lp.to_account_info(),
            authority: self.user.to_account_info(),  // Config PDA has authority to burn
        };

        // Create signer seeds for config PDA
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"config",
            &self.config.seed.to_le_bytes(),
            &[self.config.config_bump],
        ]];

        // Create CPI context with PDA signer
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        // Execute the burn operation
        burn(cpi_ctx, amount)
    }

    /// Transfers tokens from vault to user's account
    /// 
    /// # Arguments
    /// * `amount` - Amount of tokens to transfer
    /// * `is_x` - true for token X, false for token Y
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if successful, error otherwise
    pub fn withdraw_tokens(&mut self, amount: u64, is_x: bool) -> Result<()> {
        // Select appropriate accounts based on token type
        let (from, to, mint, decimals) = match is_x {
            true => (
                self.vault_x.to_account_info(),      // Transfer from vault X
                self.user_ata_x.to_account_info(),   // Transfer to user's X account
                self.mint_x.to_account_info(),       // Token X mint
                self.mint_x.decimals,                // Token X decimals
            ),
            false => (
                self.vault_y.to_account_info(),      // Transfer from vault Y
                self.user_ata_y.to_account_info(),   // Transfer to user's Y account
                self.mint_y.to_account_info(),       // Token Y mint
                self.mint_y.decimals,                // Token Y decimals
            ),
        };

        let cpi_program = self.token_program.to_account_info();

        // Set up transfer instruction accounts
        let cpi_accounts = TransferChecked {
            from,
            to,
            mint,
            authority: self.config.to_account_info(),  // Config PDA signs the transfer
        };

        // Create signer seeds for config PDA
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"config",
            &self.config.seed.to_le_bytes(),
            &[self.config.config_bump],
        ]];

        // Create CPI context with PDA signer
        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        // Execute the transfer with amount and decimal validation
        transfer_checked(cpi_context, amount, decimals)
    }
}