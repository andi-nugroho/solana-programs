#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

const DISCRIMINATOR: usize = 8;

declare_id!("4GxG4XwJa7P1t7c217Tu3yR1J51qxJ8ujmLUwQxwYCwa");

#[program]
pub mod anchor_counter {
    use super::*;

    // Initialize instruction handler
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        counter.count = 0;
        msg!("Counter Account Created");
        msg!("Current Count: { }", counter.count);

        Ok(())
    }

    // Add Increment Instruction
    pub fn increment(ctx: Context<Update>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        counter.count = counter.count.checked_add(1).unwrap();
        msg!("Updated Counter: {}", counter.count);
        Ok(())
    }

    // Add Decement Instruction
    pub fn decrement(ctx: Context<Update>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        counter.count = counter.count.saturating_sub(1);
        msg!("Updated Counter: {}", counter.count);
        Ok(())
    }
}

// Implement Context type Update
#[derive(Accounts)]
pub struct Update<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub counter: Account<'info, Counter>,
}

// Implement Context type Initialize
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = user,
        space = DISCRIMINATOR + Counter::INIT_SPACE,
    )]
    pub counter: Account<'info, Counter>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// Define Counter Account
#[account]
#[derive(InitSpace)]
pub struct Counter {
    pub count: u64,
}
