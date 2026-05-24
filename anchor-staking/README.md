# Solana NFT Staking Program

A Solana program that allows users to stake NFTs and earn reward points. The program implements a time-based freeze period mechanism to prevent immediate unstaking and includes configurable parameters for staking rewards.

## Features

- **NFT Staking**: Users can stake their NFTs to earn reward points
- **Configurable Parameters**: Admin can set points per stake, maximum unstake limit, and freeze period
- **Time-based Freeze Period**: Prevents immediate unstaking to ensure commitment
- **Reward Token System**: Creates a reward token mint for future reward distributions
- **Secure Vault System**: NFTs are stored in program-controlled vaults during staking
- **User Account Management**: Tracks individual user staking statistics and points

## Program Structure

### Instructions

1. **Initialize Config** - Sets up global staking parameters (admin only)
2. **Initialize User** - Creates a user account for staking participation
3. **Stake** - Stakes an NFT and awards points to the user
4. **Unstake** - Unstakes an NFT after the freeze period has passed
5. **Claim Rewards** - Claims reward tokens for staked NFTs

### Account Types

- **StakeConfig**: Global configuration storing staking parameters
- **UserAccount**: Individual user data including points and staked amount
- **StakeAccount**: Records for individual staked NFTs with timestamps

### Program Derived Addresses (PDAs)

- **Config PDA**: `[b"config"]` - Global configuration account
- **Reward Mint PDA**: `[b"rewards", config.key()]` - Reward token mint
- **User Account PDA**: `[b"user", user.key()]` - User-specific staking data
- **Vault PDA**: `[b"vault", nft_mint.key()]` - NFT storage vault
- **Stake Account PDA**: `[b"stake", user.key(), nft_mint.key()]` - Individual stake records

## Usage

### Configuration Parameters

When initializing the config, you can set:

- **points_per_stake**: Points awarded per staked NFT (u8)
- **max_unstake**: Maximum number of NFTs that can be unstaked at once (u8)
- **freeze_period**: Required staking duration in seconds (u32)

### Basic Flow

1. **Admin Setup**: Initialize the global configuration
2. **User Registration**: Users initialize their staking accounts
3. **Staking**: Users stake NFTs and receive points
4. **Unstaking**: Users can unstake after the freeze period expires

### Example Usage

```typescript
// Initialize config (admin only)
await program.methods
  .initializeConfig(10, 5, 86400) // 10 points, max 5 unstake, 24h freeze
  .accounts({
    admin: adminPublicKey,
    config: configPda,
    rewardMint: rewardMintPda,
    systemProgram: SystemProgram.programId,
    tokenProgram: TOKEN_PROGRAM_ID,
    rent: SYSVAR_RENT_PUBKEY,
  })
  .rpc();

// Initialize user account
await program.methods
  .initializeUser()
  .accounts({
    user: userPublicKey,
    userAccount: userAccountPda,
    systemProgram: SystemProgram.programId,
  })
  .rpc();

// Stake NFT
await program.methods
  .stake()
  .accounts({
    user: userPublicKey,
    userAccount: userAccountPda,
    config: configPda,
    nftMint: nftMintPublicKey,
    userNftAta: userNftAta,
    vaultAta: vaultAta,
    stakeAccount: stakeAccountPda,
    tokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
    rent: SYSVAR_RENT_PUBKEY,
    clock: SYSVAR_CLOCK_PUBKEY,
  })
  .rpc();

// Unstake NFT (after freeze period)
await program.methods
  .unstake()
  .accounts({
    user: userPublicKey,
    userAccount: userAccountPda,
    config: configPda,
    nftMint: nftMintPublicKey,
    stakeAccount: stakeAccountPda,
    vaultAta: vaultAta,
    userNftAta: userNftAta,
    tokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
    rent: SYSVAR_RENT_PUBKEY,
    clock: SYSVAR_CLOCK_PUBKEY,
  })
  .rpc();

// Claim Rewards
 await program.methods
  .claimRewards()
  .accounts({
    user: user.publicKey,
    userAccount: userAccountPda,
    config: configPda,
    rewardMint: rewardMintPda,
    tokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
    rent: SYSVAR_RENT_PUBKEY,
  })
  .signers([user])
  .rpc();
```

## Testing

The program includes comprehensive tests covering:

- Configuration initialization
- User account management
- NFT staking operations
- Freeze period enforcement
- Unstaking functionality
- Error handling and edge cases

Run the test suite:
```bash
anchor test
```

Test coverage includes:
- Successful operations for all instructions
- Proper error handling for invalid operations
- State verification after each operation
- Token balance verification
- Account closure verification

## Deployment Information

### Devnet
- **Program ID**: [`DND8bE3qTXHMJmNQz1GgT7Ci1mkVtCy12GmK3jnWt5DF`](https://explorer.solana.com/address/DND8bE3qTXHMJmNQz1GgT7Ci1mkVtCy12GmK3jnWt5DF?cluster=devnet)

### Example Transactions

The following transactions demonstrate the complete vault lifecycle on devnet:

**Initialize Config:**
[`WqBsemoPm2F2xjx7ssdq4wmuzeb2mX63KVgi7APQEpAmjDbZX3qrBUgjvTyQDvNfaR4weZbK3xTPQpCAPrTURhU`](https://explorer.solana.com/transaction/WqBsemoPm2F2xjx7ssdq4wmuzeb2mX63KVgi7APQEpAmjDbZX3qrBUgjvTyQDvNfaR4weZbK3xTPQpCAPrTURhU?cluster=devnet)

**Initialize User:**
[`5QbreKJEKnoMCBEbHw4z6bjSfWWUU7GEYuAeJTbVjRwRSe3kc4nuszdbjPMPYxM1D4v1PYHozm8giVFM5TTFkta8`](https://explorer.solana.com/transaction/5QbreKJEKnoMCBEbHw4z6bjSfWWUU7GEYuAeJTbVjRwRSe3kc4nuszdbjPMPYxM1D4v1PYHozm8giVFM5TTFkta8?cluster=devnet)

**Stake NFT 1:**
[`5XcwroqdoSyqw8RRmSYHQRkKUArMEKrT3iKQwuaS4CgvYmA4o9fsqJSpSbWu1XXAucqkTT7XcDQ7QXXnQoN3Tkan`](https://explorer.solana.com/transaction/5XcwroqdoSyqw8RRmSYHQRkKUArMEKrT3iKQwuaS4CgvYmA4o9fsqJSpSbWu1XXAucqkTT7XcDQ7QXXnQoN3Tkan?cluster=devnet)  

**Unstake NFT 1:**
[`55aiLj9PcEye4xYnW58Ys1TfXF1ER4YvSZWzkjtuSkhEnd94QYG1Nq9qSguGpCqSGbSPyCayLCAG3zbqCbv5CymS`](https://explorer.solana.com/transaction/55aiLj9PcEye4xYnW58Ys1TfXF1ER4YvSZWzkjtuSkhEnd94QYG1Nq9qSguGpCqSGbSPyCayLCAG3zbqCbv5CymS?cluster=devnet)

**Stake NFT 2:**
[`4jyRNgh5zuxLuTapemMwRABJzS58aYvb69sZcK6XKosRDm7aSB4yTL3PT4nwXCuzkMC7HJku1c5vqXknySDrRmrn`](https://explorer.solana.com/transaction/4jyRNgh5zuxLuTapemMwRABJzS58aYvb69sZcK6XKosRDm7aSB4yTL3PT4nwXCuzkMC7HJku1c5vqXknySDrRmrn?cluster=devnet)  

**Claim Rewards:**
[`5e1721JyTgL173whKwcpEH1U25g8hM5KBVJN9xA1KouqMxuiK4MidLGVAPDVBWE6LM4J44prukF4oPeCfhh7Yad4`](https://explorer.solana.com/transaction/5e1721JyTgL173whKwcpEH1U25g8hM5KBVJN9xA1KouqMxuiK4MidLGVAPDVBWE6LM4J44prukF4oPeCfhh7Yad4?cluster=devnet)

## Error Handling

The program defines custom errors for better debugging:

- **NotFrozen**: Attempted to unstake before freeze period expired
- **NothingToUnstake**: User has no staked NFTs to unstake

## License

This program is provided as-is for educational and development purposes. Use at your own risk in production environments.

## Contributing

Contributions are welcome! Please ensure all tests pass and follow Rust/Anchor best practices.