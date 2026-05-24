# AMM Protocol

A decentralized Automated Market Maker (AMM) built on Solana using the Anchor framework. This protocol implements a constant product formula (x × y = k) to enable decentralized token swapping and liquidity provision.

## Overview

This AMM protocol allows users to:
- Create liquidity pools for token pairs
- Provide liquidity and earn trading fees
- Swap tokens using automated market making
- Remove liquidity proportionally

The protocol uses Program Derived Addresses (PDAs) for security and implements slippage protection for all operations.

## Program ID

```
[J3Y5M9uwFXxBo2bKqqd58v44pz2v7gLi8tupjWQLf6bF](https://explorer.solana.com/address/J3Y5M9uwFXxBo2bKqqd58v44pz2v7gLi8tupjWQLf6bF?cluster=devnet)
```

## Architecture

### Core Components

- **Config Account**: Stores pool configuration, token references, and settings
- **Vault Accounts**: Hold deposited tokens for each trading pair
- **LP Token Mint**: Issues liquidity provider tokens as receipts
- **Constant Product Curve**: Implements the x × y = k formula for price discovery

### Key Features

- **Constant Product Formula**: Maintains liquidity depth using the proven x × y = k model
- **Fee Structure**: Configurable trading fees in basis points
- **Slippage Protection**: Minimum/maximum amount validation for all operations
- **PDA Security**: All critical accounts use Program Derived Addresses
- **Proportional Withdrawals**: LP tokens represent proportional ownership

## Instructions

### Initialize Pool

Creates a new AMM pool with specified parameters.

```rust
pub fn initialize(
    ctx: Context<Initialize>,
    seed: u64,
    fee: u16,
    authority: Option<Pubkey>,
) -> Result<()>
```

**Parameters:**
- `seed`: Unique identifier for the pool
- `fee`: Trading fee in basis points (e.g., 30 = 0.30%)
- `authority`: Optional authority for pool updates (None = immutable)

### Deposit Liquidity

Adds liquidity to the pool and mints LP tokens.

```rust
pub fn deposit(
    ctx: Context<Deposit>, 
    amount: u64, 
    max_x: u64, 
    max_y: u64
) -> Result<()>
```

**Parameters:**
- `amount`: Amount of LP tokens to mint
- `max_x`: Maximum token X to deposit (slippage protection)
- `max_y`: Maximum token Y to deposit (slippage protection)

### Withdraw Liquidity

Burns LP tokens and withdraws proportional amounts of both tokens.

```rust
pub fn withdraw(
    ctx: Context<Withdraw>, 
    amount: u64, 
    min_x: u64, 
    min_y: u64
) -> Result<()>
```

**Parameters:**
- `amount`: Amount of LP tokens to burn
- `min_x`: Minimum token X to receive (slippage protection)
- `min_y`: Minimum token Y to receive (slippage protection)

### Swap Tokens

Exchanges one token for another using the constant product formula.

```rust
pub fn swap(
    ctx: Context<Swap>, 
    is_x: bool, 
    amount_in: u64, 
    min_amount_out: u64
) -> Result<()>
```

**Parameters:**
- `is_x`: true for X→Y swap, false for Y→X swap
- `amount_in`: Amount of input tokens
- `min_amount_out`: Minimum output tokens (slippage protection)

## Account Structure

### Config Account

Stores pool configuration and metadata:

```rust
pub struct Config {
    pub seed: u64,              // Pool identifier
    pub authority: Option<Pubkey>, // Update authority
    pub mint_x: Pubkey,         // Token X mint
    pub mint_y: Pubkey,         // Token Y mint
    pub fee: u16,               // Trading fee (basis points)
    pub locked: bool,           // Pool lock status
    pub config_bump: u8,        // PDA bump
    pub lp_bump: u8,            // LP mint PDA bump
}
```

### PDA Seeds

- Config PDA: `["config", seed.to_le_bytes()]`
- LP Mint PDA: `["lp", config_pubkey]`
- Vault X: Associated Token Account of Config PDA
- Vault Y: Associated Token Account of Config PDA

## Error Handling

The protocol includes comprehensive error handling:

- `PoolLocked`: Pool is locked for operations
- `InvalidAmount`: Amount is zero or invalid
- `SlippageExceeded`: Output doesn't meet minimum requirements
- `InsufficientLiquidity`: Not enough liquidity for operation

## Testing

The protocol includes comprehensive tests covering:

- Pool initialization
- Liquidity deposits and withdrawals
- Token swaps in both directions
- Error conditions and edge cases

### Running Tests

```bash
anchor test
```

### Test Results

The protocol has been successfully tested on Solana Devnet with the following transactions:

- **Pool Initialization**: [2S4r7ycCYrqZivvjMfg8kGdymE9VakGxaJKkBFgNBpirqZAvuZAWYaYkeu4DHeAGAXjZUrhawfqHChEk2CHunoEW](https://explorer.solana.com/tx/2S4r7ycCYrqZivvjMfg8kGdymE9VakGxaJKkBFgNBpirqZAvuZAWYaYkeu4DHeAGAXjZUrhawfqHChEk2CHunoEW?cluster=devnet)
- **Liquidity Deposit**: [2TWaW3hHBPZaHEgCxdCstH27R7a6YJNxAMJfQEUdh7uJeZs9JqzaCj1Qn4S8SmpmdKDMFpapALUcWuHjnwLfKpiC](https://explorer.solana.com/tx/2TWaW3hHBPZaHEgCxdCstH27R7a6YJNxAMJfQEUdh7uJeZs9JqzaCj1Qn4S8SmpmdKDMFpapALUcWuHjnwLfKpiC?cluster=devnet)
- **Liquidity Withdrawal**: [4JTJygMbCKoQqQyi3dg92hJ66LLGpyh5pUJWE9g5yF464Lcg9FaBPDVwKwJVHJiX6rG39RkSJUsNgPv6N5AGaZQY](https://explorer.solana.com/tx/4JTJygMbCKoQqQyi3dg92hJ66LLGpyh5pUJWE9g5yF464Lcg9FaBPDVwKwJVHJiX6rG39RkSJUsNgPv6N5AGaZQY?cluster=devnet)
- **Token Swap**: [5CnNr5ry5pmfgD3tmwxDZQgVFRrFWvNreicnkwnKbPviiYRHTb94eCWXztAXsrz2o51zG5mAbfw7tn7ZXW5GeEtp](https://explorer.solana.com/tx/5CnNr5ry5pmfgD3tmwxDZQgVFRrFWvNreicnkwnKbPviiYRHTb94eCWXztAXsrz2o51zG5mAbfw7tn7ZXW5GeEtp?cluster=devnet)

All tests completed successfully, demonstrating the protocol's functionality across core operations.

## Security Features

- **PDA Authority**: All critical operations use Program Derived Addresses
- **Slippage Protection**: All operations include minimum/maximum validation
- **Amount Validation**: Prevents zero-amount and invalid operations
- **Pool Lock**: Emergency lock mechanism for pool operations
- **Decimal Precision**: Proper handling of token decimals

## Dependencies

- `anchor-lang`: Solana program framework
- `anchor-spl`: SPL token program integration
- `constant-product-curve`: Mathematical curve implementation

## License

This project is licensed under the MIT License.

## Development

### Prerequisites

- Rust 1.70+
- Solana CLI 1.16+
- Anchor Framework 0.29+

### Building

```bash
anchor build
```

### Deployment

```bash
anchor deploy
```

## Contributing

Contributions are welcome. Please ensure all tests pass and follow the existing code style.

## Disclaimer

This is experimental software. Use at your own risk. Always audit smart contracts before deploying to mainnet.