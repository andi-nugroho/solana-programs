use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct VaultState {
    pub vault_bump: u8, // bump for the vault pda
    pub state_bump: u8, // bump for this pda itself
}