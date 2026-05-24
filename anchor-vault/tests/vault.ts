import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Vault } from "../target/types/vault";

describe("vault", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.vault as Program<Vault>;

  // Pre-compute PDA addresses for consistent use across all tests
  const vaultState = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("state"), provider.wallet.publicKey.toBuffer()],
    program.programId
  )[0];

  const vault = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), provider.wallet.publicKey.toBuffer()],
    program.programId
  )[0];

  it("Initializes vault state and SOL storage accounts", async () => {
    const tx = await program.methods
      .initialize()
      .accountsPartial({
        user: provider.wallet.publicKey,
        vaultState,
        vault,
        systemProgram: anchor.web3.SystemProgram.programId
      })
      .rpc()

    console.log("\nInitialization transaction signature:", tx);
    console.log("Vault account info:", await provider.connection.getAccountInfo(vault));
    // https://explorer.solana.com/tx/5HjbjkVTRRHvoYpayd5esVjNZHKueYoXLYcqea5sKKdALTs76XRwcVxpRu1uL9VPwYiChQCQtDX9MmEFN4YZxBiT?cluster=devnet
  });

  it("Deposits SOL into the vault", async () => {
    const depositAmount = new anchor.BN(2 * anchor.web3.LAMPORTS_PER_SOL);

    const tx = await program.methods
      .deposit(depositAmount)
      .accountsPartial({
        user: provider.wallet.publicKey,
        vaultState,
        vault,
        systemProgram: anchor.web3.SystemProgram.programId
      })
      .rpc()

    console.log("\nDeposit transaction signature:", tx);
    console.log("Vault balance after deposit:", (await provider.connection.getBalance(vault)).toString(), "lamports");
    // https://explorer.solana.com/tx/5f6YqDrmjHhGxRmYXowUphigSRrRdfpZ5gdXX58geXUUaLe7HhD8S85mNEE6eBFZugEGU8XqkeHTANKHKsu4Gyjc?cluster=devnet
  });

  it("Withdraws SOL from the vault", async () => {
    const withdrawAmount = new anchor.BN(1 * anchor.web3.LAMPORTS_PER_SOL);

    const tx = await program.methods
      .withdraw(withdrawAmount)
      .accountsPartial({
        user: provider.wallet.publicKey,
        vaultState,
        vault,
        systemProgram: anchor.web3.SystemProgram.programId
      })
      .rpc()

    console.log("\nWithdrawal transaction signature:", tx);
    console.log("Vault balance after withdrawal:", (await provider.connection.getBalance(vault)).toString(), "lamports");
    // https://explorer.solana.com/tx/36US6noCiYAVk1MKZZXWxiLZqcbxZ2kMkWkphRwS8aruUiP7QDpvK5sWQ4aPJzH326Uj6ksu66kejDwPiuQL2kpR?cluster=devnet
  });

  it("Closes vault and returns all remaining SOL to user", async () => {
    const tx = await program.methods
      .close()
      .accountsPartial({
        user: provider.wallet.publicKey,
        vaultState,
        vault,
        systemProgram: anchor.web3.SystemProgram.programId
      })
      .rpc()

    console.log("\nClose transaction signature:", tx);
    console.log("Vault account info after closing:", await provider.connection.getAccountInfo(vault));
    console.log("Vault state should be null (closed):", await provider.connection.getAccountInfo(vaultState));
    // https://explorer.solana.com/tx/5oUnkWkiz8MirnhsimNThBGSk4pgXWLTzWv2r7xSEfsvPhFuM5r53Q3jN7U38H4aMNoBHkZUh2MJBwKjMYYSYkLX?cluster=devnet
  });
});