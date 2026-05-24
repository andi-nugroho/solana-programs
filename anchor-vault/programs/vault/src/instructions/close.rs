use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};
use crate::states::VaultState;

#[derive(Accounts)]
pub struct CloseAccounts<'info> {
   /// The vault owner who can authorize closing and receives refunded rent + SOL.
   /// Must be mutable to receive the transferred lamports.
   #[account(mut)]
   pub user: Signer<'info>,

   /// Vault state account to be closed and have its rent refunded.
   /// - Uses stored bump for PDA verification
   /// - Rent lamports are returned to the user upon closing
   /// - Account data is zeroed and marked as closed
   #[account(
       mut,
       seeds = [b"state", user.key.as_ref()],
       bump = vault_state.state_bump,
       close = user
   )]
   pub vault_state: Account<'info, VaultState>,

   /// SOL vault account containing user's deposited funds.
   /// - Uses stored bump for PDA verification  
   /// - All lamports will be transferred back to user
   /// - Account remains but will be empty after withdrawal
   #[account(
       mut,
       seeds = [b"vault", user.key.as_ref()],
       bump = vault_state.vault_bump
   )]
   pub vault: SystemAccount<'info>,

   /// System Program required for SOL transfers between accounts.
   pub system_program: Program<'info, System>,
}

impl<'info> CloseAccounts<'info> {
   /// Closes the vault by transferring all SOL back to the user.
   /// 
   /// The vault_state account is automatically closed by Anchor (due to close constraint),
   /// but the vault's SOL must be manually transferred back to prevent loss of funds.
   pub fn close(&mut self) -> Result<()> {
       let cpi_program = self.system_program.to_account_info();
       let cpi_accounts = Transfer {
           from: self.vault.to_account_info(),
           to: self.user.to_account_info(),
       };

       // Create signer seeds for the vault PDA to authorize the transfer
       let seeds: &[&[u8]] = &[
           b"vault",
           self.user.key.as_ref(),
           &[self.vault_state.vault_bump],
       ];
       let signer_seeds: &[&[&[u8]]] = &[&seeds[..]];

       let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

       // Transfer all remaining SOL from vault back to user
       transfer(cpi_context, self.vault.lamports())
   }
}