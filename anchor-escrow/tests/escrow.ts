import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram
} from "@solana/web3.js";
import { Escrow } from "../target/types/escrow";

describe("escrow", () => {
  const provider = anchor.getProvider();
  const program = anchor.workspace.escrow as Program<Escrow>;
  const programId = program.programId;
  const tokenProgram = spl.TOKEN_2022_PROGRAM_ID;

  console.log("RPC:", provider.connection.rpcEndpoint);

  const SEED = new anchor.BN(1);

  const confirm = async (signature: string): Promise<string> => {
    const block = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({ signature, ...block });
    return signature;
  };

  const log = async (signature: string): Promise<string> => {
    console.log(
      `Your transaction signature: https://explorer.solana.com/transaction/${signature}?cluster=devnet`
    );
    return signature;
  };

  const [maker, taker, mintA, mintB] = Array.from(
    { length: 4 },
    () => Keypair.generate()
  );

  const [makerAtaA, makerAtaB, takerAtaA, takerAtaB] = [maker, taker]
    .map((a) =>
      [mintA, mintB].map((m) =>
        spl.getAssociatedTokenAddressSync(
          m.publicKey,
          a.publicKey,
          false,
          tokenProgram
        )
      )
    )
    .flat();

  const escrow = PublicKey.findProgramAddressSync(
    [
      Buffer.from("escrow"),
      maker.publicKey.toBuffer(),
      SEED.toArrayLike(Buffer, "le", 8),
    ],
    programId
  )[0];

  const vault = spl.getAssociatedTokenAddressSync(
    mintA.publicKey,
    escrow,
    true,
    tokenProgram
  );

  const accounts = {
    maker: maker.publicKey,
    taker: taker.publicKey,
    mintA: mintA.publicKey,
    mintB: mintB.publicKey,
    makerAtaA,
    makerAtaB,
    takerAtaA,
    takerAtaB,
    escrow,
    vault,
    tokenProgram,
  };

  it("Airdrop & create mint", async () => {
    const lamports = await spl.getMinimumBalanceForRentExemptMint(
      provider.connection as any
    );

    const tx = new anchor.web3.Transaction();

    // Transfer 1 SOL to maker and taker
    tx.instructions.push(
      ...[maker, taker].map((a) =>
        SystemProgram.transfer({
          fromPubkey: provider.publicKey,
          toPubkey: a.publicKey,
          lamports: 1 * LAMPORTS_PER_SOL,
        })
      )
    );

    // Create mintA and mintB
    tx.instructions.push(
      ...[mintA, mintB].map((m) =>
        SystemProgram.createAccount({
          fromPubkey: provider.publicKey,
          newAccountPubkey: m.publicKey,
          lamports,
          space: spl.MINT_SIZE,
          programId: tokenProgram,
        })
      )
    );

    // Init mints, create ATAs, and mint tokens
    tx.instructions.push(
      ...[
        { mint: mintA.publicKey, authority: maker.publicKey, ata: makerAtaA },
        { mint: mintB.publicKey, authority: taker.publicKey, ata: takerAtaB },
      ].flatMap((x) => [
        spl.createInitializeMint2Instruction(
          x.mint,
          6,
          x.authority,
          null,
          tokenProgram
        ),
        spl.createAssociatedTokenAccountIdempotentInstruction(
          provider.publicKey,
          x.ata,
          x.authority,
          x.mint,
          tokenProgram
        ),
        spl.createMintToInstruction(
          x.mint,
          x.ata,
          x.authority,
          1e9,
          undefined,
          tokenProgram
        ),
      ])
    );

    await provider
      .sendAndConfirm(tx, [provider.wallet.payer, maker, taker, mintA, mintB])
      .then(log);
  });

  it("Make", async () => {
    await program.methods
      .make(SEED, new anchor.BN(1e6), new anchor.BN(1e6))
      .accounts({ ...accounts })
      .signers([maker])
      .rpc()
      .then(confirm)
      .then(log);
  });

  xit("Refund", async () => {
    await program.methods
      .refund()
      .accounts({ ...accounts })
      .signers([maker])
      .rpc()
      .then(confirm)
      .then(log);
  });

  it("Take", async () => {
    await program.methods
      .take()
      .accounts({ ...accounts })
      .signers([taker])
      .rpc()
      .then(confirm)
      .then(log);
  });
});
