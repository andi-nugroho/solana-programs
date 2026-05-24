use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::{anchor::commit, ephem::commit_and_undelegate_accounts};

use crate::states::SensorReading;

#[commit]
#[derive(Accounts)]
pub struct UndelegateReading<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, seeds = [b"sensor_reading", user.key().as_ref()], bump)]
    pub sensor_reading: Account<'info, SensorReading>,
}

impl<'info> UndelegateReading<'info> {
    pub fn undelegate(&mut self) -> Result<()> {
        commit_and_undelegate_accounts(
            &self.user,
            vec![&self.sensor_reading.to_account_info()],
            &self.magic_context,
            &self.magic_program,
        )?;
        Ok(())
    }
}