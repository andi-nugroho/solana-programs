use anchor_lang::prelude::*;
use crate::{
    states::{SensorReading}
};

#[derive(Accounts)]
pub struct InitializeReading<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
      init_if_needed,
      payer = user,
      space = 8 + SensorReading::INIT_SPACE,
      seeds = [b"sensor_reading", user.key().as_ref()],
      bump
    )]
    pub sensor_reading: Account<'info, SensorReading>,

    pub system_program: Program<'info, System>,
}

impl<'info> InitializeReading<'info> {
    pub fn init(
        &mut self,
        pm25: u16,
        pm10: u16,
        temperature: u16,
        humidity: u16,
        aqi: u16
    ) -> Result<()> {
        let timestamp: u64 = Clock::get()?.unix_timestamp as u64;
        self.sensor_reading.set_inner(SensorReading {
            authority: self.user.key(),
            pm25,
            pm10,
            temperature,
            humidity,
            timestamp,
            aqi
        });

        Ok(())
    }
}
