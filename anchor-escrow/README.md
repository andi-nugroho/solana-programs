# Solana Escrow Program

A secure, decentralized escrow system built on Solana using the Anchor framework. This program enables trustless token swaps between two parties without requiring intermediaries.

## Overview

The escrow program facilitates atomic token swaps by:
1. **Maker** deposits Token A into escrow and specifies the amount of Token B they want in return
2. **Taker** can complete the trade by providing Token B and receiving Token A
3. **Maker** can refund their escrowed tokens if no trade occurs

## Features

- **Trustless Trading**: No intermediaries required
- **Atomic Swaps**: Either both parties get their tokens or the transaction fails
- **Token-2022 Support**: Compatible with both SPL Token and Token-2022 programs
- **Secure PDAs**: Uses Program Derived Addresses for vault authority
- **Rent Optimization**: Automatically closes accounts and refunds rent
- **Decimal Validation**: Uses `transfer_checked` for enhanced security

## Program Structure

### Instructions

#### `make(seed: u64, amount: u64, receive: u64)`
Creates an escrow and deposits tokens from the maker.

**Parameters:**
- `seed`: Unique identifier for the escrow (allows multiple escrows per maker)
- `amount`: Amount of Token A to deposit into escrow
- `receive`: Amount of Token B expected in return

**Accounts:**
- `maker`: Escrow creator (signer, mutable)
- `mint_a`: Token mint for deposited asset
- `mint_b`: Token mint for expected return asset
- `maker_ata_a`: Maker's Token A account
- `escrow`: Escrow state account (PDA)
- `vault`: Token A vault account (PDA)

#### `take()`
Completes the escrow trade by swapping tokens.

**Process:**
1. Taker sends Token B to maker
2. Escrowed Token A is transferred to taker
3. Vault and escrow accounts are closed

**Accounts:**
- `taker`: Trade counterparty (signer, mutable)
- `maker`: Original escrow creator (mutable)
- `mint_a` & `mint_b`: Token mints
- `taker_ata_b`: Taker's Token B account
- `maker_ata_b`: Maker's Token B account (created if needed)
- `taker_ata_a`: Taker's Token A account (created if needed)
- `escrow`: Escrow state account
- `vault`: Token A vault account

#### `refund()`
Allows maker to reclaim their escrowed tokens.

**Process:**
1. All escrowed Token A is returned to maker
2. Vault and escrow accounts are closed

**Accounts:**
- `maker`: Original escrow creator (signer, mutable)
- `mint_a`: Token mint for escrowed asset
- `maker_ata_a`: Maker's Token A account
- `escrow`: Escrow state account
- `vault`: Token A vault account

### State Account

#### `Escrow`
Stores trade parameters and metadata:

```rust
pub struct Escrow {
    pub seed: u64,           // Unique identifier
    pub maker: Pubkey,       // Escrow creator
    pub mint_a: Pubkey,      // Deposited token mint
    pub mint_b: Pubkey,      // Expected token mint
    pub receive: u64,        // Expected Token B amount
    pub bump: u8,            // PDA bump seed
}
```

## Security Features

### PDA Authority
- Vault accounts are owned by the escrow PDA
- Prevents unauthorized token access
- Uses deterministic address derivation

### Validation Constraints
- **Token Program Compatibility**: Validates mints work with specified token program
- **Account Ownership**: Ensures associated token accounts belong to correct authorities
- **Mint Matching**: Verifies token mints match escrow expectations
- **Maker Identity**: Confirms only original maker can refund

### Transfer Safety
- Uses `transfer_checked` for decimal validation
- Prevents precision errors and token mint mismatches
- Atomic operations ensure transaction integrity

## Usage Example

### Setup
```typescript
const SEED = new anchor.BN(1);
const DEPOSIT_AMOUNT = new anchor.BN(1e6);  // 1 Token A
const RECEIVE_AMOUNT = new anchor.BN(1e6);  // 1 Token B

// Derive escrow PDA
const escrow = PublicKey.findProgramAddressSync(
  [
    Buffer.from("escrow"),
    maker.publicKey.toBuffer(),
    SEED.toArrayLike(Buffer, "le", 8)
  ],
  programId
)[0];
```

### Create Escrow
```typescript
await program.methods
  .make(SEED, DEPOSIT_AMOUNT, RECEIVE_AMOUNT)
  .accounts({
    maker: maker.publicKey,
    mintA: mintA.publicKey,
    mintB: mintB.publicKey,
    makerAtaA: makerAtaA,
    escrow: escrow,
    vault: vault,
    tokenProgram: TOKEN_2022_PROGRAM_ID,
    // ... other required accounts
  })
  .signers([maker])
  .rpc();
```

### Complete Trade
```typescript
await program.methods
  .take()
  .accounts({
    taker: taker.publicKey,
    maker: maker.publicKey,
    mintA: mintA.publicKey,
    mintB: mintB.publicKey,
    // ... all required accounts
  })
  .signers([taker])
  .rpc();
```

### Refund Escrow
```typescript
await program.methods
  .refund()
  .accounts({
    maker: maker.publicKey,
    mintA: mintA.publicKey,
    // ... required accounts
  })
  .signers([maker])
  .rpc();
```

## Testing

The program includes comprehensive tests covering:
- Escrow creation and token deposit
- Successful trade completion
- Refund functionality
- Error handling and edge cases

Run tests with:
```bash
anchor test
```
## Deployment Information

### Devnet
- **Program ID**: [`ABagojQQU4h1roF1U2ZC2vqvMVrWcBx2gCq1Gy95KEvJ`](https://explorer.solana.com/address/ABagojQQU4h1roF1U2ZC2vqvMVrWcBx2gCq1Gy95KEvJ?cluster=devnet)

### Example Transactions

The following transactions demonstrate the complete vault lifecycle on devnet:

**Airdrop and Mint:**
[`3J7RzjzbTmureXCsM5zy111nYfeZ54i9pGgshu3CvtDnrgsdfU6v6aAbMVmDemzp7r3RponM2WhUwuonppsxKJq8`](https://explorer.solana.com/transaction/3J7RzjzbTmureXCsM5zy111nYfeZ54i9pGgshu3CvtDnrgsdfU6v6aAbMVmDemzp7r3RponM2WhUwuonppsxKJq8?cluster=devnet)

**Make Escrow:**
[`5iu4AiXVACWzqsNnCJdBU9bE8qZKPMmAwiKKmWBgRcFJM2rF9WygTLob71biQaD77PVG7A5H8GhAMusfGT1i9QTX`](https://explorer.solana.com/transaction/5iu4AiXVACWzqsNnCJdBU9bE8qZKPMmAwiKKmWBgRcFJM2rF9WygTLob71biQaD77PVG7A5H8GhAMusfGT1i9QTX?cluster=devnet)  

**Take Trade:**
[`3Hxz72pV64RMUx49hfNyznieRKsgni54ZtS8o6iEkhCxiDaVMjZ1qzvQjEjsJZr8fBYqgTm8YpKaT9J9x5QWqanA`](https://explorer.solana.com/transaction/3Hxz72pV64RMUx49hfNyznieRKsgni54ZtS8o6iEkhCxiDaVMjZ1qzvQjEjsJZr8fBYqgTm8YpKaT9J9x5QWqanA?cluster=devnet)

## Development

### Build
```bash
anchor build
```

### Deploy
```bash
anchor deploy
```

### Test
```bash
anchor test
```

## Architecture Benefits

1. **No Counterparty Risk**: Tokens are held by program PDAs, not individuals
2. **Composability**: Can be integrated with other DeFi protocols
3. **Cost Efficient**: Minimal rent usage with automatic account cleanup
4. **Upgradeable**: Built with Anchor's upgrade patterns in mind

## License

This program is provided as-is for educational and development purposes. Use at your own risk in production environments.

## Contributing

Contributions are welcome! Please ensure all tests pass and follow Rust/Anchor best practices.