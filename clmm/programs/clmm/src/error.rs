
use anchor_lang::prelude::*;

#[error_code]
pub enum CLMMError {
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
    #[msg("Max quantity should be greater than or equal to Min quantity")]
    QuantityMismatch,
    #[msg("Upper bound of ticks should be greater than lower bound")]
    TickMismatch,
    #[msg("Tick values must be aligned with tick spacing")]
    UnalignedTick,
    #[msg("Tick Index doesn't match")]
    InvalidTickIndex,
    #[msg("Missing Tick Accounts")]
    MissingTickAccounts,
    #[msg("Amount too large")]
    AmountTooLarge,
    #[msg("Unexpected error no swap happened")]
    ZeroSwapOutput,
    #[msg("The tick must be lesser than, or equal to the maximum tick(221818)")]
    TickUpperOverflow,
    #[msg("The tick must be greater, or equal to the minimum tick(-221818)")]
    TickLowerOverflow,
    #[msg("sqrt_price_x64 out of range")]
    SqrtPriceX64,
}
