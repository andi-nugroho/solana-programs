import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { EscrowProgram } from "../target/types/escrow_program";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  mintTo,
  getAccount,
  createAssociatedTokenAccount,
} from "@solana/spl-token";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";

describe("escrow_program", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.EscrowProgram as Program<EscrowProgram>;

  let mint: PublicKey;
  let initializer: Keypair;
  let taker: Keypair;
  let initializerTokenAccount: PublicKey;
  let takerTokenAccount: PublicKey;
  let escrowState: PublicKey;
  let escrowVault: Keypair;

  const escrowAmount = new anchor.BN(1000);

  before(async () => {
    initializer = Keypair.generate();
    taker = Keypair.generate();
    escrowVault = Keypair.generate();

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        initializer.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      )
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        taker.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      )
    );

    mint = await createMint(
      provider.connection,
      initializer,
      initializer.publicKey,
      null,
      6
    );

    initializerTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      initializer,
      mint,
      initializer.publicKey
    );

    takerTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      taker,
      mint,
      taker.publicKey
    );

    await mintTo(
      provider.connection,
      initializer,
      mint,
      initializerTokenAccount,
      initializer.publicKey,
      10000
    );

    [escrowState] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("escrow"),
        initializer.publicKey.toBuffer(),
        mint.toBuffer(),
      ],
      program.programId
    );
  });

  it("Initialize escrow successfully", async () => {
    const tx = await program.methods
      .initializeEscrow(escrowAmount)
      .accounts({
        initializer: initializer.publicKey,
        mint: mint,
        escrowVault: escrowVault.publicKey,
      })
      .signers([initializer, escrowVault])
      .rpc();

    const escrowAccount = await program.account.escrowState.fetch(escrowState);
    expect(escrowAccount.initializer.toString()).to.equal(
      initializer.publicKey.toString()
    );
    expect(escrowAccount.vault.toString()).to.equal(
      escrowVault.publicKey.toString()
    );
    expect(escrowAccount.mint.toString()).to.equal(mint.toString());
    expect(escrowAccount.amount.toString()).to.equal(escrowAmount.toString());
    expect(escrowAccount.isActive).to.be.true;

    const vaultAccount = await getAccount(
      provider.connection,
      escrowVault.publicKey
    );
    expect(vaultAccount.amount.toString()).to.equal(escrowAmount.toString());

    const initializerAccount = await getAccount(
      provider.connection,
      initializerTokenAccount
    );
    expect(initializerAccount.amount.toString()).to.equal("9000");
  });

  it("Exchange tokens successfully", async () => {
    const tx = await program.methods
      .exchange()
      .accounts({
        taker: taker.publicKey,
        initializer: initializer.publicKey,
        mint: mint,
        escrowVault: escrowVault.publicKey,
      })
      .signers([taker])
      .rpc();

    const escrowAccount = await program.account.escrowState.fetch(escrowState);
    expect(escrowAccount.isActive).to.be.false;

    const takerAccount = await getAccount(
      provider.connection,
      takerTokenAccount
    );
    expect(takerAccount.amount.toString()).to.equal(escrowAmount.toString());

    const vaultAccount = await getAccount(
      provider.connection,
      escrowVault.publicKey
    );
    expect(vaultAccount.amount.toString()).to.equal("0");
  });

  describe("Error cases with different initializers", () => {
    let initializer2: Keypair;
    let initializer2TokenAccount: PublicKey;
    let escrowState2: PublicKey;
    let escrowVault2: Keypair;
    let taker2: Keypair;
    let taker2TokenAccount: PublicKey;

    before(async () => {
      initializer2 = Keypair.generate();
      taker2 = Keypair.generate();
      escrowVault2 = Keypair.generate();

      await provider.connection.confirmTransaction(
        await provider.connection.requestAirdrop(
          initializer2.publicKey,
          2 * anchor.web3.LAMPORTS_PER_SOL
        )
      );
      await provider.connection.confirmTransaction(
        await provider.connection.requestAirdrop(
          taker2.publicKey,
          2 * anchor.web3.LAMPORTS_PER_SOL
        )
      );

      initializer2TokenAccount = await createAssociatedTokenAccount(
        provider.connection,
        initializer2,
        mint,
        initializer2.publicKey
      );

      taker2TokenAccount = await createAssociatedTokenAccount(
        provider.connection,
        taker2,
        mint,
        taker2.publicKey
      );

      await mintTo(
        provider.connection,
        initializer,
        mint,
        initializer2TokenAccount,
        initializer.publicKey,
        5000
      );

      [escrowState2] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("escrow"),
          initializer2.publicKey.toBuffer(),
          mint.toBuffer(),
        ],
        program.programId
      );

      await program.methods
        .initializeEscrow(new anchor.BN(2000))
        .accounts({
          initializer: initializer2.publicKey,
          mint: mint,
          escrowVault: escrowVault2.publicKey,
        })
        .signers([initializer2, escrowVault2])
        .rpc();
    });

    it("Fails to exchange on inactive escrow", async () => {
      try {
        await program.methods
          .exchange()
          .accounts({
            taker: taker.publicKey,
            initializer: initializer.publicKey,
            mint: mint,
            escrowVault: escrowVault.publicKey,
          })
          .signers([taker])
          .rpc();

        expect.fail("Should have failed");
      } catch (error) {
        expect(error.error.errorMessage).to.include(
          "Escrow is no longer active"
        );
      }
    });

    it("Fails to cancel with unauthorized signer", async () => {
      try {
        await program.methods
          .cancel()
          .accounts({
            initializer: initializer2.publicKey,
            mint: mint,
            escrowVault: escrowVault2.publicKey,
          })
          .signers([taker2])
          .rpc();

        expect.fail("Should have failed");
      } catch (error) {
        expect(error.message).to.include("unknown signer");
      }
    });

    it("Cancel escrow successfully", async () => {
      const initialBalance = await getAccount(
        provider.connection,
        initializer2TokenAccount
      );

      const tx = await program.methods
        .cancel()
        .accounts({
          initializer: initializer2.publicKey,
          mint: mint,
          escrowVault: escrowVault2.publicKey,
        })
        .signers([initializer2])
        .rpc();

      const escrowAccount = await program.account.escrowState.fetch(
        escrowState2
      );
      expect(escrowAccount.isActive).to.be.false;

      const finalBalance = await getAccount(
        provider.connection,
        initializer2TokenAccount
      );
      expect(finalBalance.amount.toString()).to.equal("5000");
      const vaultAccount = await getAccount(
        provider.connection,
        escrowVault2.publicKey
      );
      expect(vaultAccount.amount.toString()).to.equal("0");
    });

    it("Fails to cancel already inactive escrow", async () => {
      try {
        await program.methods
          .cancel()
          .accounts({
            initializer: initializer2.publicKey,
            mint: mint,
            escrowVault: escrowVault2.publicKey,
          })
          .signers([initializer2])
          .rpc();

        expect.fail("Should have failed");
      } catch (error) {
        expect(error.error.errorMessage).to.include(
          "Escrow is no longer active"
        );
      }
    });
  });

  describe("Edge cases with different initializers", () => {
    let initializer3: Keypair;
    let initializer3TokenAccount: PublicKey;
    let escrowState3: PublicKey;
    let escrowVault3: Keypair;

    before(async () => {
      initializer3 = Keypair.generate();
      escrowVault3 = Keypair.generate();

      await provider.connection.confirmTransaction(
        await provider.connection.requestAirdrop(
          initializer3.publicKey,
          2 * anchor.web3.LAMPORTS_PER_SOL
        )
      );

      initializer3TokenAccount = await createAssociatedTokenAccount(
        provider.connection,
        initializer3,
        mint,
        initializer3.publicKey
      );

      await mintTo(
        provider.connection,
        initializer,
        mint,
        initializer3TokenAccount,
        initializer.publicKey,
        1000
      );

      [escrowState3] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("escrow"),
          initializer3.publicKey.toBuffer(),
          mint.toBuffer(),
        ],
        program.programId
      );
    });

    it("Initialize escrow with zero amount", async () => {
      const tx = await program.methods
        .initializeEscrow(new anchor.BN(0))
        .accounts({
          initializer: initializer3.publicKey,
          mint: mint,
          escrowVault: escrowVault3.publicKey,
        })
        .signers([initializer3, escrowVault3])
        .rpc();

      const escrowAccount = await program.account.escrowState.fetch(
        escrowState3
      );
      expect(escrowAccount.amount.toString()).to.equal("0");
      expect(escrowAccount.isActive).to.be.true;
    });

    it("Exchange zero amount escrow", async () => {
      const taker3 = Keypair.generate();
      await provider.connection.confirmTransaction(
        await provider.connection.requestAirdrop(
          taker3.publicKey,
          2 * anchor.web3.LAMPORTS_PER_SOL
        )
      );

      const taker3TokenAccount = await createAssociatedTokenAccount(
        provider.connection,
        taker3,
        mint,
        taker3.publicKey
      );

      const tx = await program.methods
        .exchange()
        .accounts({
          taker: taker3.publicKey,
          initializer: initializer3.publicKey,
          mint: mint,
          escrowVault: escrowVault3.publicKey,
        })
        .signers([taker3])
        .rpc();

      const escrowAccount = await program.account.escrowState.fetch(
        escrowState3
      );
      expect(escrowAccount.isActive).to.be.false;

      const takerAccount = await getAccount(
        provider.connection,
        taker3TokenAccount
      );
      expect(takerAccount.amount.toString()).to.equal("0");
    });
  });

  describe("Multiple escrows with same mint but different initializers", () => {
    it("Create multiple escrows simultaneously", async () => {
      const initializers = [];
      const escrowStates = [];
      const escrowVaults = [];

      for (let i = 0; i < 3; i++) {
        const newInitializer = Keypair.generate();
        const newEscrowVault = Keypair.generate();

        await provider.connection.confirmTransaction(
          await provider.connection.requestAirdrop(
            newInitializer.publicKey,
            2 * anchor.web3.LAMPORTS_PER_SOL
          )
        );

        const tokenAccount = await createAssociatedTokenAccount(
          provider.connection,
          newInitializer,
          mint,
          newInitializer.publicKey
        );

        await mintTo(
          provider.connection,
          initializer,
          mint,
          tokenAccount,
          initializer.publicKey,
          1000 * (i + 1)
        );

        const [newEscrowState] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("escrow"),
            newInitializer.publicKey.toBuffer(),
            mint.toBuffer(),
          ],
          program.programId
        );

        initializers.push(newInitializer);
        escrowStates.push(newEscrowState);
        escrowVaults.push(newEscrowVault);

        await program.methods
          .initializeEscrow(new anchor.BN(500 * (i + 1)))
          .accounts({
            initializer: newInitializer.publicKey,
            mint: mint,
            escrowVault: newEscrowVault.publicKey,
          })
          .signers([newInitializer, newEscrowVault])
          .rpc();
      }

      for (let i = 0; i < 3; i++) {
        const escrowAccount = await program.account.escrowState.fetch(
          escrowStates[i]
        );
        expect(escrowAccount.isActive).to.be.true;
        expect(escrowAccount.amount.toString()).to.equal(
          (500 * (i + 1)).toString()
        );
        expect(escrowAccount.initializer.toString()).to.equal(
          initializers[i].publicKey.toString()
        );
      }
    });
  });
});