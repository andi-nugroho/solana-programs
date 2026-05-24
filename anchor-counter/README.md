# Anchor Counter

Learning the on-chain program development using Anchor framework from Solana Courses

## Anchor Program Structure

Some important macros provided by Anchor are:
- `declare_id!` - a macro for declaring the program’s onchain address
- `#[program]` - an attribute macro used to denote the module containing the program’s instruction handlers.
- `Accounts` - a trait applied to structs representing the list of accounts required for an instruction.
- `#[account]` - an attribute macro used to define custom account types for the program.

## Breakdown of the program

1. Use this import to gain access to common anchor features
```rs
use anchor_lang::prelude::*;
```

2. Declare your on-chain programId using this macro
```rs
declare_id!("4GxG4XwJa7P1t7c217Tu3yR1J51qxJ8ujmLUwQxwYCwa");
```

3. Define Account Structure
```rs
#[account]
#[derive(InitSpace)]
pub struct Counter {
    pub count: u64,
}
```

5. Implement Context type
```rs
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = user,
        space = DISCRIMINATOR + Counter::INIT_SPACE,
    )]
    pub counter: Account<'info, Counter>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}
```

6. Initialize instruction handler
```rs
pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
  let counter = &mut ctx.accounts.counter;
  counter.count = 0;
  msg!("Counter Account Created");
  msg!("Current Count: { }", counter.count);

  Ok(())
}
```

7. Build and Write Tests in `tests/anchor-counter.ts` 
```ts
describe("anchor-counter", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.AnchorCounter as Program<AnchorCounter>;

  const counter = anchor.web3.Keypair.generate();

  it("Initilalize Counter!", async () => {
    await program.methods
      .initialize()
      .accounts({ counter: counter.publicKey })
      .signers([counter])
      .rpc();

    const account = await program.account.counter.fetch(counter.publicKey);
    expect(account.count.toNumber()).to.equal(0);
  });
}
```

8. Test using the following command
```sh
anchor test

# If local-test-validator is already running 
anchor test --skip-local-validator
```

## Key Takeaways

The whole program structure can be broadly divided into three parts:

1. Account constraints
2. Instruction handlers
3. Accounts
