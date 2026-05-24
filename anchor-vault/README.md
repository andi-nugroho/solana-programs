# Vault Program

A Solana program that implements a secure SOL vault system using Program Derived Addresses (PDAs). Users can deposit, withdraw, and manage their SOL holdings through a user-specific vault account.

## Overview

The Vault program provides a simple yet secure way to store SOL on-chain with the following features:

- **User-specific vaults**: Each user gets their own vault derived from their public key
- **Secure storage**: Funds are stored in PDAs controlled by the program
- **Full control**: Users can deposit, withdraw, and close their vaults at any time
- **Rent optimization**: Automatic rent exemption and reclamation

## Architecture

The program uses two main Program Derived Addresses (PDAs):

1. **Vault State Account**: Stores configuration data and bump seeds
   - Seeds: `["state", user_pubkey]`
   - Purpose: Store bump values for consistent PDA derivation

2. **Vault Account**: Holds the actual SOL deposits
   - Seeds: `["vault", user_pubkey]`
   - Purpose: Store user's deposited lamports

## Instructions

### Initialize
Creates the vault state account and prepares the vault for deposits.

**Accounts:**
- `user`: Signer and payer for account creation
- `vault_state`: PDA to store vault configuration
- `vault`: PDA that will hold SOL deposits
- `system_program`: Required for account creation

### Deposit
Transfers SOL from the user's account to their vault.

**Parameters:**
- `amount`: Amount of lamports to deposit

**Accounts:**
- `user`: Signer and source of funds
- `vault_state`: Vault configuration (read-only)
- `vault`: Destination for deposited SOL
- `system_program`: Required for SOL transfers

### Withdraw
Transfers SOL from the vault back to the user's account.

**Parameters:**
- `amount`: Amount of lamports to withdraw

**Accounts:**
- `user`: Signer and destination for funds
- `vault_state`: Vault configuration (read-only)
- `vault`: Source of withdrawn SOL
- `system_program`: Required for SOL transfers

### Close
Closes the vault by transferring all remaining SOL to the user and closing the state account.

**Accounts:**
- `user`: Signer and destination for funds
- `vault_state`: Account to be closed
- `vault`: Source of remaining SOL
- `system_program`: Required for SOL transfers

## Security Features

- **PDA Ownership**: All vault accounts are PDAs controlled by the program
- **User Authorization**: Only the vault owner can perform operations
- **Seed Validation**: Consistent PDA derivation using stored bump seeds
- **Rent Protection**: Automatic rent exemption prevents account deletion

## Development

### Prerequisites

- Rust 1.70+
- Solana CLI 1.16+
- Anchor Framework 0.28+
- Node.js 16+

### Building

```bash
anchor build
```

### Testing

```bash
anchor test
```

### Deployment

```bash
anchor deploy
```

## Usage Example

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Vault } from "./target/types/vault";

// Setup
const provider = anchor.AnchorProvider.env();
const program = anchor.workspace.vault as Program<Vault>;

// Derive PDA addresses
const [vaultState] = anchor.web3.PublicKey.findProgramAddressSync(
  [Buffer.from("state"), provider.wallet.publicKey.toBuffer()],
  program.programId
);

const [vault] = anchor.web3.PublicKey.findProgramAddressSync(
  [Buffer.from("vault"), provider.wallet.publicKey.toBuffer()],
  program.programId
);

// Initialize vault
await program.methods
  .initialize()
  .accountsPartial({
    user: provider.wallet.publicKey,
    vaultState,
    vault,
    systemProgram: anchor.web3.SystemProgram.programId
  })
  .rpc();

// Deposit SOL
await program.methods
  .deposit(new anchor.BN(1 * anchor.web3.LAMPORTS_PER_SOL))
  .accountsPartial({
    user: provider.wallet.publicKey,
    vaultState,
    vault,
    systemProgram: anchor.web3.SystemProgram.programId
  })
  .rpc();
```

## Error Handling

The program includes standard Anchor error handling. Common error scenarios:

- **Insufficient funds**: When attempting to withdraw more than available balance
- **Invalid signer**: When non-owner attempts to access vault
- **Account validation**: When provided accounts don't match expected PDAs

## Deployment Information

### Devnet
- **Program ID**: [`G5m8rLBcLLmbV1Jrt78p7FmDD2GhiNSDBQ1Tqzn8Lq5i`](https://explorer.solana.com/address/G5m8rLBcLLmbV1Jrt78p7FmDD2GhiNSDBQ1Tqzn8Lq5i?cluster=devnet)

### Example Transactions

The following transactions demonstrate the complete vault lifecycle on devnet:

**Initialize Vault:**
[`5HjbjkVTRRHvoYpayd5esVjNZHKueYoXLYcqea5sKKdALTs76XRwcVxpRu1uL9VPwYiChQCQtDX9MmEFN4YZxBiT`](https://explorer.solana.com/tx/5HjbjkVTRRHvoYpayd5esVjNZHKueYoXLYcqea5sKKdALTs76XRwcVxpRu1uL9VPwYiChQCQtDX9MmEFN4YZxBiT?cluster=devnet)

**Deposit 2 SOL:**
[`5f6YqDrmjHhGxRmYXowUphigSRrRdfpZ5gdXX58geXUUaLe7HhD8S85mNEE6eBFZugEGU8XqkeHTANKHKsu4Gyjc`](https://explorer.solana.com/tx/5f6YqDrmjHhGxRmYXowUphigSRrRdfpZ5gdXX58geXUUaLe7HhD8S85mNEE6eBFZugEGU8XqkeHTANKHKsu4Gyjc?cluster=devnet)  
*Result: 2,000,000,000 lamports in vault*

**Withdraw 1 SOL:**
[`36US6noCiYAVk1MKZZXWxiLZqcbxZ2kMkWkphRwS8aruUiP7QDpvK5sWQ4aPJzH326Uj6ksu66kejDwPiuQL2kpR`](https://explorer.solana.com/tx/36US6noCiYAVk1MKZZXWxiLZqcbxZ2kMkWkphRwS8aruUiP7QDpvK5sWQ4aPJzH326Uj6ksu66kejDwPiuQL2kpR?cluster=devnet)  
*Result: 1,000,000,000 lamports remaining in vault*

**Close Vault:**
[`5oUnkWkiz8MirnhsimNThBGSk4pgXWLTzWv2r7xSEfsvPhFuM5r53Q3jN7U38H4aMNoBHkZUh2MJBwKjMYYSYkLX`](https://explorer.solana.com/tx/5oUnkWkiz8MirnhsimNThBGSk4pgXWLTzWv2r7xSEfsvPhFuM5r53Q3jN7U38H4aMNoBHkZUh2MJBwKjMYYSYkLX?cluster=devnet)  
*Result: All remaining SOL returned to user, accounts closed*

## License

This program is provided as-is for educational and development purposes. Review and audit thoroughly before any production use.

## Contributing

Contributions are welcome. Please ensure all tests pass and follow the existing code style.

## Support

For issues and questions, please open an issue in the repository.