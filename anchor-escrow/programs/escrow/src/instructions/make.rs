use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, 
    token_interface::{ Mint, TokenAccount, TransferChecked, TokenInterface, transfer_checked}
};

use crate::states::Escrow;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'info> {
    /// The escrow creator who deposits tokens and pays for account creation.
    /// Must be mutable to deduct lamports for rent and transaction fees.
    #[account(mut)] 
    pub maker: Signer<'info>,

    /// Token mint for the asset being deposited into escrow (Token A).
    /// Validates compatibility with the specified token program.
    #[account(
        mint::token_program = token_program
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,

    /// Token mint for the asset expected in return (Token B).
    /// Used for validation and stored in escrow state for future verification.
    #[account(
        mint::token_program = token_program
    )]
    pub mint_b: InterfaceAccount<'info, Mint>,

    /// Maker's token account holding Token A to be escrowed.
    /// Must have sufficient balance for the deposit amount.
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    /// Escrow state account storing trade parameters and metadata.
    /// - Contains maker, token mints, expected amount, and bump seed
    /// - Derived from maker's pubkey and user-provided seed for uniqueness
    #[account(
        init,
        payer = maker,
        seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump,
        space = 8 + Escrow::INIT_SPACE,
    )]
    pub escrow: Account<'info, Escrow>,

    /// Vault token account that holds escrowed Token A.
    /// - Owned by the escrow PDA to prevent unauthorized access
    /// - Created as ATA for deterministic address derivation
    #[account(
        init, 
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program    
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    
    /// Token program interface for SPL token operations.
    /// Supports both Token Program and Token-2022 for flexibility.
    pub token_program: Interface<'info, TokenInterface>,
    
    /// Associated Token Program for creating and managing ATAs.
    pub associated_token_program: Program<'info, AssociatedToken>,
    
    /// System Program required for account creation and rent payments.
    pub system_program: Program<'info, System>,
}

impl<'info> Make<'info> {
    /// Initializes the escrow account with trade parameters.
    /// 
    /// Stores all necessary information for future trade execution including
    /// maker identity, token mints, expected receive amount, and PDA bump.
    pub fn init_escrow(&mut self, seed: u64, bump: &MakeBumps, receive: u64) -> Result<()>{
        self.escrow.set_inner(Escrow { 
            seed, 
            maker: self.maker.key(), 
            mint_a: self.mint_a.key(), 
            mint_b: self.mint_b.key(), 
            receive, 
            bump: bump.escrow 
        });

        Ok(())
    } 

    /// Deposits Token A from maker's account into the escrow vault.
    /// 
    /// Uses transfer_checked for enhanced security with decimal validation.
    /// Tokens remain locked until trade completion or refund.
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        let decimals = self.mint_a.decimals;
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = TransferChecked{            
            mint: self.mint_a.to_account_info(),
            from: self.maker_ata_a.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.maker.to_account_info(),
        };

        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);

        // Transfer tokens with decimal validation for security
        transfer_checked(cpi_context, amount, decimals)
    }
}