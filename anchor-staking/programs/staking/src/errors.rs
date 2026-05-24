use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Not frozen")]
    NotFrozen,

    #[msg("Nothing to unstake")]
    NothingToUnstake,

    #[msg("You have no rewards to claim.")]
    NoRewardsToClaim,

    #[msg("Underflow")]
    Underflow,

    #[msg("Overflow")]
    Overflow
}