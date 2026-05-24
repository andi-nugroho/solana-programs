use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, transfer_checked, Mint, MintTo, Token, TokenAccount, TransferChecked},
};
use constant_product_curve::ConstantProduct;

use crate::states::Config;
use crate::errors::AmmError;

/// Accounts required for depositing liquidity into the AMM pool
/// This struct defines all the accounts needed to perform a liquidity deposit operation
#[derive(Accounts)]
pub struct Deposit<'info> {
    /// The user who is depositing liquidity (must sign the transaction)
    #[account(mut)]
    pub user: Signer<'info>,

    /// The mint account for token X in the trading pair
    /// This is immutable as we only need to read mint information
    pub mint_x: Account<'info, Mint>,

    /// The mint account for token Y in the trading pair
    /// This is immutable as we only need to read mint information
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
    /// Mutable because we need to mint new LP tokens to the user
    /// Uses PDA derived from "lp" seed and config pubkey
    #[account(
        mut,
        seeds = [b"lp", config.key().as_ref()],
        bump = config.lp_bump
    )]
    pub mint_lp: Account<'info, Mint>,

    /// The vault that holds all deposited token X
    /// Mutable because we're depositing tokens into it
    /// Associated token account owned by the config PDA
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
        associated_token::token_program = token_program
    )]
    pub vault_x: Account<'info, TokenAccount>,

    /// The vault that holds all deposited token Y
    /// Mutable because we're depositing tokens into it
    /// Associated token account owned by the config PDA
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
        associated_token::token_program = token_program
    )]
    pub vault_y: Account<'info, TokenAccount>,
    
    /// User's token account for token X
    /// Mutable because we're transferring tokens from it
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_ata_x: Account<'info, TokenAccount>,

    /// User's token account for token Y
    /// Mutable because we're transferring tokens from it
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_ata_y: Account<'info, TokenAccount>,

    /// User's token account for LP tokens
    /// Will be created if it doesn't exist, user pays for creation
    /// Mutable because we're minting LP tokens to it
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

impl<'info> Deposit<'info> {
    /// Main deposit function that handles liquidity provision
    /// 
    /// For the first deposit (pool initialization):
    /// - Uses exact amounts provided by user (max_x, max_y)
    /// - Establishes the initial price ratio
    /// 
    /// For subsequent deposits:
    /// - Calculates proportional amounts based on current pool ratio
    /// - Maintains constant product invariant
    /// 
    /// # Arguments
    /// * `amount` - Amount of LP tokens to mint to the user
    /// * `max_x` - Maximum amount of token X user is willing to deposit
    /// * `max_y` - Maximum amount of token Y user is willing to deposit
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if successful, error otherwise
    pub fn deposit(
        &mut self, 
        amount: u64, 
        max_x: u64,
        max_y: u64,
    ) -> Result<()> { 
        // Ensure the pool is not locked for deposits
        require!(self.config.locked == false, AmmError::PoolLocked);
        // Ensure user is requesting to mint some LP tokens
        require!(amount != 0, AmmError::InvalidAmount);

        // Calculate required token amounts based on whether this is first deposit
        let (x, y) = match self.mint_lp.supply == 0 && self.vault_x.amount == 0 && self.vault_y.amount == 0 {
            // First deposit: use exact amounts provided by user
            // This establishes the initial price ratio for the pool
            true => (max_x, max_y), 
            // Subsequent deposits: calculate proportional amounts to maintain pool ratio
            false => {
                let amounts = ConstantProduct::xy_deposit_amounts_from_l(
                    self.vault_x.amount, 
                    self.vault_y.amount, 
                    self.mint_lp.supply, 
                    amount, 
                    6  // Precision for calculations
                ).unwrap();
                (amounts.x, amounts.y)
            }
        };

        // Slippage protection: ensure calculated amounts don't exceed user's maximum
        require!(x <= max_x && y <= max_y, AmmError::SlippageExceeded );

        // Transfer token X from user to vault
        self.deposit_tokens(true, x)?;
        // Transfer token Y from user to vault
        self.deposit_tokens(false, y)?;

        // Mint LP tokens to user as proof of liquidity provision
        self.mint_lp_tokens(amount)
    }

    /// Transfers tokens from user's account to the appropriate vault
    /// 
    /// # Arguments
    /// * `is_x` - true for token X, false for token Y
    /// * `amount` - Amount of tokens to transfer
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if successful, error otherwise
    pub fn deposit_tokens(&mut self, is_x: bool, amount: u64) -> Result<()> {
        // Select appropriate accounts based on token type
        let (
            from,      // User's token account
            to,        // Vault token account
            mint,      // Token mint
            decimals   // Token decimal places
        ) = match is_x {
            true => (
                self.user_ata_x.to_account_info(), 
                self.vault_x.to_account_info(),
                self.mint_x.to_account_info(),
                self.mint_x.decimals
            ),
            false => (
                self.user_ata_y.to_account_info(), 
                self.vault_y.to_account_info(),
                self.mint_y.to_account_info(),
                self.mint_y.decimals
            ),
        };

        let cpi_program = self.token_program.to_account_info();

        // Set up transfer instruction accounts
        let cpi_accounts = TransferChecked {
            from,
            to,
            authority: self.user.to_account_info(),  // User signs the transfer
            mint
        };

        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);

        // Execute the transfer with amount and decimal validation
        transfer_checked(cpi_context, amount, decimals)
    }

    /// Mints LP tokens to the user as receipt for their liquidity provision
    /// 
    /// # Arguments
    /// * `amount` - Amount of LP tokens to mint
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if successful, error otherwise
    pub fn mint_lp_tokens(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        // Set up mint instruction accounts
        let cpi_accounts = MintTo {
            mint: self.mint_lp.to_account_info(),
            to: self.user_ata_lp.to_account_info(),
            authority: self.config.to_account_info(),  // Config PDA is mint authority
        };

        // Create signer seeds for config PDA
        let seeds: &[&[u8]; 3] = &[ 
            &b"config"[..], 
            &self.config.seed.to_le_bytes(), 
            &[self.config.config_bump],
        ];
        
        let signer_seeds: &[&[&[u8]]] = &[&seeds[..]];

        // Create CPI context with PDA signer
        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        // Mint the LP tokens to user
        mint_to(cpi_context, amount)
    }
}