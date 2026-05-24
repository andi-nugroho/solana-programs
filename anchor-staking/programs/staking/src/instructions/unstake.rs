use crate::{errors::CustomError, states::*};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
pub struct Unstake<'info> {
    /// User unstaking their NFT
    #[account(mut)]
    pub user: Signer<'info>,

    /// User account tracking staked amount and points
    #[account(
        mut,
        seeds = [b"user", user.key.as_ref()],
        bump = user_account.bump
    )]
    pub user_account: Account<'info, UserAccount>,

    /// Global staking config
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, StakeConfig>,

    /// NFT mint being unstaked
    pub nft_mint: Account<'info, Mint>,

    /// Stake record for this NFT, to be closed after unstaking
    #[account(
        mut,
        seeds = [b"stake", user.key.as_ref(), nft_mint.key().as_ref()],
        bump = stake_account.bump,
        close = user  // Return rent to user
    )]
    pub stake_account: Account<'info, StakeAccount>,

    /// Vault holding the staked NFT
    #[account(
        mut,
        seeds = [b"vault", nft_mint.key().as_ref()],
        bump,
    )]
    pub vault_ata: Account<'info, TokenAccount>,

    /// User's token account to receive NFT
    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = user,
    )]
    pub user_nft_ata: Account<'info, TokenAccount>,

    /// Programs
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

impl<'info> Unstake<'info> {
    pub fn unstake(&mut self) -> Result<()> {
        // Check that the freeze period has passed
        let now = Clock::get()?.unix_timestamp;
        require!(
            now - self.stake_account.stake_at >= self.config.freeze_period as i64,
            CustomError::NotFrozen
        );

        // Ensure user has at least one NFT staked
        require!(
            self.user_account.amount_staked > 0,
            CustomError::NothingToUnstake
        );

        // Decrease the user's staked NFT count
        self.user_account.amount_staked = self
            .user_account
            .amount_staked
            .checked_sub(1)
            .ok_or(CustomError::Underflow)?;

        // Increase user's reward points (this NFT's reward)
        self.user_account.points = self
            .user_account
            .points
            .checked_add(self.config.points_per_stake as u32)
            .ok_or(CustomError::Overflow)?;

        // Generate signer seeds for config PDA
        let seeds: &[&[u8]] = &[b"config", &[self.config.bump]];
        let signer: &[&[&[u8]]; 1] = &[seeds];

        // Transfer the NFT token from vault ATA back to user's wallet
        let cpi_accounts = Transfer {
            from: self.vault_ata.to_account_info(),
            to: self.user_nft_ata.to_account_info(),
            authority: self.config.to_account_info(),
        };

        let cpi_ctx =
            CpiContext::new_with_signer(self.token_program.to_account_info(), cpi_accounts, signer);

        // Only 1 NFT is transferred
        transfer(cpi_ctx, 1)?;

        Ok(())
    }
}