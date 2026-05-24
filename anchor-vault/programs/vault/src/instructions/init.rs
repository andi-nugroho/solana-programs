use anchor_lang::prelude::*;

use crate::states::VaultState;

#[derive(Accounts)]
pub struct Initialize<'info> {
   /// The signer account that pays for account creation and transaction fees.
   /// Must be mutable to deduct lamports for rent and fees.
   #[account(mut)]
   pub user: Signer<'info>,

   /// Program-derived account that stores vault configuration and bump seeds.
   /// - Derived from seeds: ["state", user.pubkey]
   /// - Owned by this program to store custom VaultState data
   /// - Automatically allocated with enough space for the VaultState struct
   #[account(
       init,
       payer = user,
       seeds = [b"state", user.key.as_ref()],
       bump,
       space = 8 + VaultState::INIT_SPACE
   )]
   pub vault_state: Account<'info, VaultState>,

   /// Program-derived account that holds SOL deposits for this user's vault.
   /// - Derived from seeds: ["vault", user.pubkey]
   /// - Owned by System Program (for holding lamports only)
   /// - No init constraint needed - becomes rent-exempt when SOL is transferred
   #[account(
       seeds = [b"vault", user.key.as_ref()],
       bump
   )]
   pub vault: SystemAccount<'info>,

   /// System Program reference required for account creation.
   /// Anchor invokes this program when using the 'init' constraint.
   pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
   /// Initializes the vault by storing PDA bump seeds on-chain.
   /// 
   /// Bump seeds are stored to ensure consistent PDA derivation in future
   /// instructions without needing to recompute or pass them as parameters.
   pub fn initialize(&mut self, bumps: &InitializeBumps) -> Result<()> {
       self.vault_state.vault_bump = bumps.vault;
       self.vault_state.state_bump = bumps.vault_state;

       Ok(())
   }
}