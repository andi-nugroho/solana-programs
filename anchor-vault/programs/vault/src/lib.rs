use anchor_lang::prelude::*;

declare_id!("G5m8rLBcLLmbV1Jrt78p7FmDD2GhiNSDBQ1Tqzn8Lq5i");

pub mod instructions;
pub mod states;

pub use instructions::*;

#[program]
pub mod vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)
    }

    pub fn deposit(ctx: Context<Payment>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    pub fn withdraw(ctx: Context<Payment>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }

    pub fn close(ctx: Context<CloseAccounts>) -> Result<()> {
        ctx.accounts.close()
    }
}