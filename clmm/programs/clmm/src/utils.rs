use crate::error::CLMMError;
use anchor_lang::prelude::*;

const Q64: u128 = 1 << 64;
pub const TICK_SPACING: i32 = 10;
const BASE_SQRT_PRICE_X64: u128 = Q64;

const SQRT_1_0001_X64: u128 = 18446758646477570048; // sqrt(1.0001) * 2^64
pub const MIN_TICK: i32 = -443636;
pub const MAX_TICK: i32 = -MIN_TICK;
/// The minimum value that can be returned from #get_sqrt_price_at_tick. Equivalent to get_sqrt_price_at_tick(MIN_TICK)
pub const MIN_SQRT_PRICE_X64: u128 = 4295048016;
/// The maximum value that can be returned from #get_sqrt_price_at_tick. Equivalent to get_sqrt_price_at_tick(MAX_TICK)
pub const MAX_SQRT_PRICE_X64: u128 = 79226673521066979257578248091;
const BIT_PRECISION: u32 = 16;

pub fn integer_sqrt(value: u128) -> u64 {
    if value == 0 {
        return 0;
    }

    let mut x = value;
    let mut y = (value + 1) / 2;

    while y < x {
        x = y;
        y = (x + value / x) / 2;
    }

    x as u64
}

/// Convert a u64 price to sqrt_price_x64 format
/// Formula: sqrt(price) * 2^64
pub fn price_to_sqrt_price_x64(price: u64) -> Result<u128> {
    if price == 0 {
        return Err(CLMMError::ZeroAmount.into());
    }

    let price_scaled = price as u128;

    let mut x = price_scaled;
    let mut y = (price_scaled + 1) / 2;

    while y < x {
        x = y;
        y = (x + price_scaled / x) / 2;
    }

    let sqrt_price_x64 = x.checked_mul(Q64).ok_or(CLMMError::ArithmeticOverflow)?;

    Ok(sqrt_price_x64)
}

// formula = sqrt(log(1.0001^tick)) * 2^64
pub fn tick_to_sqrt_price_x64(tick: i32) -> Result<u128> {
    let abs_tick = tick.abs() as u32;
    require!(abs_tick <= MAX_TICK as u32, CLMMError::TickUpperOverflow);

    // i = 0
    let mut ratio = if abs_tick & 0x1 != 0 {
        0xfffcb933bd6fb800u128
    } else {
        // 2^64
        1u128 << 64
    };

    // i = 1
    if abs_tick & 0x2 != 0 {
        ratio = (ratio.wrapping_mul(0xfff97272373d4000u128)) >> 64;
    }
    // i = 2
    if abs_tick & 0x4 != 0 {
        ratio = (ratio.wrapping_mul(0xfff2e50f5f657000u128)) >> 64;
    }
    // i = 3
    if abs_tick & 0x8 != 0 {
        ratio = (ratio.wrapping_mul(0xffe5caca7e10f000u128)) >> 64;
    }
    // i = 4
    if abs_tick & 0x10 != 0 {
        ratio = (ratio.wrapping_mul(0xffcb9843d60f7000u128)) >> 64;
    }
    // i = 5
    if abs_tick & 0x20 != 0 {
        ratio = (ratio.wrapping_mul(0xff973b41fa98e800u128)) >> 64;
    }
    // i = 6
    if abs_tick & 0x40 != 0 {
        ratio = (ratio.wrapping_mul(0xff2ea16466c9b000u128)) >> 64;
    }
    // i = 7
    if abs_tick & 0x80 != 0 {
        ratio = (ratio.wrapping_mul(0xfe5dee046a9a3800u128)) >> 64;
    }
    // i = 8
    if abs_tick & 0x100 != 0 {
        ratio = (ratio.wrapping_mul(0xfcbe86c7900bb000u128)) >> 64;
    }
    // i = 9
    if abs_tick & 0x200 != 0 {
        ratio = (ratio.wrapping_mul(0xf987a7253ac65800u128)) >> 64;
    }
    // i = 10
    if abs_tick & 0x400 != 0 {
        ratio = (ratio.wrapping_mul(0xf3392b0822bb6000u128)) >> 64;
    }
    // i = 11
    if abs_tick & 0x800 != 0 {
        ratio = (ratio.wrapping_mul(0xe7159475a2caf000u128)) >> 64;
    }
    // i = 12
    if abs_tick & 0x1000 != 0 {
        ratio = (ratio.wrapping_mul(0xd097f3bdfd2f2000u128)) >> 64;
    }
    // i = 13
    if abs_tick & 0x2000 != 0 {
        ratio = (ratio.wrapping_mul(0xa9f746462d9f8000u128)) >> 64;
    }
    // i = 14
    if abs_tick & 0x4000 != 0 {
        ratio = (ratio.wrapping_mul(0x70d869a156f31c00u128)) >> 64;
    }
    // i = 15
    if abs_tick & 0x8000 != 0 {
        ratio = (ratio.wrapping_mul(0x31be135f97ed3200u128)) >> 64;
    }
    // i = 16
    if abs_tick & 0x10000 != 0 {
        ratio = (ratio.wrapping_mul(0x9aa508b5b85a500u128)) >> 64;
    }
    // i = 17
    if abs_tick & 0x20000 != 0 {
        ratio = (ratio.wrapping_mul(0x5d6af8dedc582cu128)) >> 64;
    }
    // i = 18
    if abs_tick & 0x40000 != 0 {
        ratio = (ratio.wrapping_mul(0x2216e584f5fau128)) >> 64;
    }

    // For negative ticks, invert the ratio
    if tick > 0 {
        ratio = u128::MAX / ratio;
    }

    Ok(ratio as u128)
}

// tick = log base(sqrt(1.0001) ( sqrt_price_x64 / Q64) )
//to efficiently compute the above we find the log2 of above and divide by log(sqrt(1.0001))
pub fn sqrt_price_x64_to_tick(sqrt_price_x64: u128) -> Result<i32> {
    require!(
        sqrt_price_x64 >= MIN_SQRT_PRICE_X64 && sqrt_price_x64 < MAX_SQRT_PRICE_X64,
        CLMMError::SqrtPriceX64
    );

    // Determine log_b(sqrt_ratio). First by calculating integer portion (msb)
    let msb: u32 = 128 - sqrt_price_x64.leading_zeros() - 1;
    let log2p_integer_x32 = (msb as i128 - 64) << 32;

    // get fractional value (r/2^msb), msb always > 128
    // We begin the iteration from bit 63 (0.5 in Q64.64)
    let mut bit: i128 = 0x8000_0000_0000_0000i128;
    let mut precision = 0;
    let mut log2p_fraction_x64 = 0;

    // Log2 iterative approximation for the fractional part
    // Go through each 2^(j) bit where j < 64 in a Q64.64 number
    // Append current bit value to fraction result if r^2 Q2.126 is more than 2
    let mut r = if msb >= 64 {
        sqrt_price_x64 >> (msb - 63)
    } else {
        sqrt_price_x64 << (63 - msb)
    };

    while bit > 0 && precision < BIT_PRECISION {
        r *= r;
        let is_r_more_than_two = r >> 127 as u32;
        r >>= 63 + is_r_more_than_two;
        log2p_fraction_x64 += bit * is_r_more_than_two as i128;
        bit >>= 1;
        precision += 1;
    }
    let log2p_fraction_x32 = log2p_fraction_x64 >> 32;
    let log2p_x32 = log2p_integer_x32 + log2p_fraction_x32;

    // 14 bit refinement gives an error margin of 2^-14 / log2 (√1.0001) = 0.8461 < 1
    // Since tick is a decimal, an error under 1 is acceptable

    // Change of base rule: multiply with 2^16 / log2 (√1.0001)
    let log_sqrt_10001_x64 = log2p_x32 * 59543866431248i128;

    // tick - 0.01
    let tick_low = ((log_sqrt_10001_x64 - 184467440737095516i128) >> 64) as i32;

    // tick + (2^-14 / log2(√1.001)) + 0.01
    let tick_high = ((log_sqrt_10001_x64 + 15793534762490258745i128) >> 64) as i32;

    Ok(if tick_low == tick_high {
        tick_low
    } else if tick_to_sqrt_price_x64(tick_high)? <= sqrt_price_x64 {
        tick_high
    } else {
        tick_low
    })
}

pub fn calculate_liquidity_amounts(
    sqrt_price_current_x64: u128,
    sqrt_price_lower_x64: u128,
    sqrt_price_upper_x64: u128,
    liquidity: u128,
) -> Result<(u64, u64)> {
    let amount_a: u64;
    let amount_b: u64;

    if sqrt_price_current_x64 <= sqrt_price_lower_x64 {
        // Token A only: amount_a = L * (upper - lower) * Q64 / (upper * lower)
        let numerator = liquidity
            .checked_mul(
                sqrt_price_upper_x64
                    .checked_sub(sqrt_price_lower_x64)
                    .ok_or(CLMMError::ArithmeticOverflow)?,
            )
            .ok_or(CLMMError::ArithmeticOverflow)?
            .checked_mul(Q64)
            .ok_or(CLMMError::ArithmeticOverflow)?;

        let denominator = sqrt_price_upper_x64
            .checked_mul(sqrt_price_lower_x64)
            .ok_or(CLMMError::ArithmeticOverflow)?;

        amount_a = (numerator / denominator)
            .try_into()
            .map_err(|_| CLMMError::ArithmeticOverflow)?;
        amount_b = 0;
    } else if sqrt_price_current_x64 >= sqrt_price_upper_x64 {
        // Token B only: amount_b = L * (upper - lower) / Q64
        amount_a = 0;
        amount_b = (liquidity
            .checked_mul(
                sqrt_price_upper_x64
                    .checked_sub(sqrt_price_lower_x64)
                    .ok_or(CLMMError::ArithmeticOverflow)?,
            )
            .ok_or(CLMMError::ArithmeticOverflow)?
            .checked_div(Q64)
            .ok_or(CLMMError::ArithmeticOverflow)?)
        .try_into()
        .map_err(|_| CLMMError::ArithmeticOverflow)?;
    } else {
        // Both tokens:
        // amount_a = L * (upper - current) * Q64 / (upper * current)
        let numerator_a = liquidity
            .checked_mul(
                sqrt_price_upper_x64
                    .checked_sub(sqrt_price_current_x64)
                    .ok_or(CLMMError::ArithmeticOverflow)?,
            )
            .ok_or(CLMMError::ArithmeticOverflow)?
            .checked_mul(Q64)
            .ok_or(CLMMError::ArithmeticOverflow)?;

        let denominator_a = sqrt_price_upper_x64
            .checked_mul(sqrt_price_current_x64)
            .ok_or(CLMMError::ArithmeticOverflow)?;

        amount_a = (numerator_a / denominator_a)
            .try_into()
            .map_err(|_| CLMMError::ArithmeticOverflow)?;

        // amount_b = L * (current - lower) / Q64
        amount_b = (liquidity
            .checked_mul(
                sqrt_price_current_x64
                    .checked_sub(sqrt_price_lower_x64)
                    .ok_or(CLMMError::ArithmeticOverflow)?,
            )
            .ok_or(CLMMError::ArithmeticOverflow)?
            .checked_div(Q64)
            .ok_or(CLMMError::ArithmeticOverflow)?)
        .try_into()
        .map_err(|_| CLMMError::ArithmeticOverflow)?;
    }

    Ok((amount_a, amount_b))
}

pub fn compute_swap_step(
    sqrt_price_current_x64: u128,
    sqrt_price_target_x64: u128,
    liquidity: u128,
    amount_remaining: u128,
    a_to_b: bool,
) -> Result<(u128, u128, u128)> {
    let next_price: u128;
    let amount_in: u128;
    let amount_out: u128;

    if a_to_b {
        // Calculate required input for full step
        let price_diff = sqrt_price_current_x64
            .checked_sub(sqrt_price_target_x64)
            .ok_or(CLMMError::ArithmeticOverflow)?;

        let required_in = liquidity
            .checked_mul(price_diff)
            .ok_or(CLMMError::ArithmeticOverflow)?
            .checked_mul(Q64)
            .ok_or(CLMMError::ArithmeticOverflow)?
            .checked_div(
                sqrt_price_current_x64
                    .checked_mul(sqrt_price_target_x64)
                    .ok_or(CLMMError::ArithmeticOverflow)?,
            )
            .ok_or(CLMMError::ArithmeticOverflow)?;

        if amount_remaining >= required_in {
            // Full step
            next_price = sqrt_price_target_x64;
            amount_in = required_in;
        } else {
            // Partial step
            let numerator = liquidity
                .checked_mul(sqrt_price_current_x64)
                .ok_or(CLMMError::ArithmeticOverflow)?
                .checked_mul(sqrt_price_current_x64)
                .ok_or(CLMMError::ArithmeticOverflow)?;

            let denominator = liquidity
                .checked_mul(sqrt_price_current_x64)
                .ok_or(CLMMError::ArithmeticOverflow)?
                .checked_add(
                    amount_remaining
                        .checked_mul(sqrt_price_current_x64)
                        .ok_or(CLMMError::ArithmeticOverflow)?
                        .checked_div(Q64)
                        .ok_or(CLMMError::ArithmeticOverflow)?,
                )
                .ok_or(CLMMError::ArithmeticOverflow)?;

            next_price = numerator
                .checked_div(denominator)
                .ok_or(CLMMError::ArithmeticOverflow)?;
            amount_in = amount_remaining;
        }

        // Calculate output
        amount_out = liquidity
            .checked_mul(
                sqrt_price_current_x64
                    .checked_sub(next_price)
                    .ok_or(CLMMError::ArithmeticOverflow)?,
            )
            .ok_or(CLMMError::ArithmeticOverflow)?
            .checked_div(Q64)
            .ok_or(CLMMError::ArithmeticOverflow)?;
    } else {
        // B to A swap
        let price_diff = sqrt_price_target_x64
            .checked_sub(sqrt_price_current_x64)
            .ok_or(CLMMError::ArithmeticOverflow)?;

        let required_in = liquidity
            .checked_mul(price_diff)
            .ok_or(CLMMError::ArithmeticOverflow)?
            .checked_div(Q64)
            .ok_or(CLMMError::ArithmeticOverflow)?;

        if amount_remaining >= required_in {
            // Full step
            next_price = sqrt_price_target_x64;
            amount_in = required_in;
        } else {
            // Partial step
            next_price = sqrt_price_current_x64
                .checked_add(
                    amount_remaining
                        .checked_mul(Q64)
                        .ok_or(CLMMError::ArithmeticOverflow)?
                        .checked_div(liquidity)
                        .ok_or(CLMMError::ArithmeticOverflow)?,
                )
                .ok_or(CLMMError::ArithmeticOverflow)?;
            amount_in = amount_remaining;
        }

        // Calculate output
        let price_diff_out = next_price
            .checked_sub(sqrt_price_current_x64)
            .ok_or(CLMMError::ArithmeticOverflow)?;
        amount_out = liquidity
            .checked_mul(price_diff_out)
            .ok_or(CLMMError::ArithmeticOverflow)?
            .checked_mul(Q64)
            .ok_or(CLMMError::ArithmeticOverflow)?
            .checked_div(
                sqrt_price_current_x64
                    .checked_mul(next_price)
                    .ok_or(CLMMError::ArithmeticOverflow)?,
            )
            .ok_or(CLMMError::ArithmeticOverflow)?;
    }

    Ok((next_price, amount_in, amount_out))
}