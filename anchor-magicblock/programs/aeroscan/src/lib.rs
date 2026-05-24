use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::ephemeral;

declare_id!("aero8wSmn3uAj5g5jYq92Rd2SQv2MtGxu1ZXfysfFHX");

pub mod events;
pub mod instructions;
pub mod states;

pub use instructions::*;

#[ephemeral]
#[program]
pub mod aeroscan {
    use super::*;

    pub fn initialize(
        ctx: Context<InitializeReading>,
        pm25: u16,
        pm10: u16,
        temperature: u16,
        humidity: u16,
        aqi: u16
    ) -> Result<()> {
        ctx.accounts
            .init(pm25, pm10, temperature, humidity, aqi)
    }

    pub fn update_reading(
        ctx: Context<UpdateReading>,
        authority: Pubkey,
        pm25: u16,
        pm10: u16,
        temperature: u16,
        humidity: u16,
        aqi: u16
    ) -> Result<()> {
        ctx.accounts
            .update_reading(authority, pm25, pm10, temperature, humidity, aqi)
    }

    pub fn delegate(ctx: Context<DelegateReading>) -> Result<()> {
        ctx.accounts.delegate()
    }

    pub fn undelegate(ctx: Context<UndelegateReading>) -> Result<()> {
        ctx.accounts.undelegate()
    }
}
