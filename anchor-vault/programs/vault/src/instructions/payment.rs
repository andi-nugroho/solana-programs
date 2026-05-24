use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};

use crate::states::VaultState;

#[derive(Accounts)]
pub struct Payment<'info> {
   /// The vault owner who can deposit SOL and authorize withdrawals.
   /// Must be mutable to send/receive lamports during transfers.
   #[account(mut)]
   pub user: Signer<'info>,

   /// Vault configuration account containing stored bump seeds.
   /// Read-only access needed to retrieve bump values for PDA derivation.
   #[account(
       seeds = [b"state", user.key.as_ref()],
       bump = vault_state.state_bump
   )]
   pub vault_state: Account<'info, VaultState>,

   /// SOL storage account that holds the user's deposited funds.
   /// - Becomes rent-exempt when first deposit makes it viable
   /// - Uses stored bump for consistent PDA verification
   #[account(
       mut,
       seeds = [b"vault", user.key.as_ref()],
       bump = vault_state.vault_bump
   )]
   pub vault: SystemAccount<'info>,

   /// System Program required for native SOL transfers.
   pub system_program: Program<'info, System>,
}

impl<'info> Payment<'info> {
   /// Deposits SOL from user's account into their vault.
   /// 
   /// Uses standard CPI since the user's signature authorizes the transfer.
   /// The vault PDA will be automatically created if this is the first deposit.
   pub fn deposit(&mut self, amount: u64) -> Result<()> {
       let cpi_program = self.system_program.to_account_info();
       let cpi_accounts = Transfer {
           from: self.user.to_account_info(),
           to: self.vault.to_account_info(),
       };

       // User's existing signature authorizes this outbound transfer
       let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
       transfer(cpi_context, amount)
   }

   /// Withdraws SOL from the vault back to the user's account.
   /// 
   /// Requires PDA signing since the vault (not user) is the source account.
   /// The program proves ownership of the PDA by providing the correct seeds.
   pub fn withdraw(&mut self, amount: u64) -> Result<()> {
       let cpi_program = self.system_program.to_account_info();
       let cpi_accounts = Transfer {
           from: self.vault.to_account_info(),
           to: self.user.to_account_info(),
       };

       // Construct PDA signer seeds to prove program ownership of the vault
       let seeds: &[&[u8]] = &[
           b"vault",
           self.user.key.as_ref(),
           &[self.vault_state.vault_bump],
       ];
       let signer_seeds: &[&[&[u8]]] = &[&seeds[..]];

       // PDA signing enables the program to authorize transfers from accounts it controls
       let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
       transfer(cpi_context, amount)
   }
}