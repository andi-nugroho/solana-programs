use crate::states::*;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
pub struct Stake<'info> {
    /// User who owns the NFT and wants to stake it
    #[account(mut)]
    pub user: Signer<'info>,

    /// User's staking account to update stake count
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump
    )]
    pub user_account: Account<'info, UserAccount>,

    /// Global config containing staking parameters
    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, StakeConfig>,

    /// NFT mint to be staked
    pub nft_mint: Account<'info, Mint>,

    /// User's NFT token account (source of NFT transfer)
    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = user,
    )]
    pub user_nft_ata: Account<'info, TokenAccount>,

    /// Vault token account where NFT will be stored (owned by config)
    #[account(
        init_if_needed,                              // Create if doesn't exist
        payer = user,                               // User pays rent
        seeds = [b"vault", nft_mint.key().as_ref()], // Unique vault per NFT
        bump,                                       // Store bump
        token::mint = nft_mint,                     // Must match NFT mint
        token::authority = config,                  // Config owns the vault
    )]
    pub vault_ata: Account<'info, TokenAccount>,

    /// Stake record PDA to track individual NFT stake info
    #[account(
        init,                                                         // Create new account
        payer = user,                                                // User pays rent
        seeds = [b"stake", user.key().as_ref(), nft_mint.key().as_ref()], // Unique per user+NFT
        bump,                                                        // Store bump
        space = 8 + StakeAccount::INIT_SPACE,                       // Account size
    )]
    pub stake_account: Account<'info, StakeAccount>,

    /// Required programs
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>, // Needed for timestamp
}

impl<'info> Stake<'info> {
    /// Execute the staking operation
    pub fn stake(&mut self, bumps: StakeBumps) -> Result<()> {
        let clock = Clock::get()?;

        // Record stake metadata for this specific NFT
        self.stake_account.set_inner(StakeAccount {
            owner: self.user.key(),         // Who staked this NFT
            mint: self.nft_mint.key(),      // Which NFT was staked
            stake_at: clock.unix_timestamp, // When it was staked
            bump: bumps.stake_account,      // PDA bump
        });

        self.user_account.amount_staked = self.user_account.amount_staked.saturating_add(1);

        // Transfer NFT from user to vault
        let cpi_accounts = Transfer {
            from: self.user_nft_ata.to_account_info(),
            to: self.vault_ata.to_account_info(),
            authority: self.user.to_account_info(),
        };

        // CPI Context
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);
        transfer(cpi_ctx, 1)?; // Transfer 1 NFT token

        Ok(())
    }
}