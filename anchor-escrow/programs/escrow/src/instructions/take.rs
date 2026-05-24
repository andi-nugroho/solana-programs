use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

use crate::states::Escrow;

/// Instruction for completing an escrow trade.
/// 
/// Process:
/// 1. Taker sends Token B to maker
/// 2. Escrowed Token A is transferred to taker
/// 3. Vault and escrow accounts are closed
#[derive(Accounts)]
pub struct Take<'info> {
    /// The trade counterparty who provides Token B and receives Token A.
    /// Must be mutable to receive tokens and pay for account creation.
    #[account(mut)]
    pub taker: Signer<'info>,

    /// Original escrow creator who receives Token B.
    /// Must be mutable to receive rent refunds from closed accounts.
    #[account(mut)]
    pub maker: SystemAccount<'info>,

    /// Token mint for the asset taker provides (Token B).
    /// Must match the mint stored in escrow state.
    #[account(
        mint::token_program = token_program
    )]
    pub mint_b: InterfaceAccount<'info, Mint>,

    /// Token mint for the asset taker receives (Token A).
    /// Must match the mint stored in escrow state.
    #[account(
        mint::token_program = token_program
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,

    /// Taker's token account holding Token B for payment.
    /// Must have sufficient balance for the required amount.
    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Maker's token account to receive Token B.
    /// Created automatically if it doesn't exist (taker pays rent).
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Taker's token account to receive Token A.
    /// Created automatically if it doesn't exist (taker pays rent).
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_ata_a: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Escrow state account containing trade parameters.
    /// - Validates token mints and maker identity match stored values
    /// - Rent is returned to maker upon closing
    /// - Uses stored bump for PDA verification
    #[account(
        mut,
        close = maker,
        has_one = mint_a,
        has_one = maker,
        has_one = mint_b,
        seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,

    /// Vault holding escrowed Token A.
    /// Will be emptied and closed during trade execution.
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

impl<'info> Take<'info> {
    /// Transfers Token B from taker to maker as payment.
    /// 
    /// Uses the expected receive amount stored in escrow state.
    /// This completes the taker's side of the trade agreement.
    pub fn deposit(&mut self) -> Result<()> {
        let decimals = self.mint_b.decimals;
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = TransferChecked {
            mint: self.mint_b.to_account_info(),
            from: self.taker_ata_b.to_account_info(),
            to: self.maker_ata_b.to_account_info(),
            authority: self.taker.to_account_info(),
        };

        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);

        // Transfer the exact amount expected by the maker
        transfer_checked(cpi_context, self.escrow.receive, decimals)?;

        Ok(())
    }

    /// Transfers escrowed Token A to taker and closes the vault.
    /// 
    /// Completes the trade by releasing all escrowed tokens to the taker
    /// and cleaning up the vault account (rent goes to maker).
    pub fn transfer_and_close_vault(&mut self) -> Result<()> {
        // Create PDA signer seeds for vault authority
        let signer_seeds: &[&[&[u8]]; 1] = &[&[
            b"escrow",
            self.maker.key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];

        let decimals = self.mint_a.decimals;
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.taker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        // Transfer all escrowed tokens to taker
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
            signer_seeds,
        );

        close_account(close_cpi_ctx)?;

        Ok(())
    }
}