import BN from "bn.js";

export const BASE_SQRT_PRICE_X64 = new BN(1).shln(64); // 2^64,Q64.64; base tick repr or tick 0
export const TICK_PER_BASE = new BN(20000);
export const TICK_STEP_SIZE = BASE_SQRT_PRICE_X64.div(TICK_PER_BASE); // Distance between ticks in sqrt_price_x64 space

export const integer_sqrt = (value: number): number => {
  if (value == 0) {
    return 0;
  }
  let x = value;
  let y = (value + 1) / 2;

  while (y < x) {
    x = y;
    y = (y + value / y) / 2;
  }

  return x;
};

export const tickToSqrtPriceX64 = (tick: number): BN => {
  const tickAdjustment = new BN(Math.abs(tick)).mul(TICK_STEP_SIZE);

  if (tick >= 0) {
    return BASE_SQRT_PRICE_X64.add(tickAdjustment);
  } else {
    return BASE_SQRT_PRICE_X64.sub(tickAdjustment);
  }
};