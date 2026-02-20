use anchor_lang::prelude::*;

#[error_code]
pub enum AMMError {
    #[msg("Token A and Token B must be different.")]
    SameTokenMint,
    #[msg("Wrong Token Mint")]
    InvalidTokenMint,
    #[msg("Token deposit amounts are not proportional to existing pool reserves.")]
    InvalidLiquidity,
    #[msg("Arithmetic operation overflow or division by zero.")]
    ArithmeticOverflow,
    #[msg("Liquidity pool is empty")]
    PoolEmpty,
    #[msg("You don't have sufficient liquidity provided tokens to redeem")]
    InsufficientLPTokens,
    #[msg("To add to liquidity pool, quantity must be greater than zero")]
    ZeroAmount,
    #[msg("Insufficient funds in pool")]
    InsufficientFundsInPool,
    #[msg("Slippage exceeded than the mininum quantity mentioned")]
    SlippageExceeded,
    #[msg("Invalid vault account")]
    InvalidVault,
}