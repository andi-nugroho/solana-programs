use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked},
};
use constant_product_curve::{ConstantProduct, LiquidityPair};

use crate::errors::AmmError;
use crate::states::Config;

/// Accounts required for performing token swaps in the AMM pool
/// This struct defines all the accounts needed to execute a swap operation
/// between two tokens in a constant product AMM pool
#[derive(Accounts)]
pub struct Swap<'info> {
    /// The user who is performing the swap (must sign the transaction)
    /// Mutable because they may need to pay for ATA creation if accounts don't exist
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
    /// Contains pool settings, fees, and references to the token mints
    /// Uses PDA derived from "config" seed and config.seed
    #[account(
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
        has_one = mint_x,  // Ensures mint_x matches the one in config
        has_one = mint_y,  // Ensures mint_y matches the one in config
    )]
    pub config: Account<'info, Config>,

    /// The LP (Liquidity Provider) token mint
    /// Used for reading supply information in swap calculations
    /// Uses PDA derived from "lp" seed and config pubkey
    #[account(
        seeds = [b"lp", config.key().as_ref()],
        bump = config.lp_bump
    )]
    pub mint_lp: Account<'info, Mint>,

    /// The vault that holds all deposited token X
    /// Mutable because swap operations either deposit to or withdraw from this vault
    /// Associated token account owned by the config PDA
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
        associated_token::token_program = token_program,
    )]
    pub vault_x: Account<'info, TokenAccount>,

    /// The vault that holds all deposited token Y
    /// Mutable because swap operations either deposit to or withdraw from this vault
    /// Associated token account owned by the config PDA
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
        associated_token::token_program = token_program,
    )]
    pub vault_y: Account<'info, TokenAccount>,

    /// User's token account for token X
    /// Will be created if it doesn't exist, user pays for creation
    /// Mutable because we may transfer tokens to/from this account
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_x,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_ata_x: Account<'info, TokenAccount>,

    /// User's token account for token Y
    /// Will be created if it doesn't exist, user pays for creation
    /// Mutable because we may transfer tokens to/from this account
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_y,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_ata_y: Account<'info, TokenAccount>,

    /// SPL Token program for token operations
    pub token_program: Program<'info, Token>,
    /// Associated Token program for ATA operations
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// System program for account creation
    pub system_program: Program<'info, System>,
}

impl<'info> Swap<'info> {
    /// Main swap function that handles token exchanges using constant product formula
    /// 
    /// The swap process follows these steps:
    /// 1. Validate pool is not locked and amounts are valid
    /// 2. Initialize constant product curve with current pool state
    /// 3. Calculate swap amounts using the curve (accounting for fees)
    /// 4. Validate slippage protection (output meets minimum requirement)
    /// 5. Deposit input tokens to appropriate vault
    /// 6. Withdraw output tokens from appropriate vault to user
    /// 
    /// The constant product formula (x * y = k) ensures that the product of
    /// token reserves remains constant after accounting for fees.
    /// 
    /// # Arguments
    /// * `is_x` - true if swapping token X for Y, false if swapping Y for X
    /// * `amount_in` - Amount of input tokens to swap
    /// * `min_amount_out` - Minimum amount of output tokens expected (slippage protection)
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if successful, error otherwise
    pub fn swap(&mut self, is_x: bool, amount_in: u64, min_amount_out: u64) -> Result<()> {
        // Ensure the pool is not locked for swaps
        require!(!self.config.locked, AmmError::PoolLocked);
        // Ensure user is swapping a positive amount
        require!(amount_in > 0, AmmError::InvalidAmount);

        // Initialize constant product curve with current pool state
        let mut curve = ConstantProduct::init(
            self.vault_x.amount,    // Current token X reserves
            self.vault_y.amount,    // Current token Y reserves
            self.mint_lp.supply,    // Current LP token supply
            self.config.fee,        // Trading fee in basis points
            None,                   // No additional configuration
        )
        .map_err(AmmError::from)?;

        // Determine which token is being swapped in
        let p = match is_x {
            true => LiquidityPair::X,   // Swapping X for Y
            false => LiquidityPair::Y,  // Swapping Y for X
        };

        // Calculate swap amounts using constant product formula
        // This accounts for fees and maintains the invariant
        let swap_result = curve
            .swap(p, amount_in, min_amount_out)
            .map_err(AmmError::from)?;

        // Validate that the calculated amounts are valid
        require!(swap_result.deposit != 0, AmmError::InvalidAmount);
        require!(swap_result.withdraw != 0, AmmError::InvalidAmount);

        // Execute the swap by depositing input tokens and withdrawing output tokens
        self.deposit_token(is_x, swap_result.deposit)?;      // Deposit input tokens
        self.withdraw_token(!is_x, swap_result.withdraw)?;   // Withdraw output tokens

        Ok(())
    }

    /// Deposits tokens from user's account to the appropriate vault
    /// This increases the vault's balance and decreases the user's balance
    /// 
    /// # Arguments
    /// * `is_x` - true for token X, false for token Y
    /// * `amount` - Amount of tokens to deposit
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if successful, error otherwise
    pub fn deposit_token(&mut self, is_x: bool, amount: u64) -> Result<()> {
        // Select appropriate accounts based on token type
        let (from, to, mint, decimals) = match is_x {
            true => (
                self.user_ata_x.to_account_info(),    // Transfer from user's X account
                self.vault_x.to_account_info(),       // Transfer to vault X
                self.mint_x.to_account_info(),        // Token X mint
                self.mint_x.decimals,                 // Token X decimals
            ),
            false => (
                self.user_ata_y.to_account_info(),    // Transfer from user's Y account
                self.vault_y.to_account_info(),       // Transfer to vault Y
                self.mint_y.to_account_info(),        // Token Y mint
                self.mint_y.decimals,                 // Token Y decimals
            ),
        };

        let cpi_program = self.token_program.to_account_info();

        // Set up transfer instruction accounts
        let cpi_accounts = TransferChecked {
            from,
            to,
            authority: self.user.to_account_info(),  // User signs the transfer
            mint,
        };

        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);

        // Execute the transfer with amount and decimal validation
        transfer_checked(cpi_context, amount, decimals)
    }

    /// Withdraws tokens from vault to user's account
    /// This decreases the vault's balance and increases the user's balance
    /// 
    /// # Arguments
    /// * `is_x` - true for token X, false for token Y
    /// * `amount` - Amount of tokens to withdraw
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if successful, error otherwise
    pub fn withdraw_token(&mut self, is_x: bool, amount: u64) -> Result<()> {
        // Select appropriate accounts based on token type
        let (from, to, mint, decimals) = match is_x {
            true => (
                self.vault_x.to_account_info(),       // Transfer from vault X
                self.user_ata_x.to_account_info(),    // Transfer to user's X account
                self.mint_x.to_account_info(),        // Token X mint
                self.mint_x.decimals,                 // Token X decimals
            ),
            false => (
                self.vault_y.to_account_info(),       // Transfer from vault Y
                self.user_ata_y.to_account_info(),    // Transfer to user's Y account
                self.mint_y.to_account_info(),        // Token Y mint
                self.mint_y.decimals,                 // Token Y decimals
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