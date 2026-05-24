use anchor_lang::prelude::*;

#[event]
pub struct SensorReadingEvent {
    pub pm25: u16,
    pub pm10: u16,
    pub temperature: u16,
    pub humidity: u16,
    pub aqi: u16,
    pub timestamp: u64,
}