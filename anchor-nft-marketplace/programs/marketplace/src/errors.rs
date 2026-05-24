use anchor_lang::prelude::*;

#[error_code]
pub enum MarketplaceError {
  #[msg("Invalid fee percentage")]
  InvalidFeePercentage,

  #[msg("Invalid price")]
  InvalidPrice,

  #[msg("Listing is not active")]
  ListingNotActive,

  #[msg("Math overflow")]
  MathOverflow
}