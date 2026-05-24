use crate::{events::SensorReadingEvent, states::SensorReading};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(authority: Pubkey)]
pub struct UpdateReading<'info> {
    #[account(
        mut,
        seeds = [b"sensor_reading", authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub sensor_reading: Account<'info, SensorReading>,
}

impl<'info> UpdateReading<'info> {
    pub fn update_reading(
        &mut self,
        _authority: Pubkey,
        pm25: u16,
        pm10: u16,
        temperature: u16,
        humidity: u16,
        aqi: u16
    ) -> Result<()> {
        let timestamp: u64 = Clock::get()?.unix_timestamp as u64;

        self.sensor_reading.pm25 = pm25;
        self.sensor_reading.pm10 = pm10;
        self.sensor_reading.temperature = temperature;
        self.sensor_reading.humidity = humidity;
        self.sensor_reading.aqi = aqi;
        self.sensor_reading.timestamp = timestamp;

        msg!(
            "pm25: {}, pm10: {}, temperature: {}, humidity: {}, timestamp: {}",
            pm25,
            pm10,
            temperature,
            humidity,
            timestamp
        );
        emit!(SensorReadingEvent {
            pm25,
            pm10,
            temperature,
            humidity,
            aqi,
            timestamp,
        });

        Ok(())
    }
}
