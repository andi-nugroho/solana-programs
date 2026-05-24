use crate::states::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    /// Admin who can initialize the config (pays for account creation)
    #[account(mut)]
    pub admin: Signer<'info>,

    /// Global config PDA that stores staking parameters
    #[account(
        init,                                    // Create new account
        payer = admin,                          // Admin pays rent
        seeds = [b"config"],                    // Deterministic PDA seed
        bump,                                   // Store bump seed
        space = 8 + StakeConfig::INIT_SPACE,   // Account size allocation
    )]
    pub config: Account<'info, StakeConfig>,

    /// Reward token mint - created and owned by config PDA
    #[account(
        init,                                           // Create new mint
        payer = admin,                                 // Admin pays rent
        seeds = [b"rewards", config.key().as_ref()],   // Deterministic PDA seed
        bump,                                          // Store bump seed
        mint::decimals = 6,                           // Token decimals
        mint::authority = config,                     // Config owns mint authority
    )]
    pub reward_mint: Account<'info, Mint>,

    /// Required system programs
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> InitializeConfig<'info> {
    /// Initialize the global staking configuration
    pub fn initialize_config(
        &mut self,
        points_per_stake: u8,
        max_unstake: u8,
        freeze_period: u32,
        bumps: InitializeConfigBumps,
    ) -> Result<()> {
        // Store configuration parameters in the config account
        self.config.set_inner(StakeConfig {
            points_per_stake,                    // Points awarded per staked NFT
            max_unstake,                        // Max NFTs unstakeable at once
            freeze_period,                      // Required staking duration
            rewards_bump: bumps.reward_mint,    // Bump for reward mint PDA
            bump: bumps.config,                 // Bump for config PDA
        });
        Ok(())
    }
}
