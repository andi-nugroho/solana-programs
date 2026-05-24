use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

use crate::states::Escrow;

#[derive(Accounts)]
pub struct Refund<'info> {
    /// The original escrow creator requesting refund of their tokens.
    /// Must be mutable to receive refunded tokens and rent.
    #[account(mut)]
    pub maker: Signer<'info>,

    /// Token mint for the escrowed asset (Token A).
    /// Must match the mint stored in escrow state.
    #[account(
        mint::token_program = token_program
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,

    /// Maker's token account to receive refunded Token A.
    /// Must be the same account that originally funded the escrow.
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    /// Escrow state account to be closed after refund.
    /// - Validates maker identity and token mint match stored values
    /// - Rent is returned to maker upon closing
    /// - Uses stored bump for PDA verification
    #[account(
        mut,
        close = maker,
        has_one = mint_a,
        has_one = maker,
        seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,

    /// Vault holding escrowed Token A to be refunded.
    /// Will be emptied and closed during refund process.
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program    
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Refund<'info> {
    /// Refunds all escrowed tokens to maker and closes the vault.
    /// 
    /// Returns the entire vault balance to the original maker,
    /// effectively canceling the escrow and cleaning up accounts.
    pub fn refund_and_close_vault(&mut self) -> Result<()> {
        // Create PDA signer seeds for vault authority
        let signer_seeds: &[&[&[u8]]; 1] = &[&[
            b"escrow", 
            self.maker.key.as_ref(), 
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump]
        ]];
        
        let decimals = self.mint_a.decimals;
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = TransferChecked{            
            mint: self.mint_a.to_account_info(),
            from: self.vault.to_account_info(),
            to: self.maker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        // Return all escrowed tokens to maker
        transfer_checked(cpi_context, self.vault.amount, decimals)?;

        // Close vault account and refund rent to maker
        let close_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let close_cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(), 
            close_accounts, 
            signer_seeds
        );

        close_account(close_cpi_ctx)?;

        Ok(())
    }
}