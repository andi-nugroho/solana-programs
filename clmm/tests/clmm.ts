import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Clmm } from "../target/types/clmm";
import { createMint } from "@solana/spl-token";
import { assert } from "chai";
import { BASE_SQRT_PRICE_X64, tickToSqrtPriceX64 } from "./utils";
import { sqrt } from "bn-sqrt";

describe("clmm", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.clmm as Program<Clmm>;
  const provider = anchor.getProvider();
  const poolCreator = provider.wallet;
  let poolAuthorityPda: anchor.web3.PublicKey;
  let poolAuthorityBump: Number;
  let tokenAMint: anchor.web3.PublicKey;
  let tokenBMint: anchor.web3.PublicKey;
  let lpMintPda: anchor.web3.PublicKey;
  let lpMintBump: Number;
  let vaultAPda: anchor.web3.PublicKey;
  let vaultABump: Number;
  let vaultBPda: anchor.web3.PublicKey;
  let vaultBBump: Number;
  let poolPda: anchor.web3.PublicKey;
  let poolBump: Number;
  let lpCreatorTokenAccount: anchor.web3.PublicKey;

  before(async () => {
    // Create token A mint,assuming tokenA as base as Sol
    tokenAMint = await createMint(
      provider.connection,
      poolCreator.payer,
      poolCreator.publicKey,
      poolCreator.publicKey,
      6
    );

    // Create token B mint,assuming tokenB as base as USDC
    tokenBMint = await createMint(
      provider.connection,
      poolCreator.payer,
      poolCreator.publicKey,
      poolCreator.publicKey,
      6
    );

    [poolAuthorityPda, poolAuthorityBump] =
      anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("authority"), tokenAMint.toBytes(), tokenBMint.toBytes()],
        program.programId
      );

    [lpMintPda, lpMintBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("lp_mint"), tokenAMint.toBytes(), tokenBMint.toBytes()],
      program.programId
    );

    [vaultAPda, vaultABump] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("vault_token"),
        tokenAMint.toBytes(),
        tokenBMint.toBytes(),
        Buffer.from("A"),
      ],
      program.programId
    );

    [vaultBPda, vaultBBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("vault_token"),
        tokenAMint.toBytes(),
        tokenBMint.toBytes(),
        Buffer.from("B"),
      ],
      program.programId
    );

    [poolPda, poolBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), tokenAMint.toBytes(), tokenBMint.toBytes()],
      program.programId
    );
  });

  describe("Initialize Pool", () => {
    it("should initialize a pool with correct parameters", async () => {
      // Assuming current price of 1 SOL = 150 USDC
      const currentPrice = new anchor.BN(150);
      const currentPriceSqrtX64 = sqrt(currentPrice)
        .mul(BASE_SQRT_PRICE_X64)
        .toString();
      const tx = await program.methods
        .initializePool(currentPrice)
        .accounts({
          initializer: poolCreator.publicKey,
          tokenAMint: tokenAMint,
          tokenBMint: tokenBMint,
        })
        .rpc();

      console.log("Initialize pool transaction signature:", tx);

      const poolAccount = await program.account.pool.fetch(poolPda);

      assert.equal(poolAccount.mintA.toString(), tokenAMint.toString());
      assert.equal(poolAccount.mintB.toString(), tokenBMint.toString());
      assert.equal(
        poolAccount.poolAuthority.toString(),
        poolAuthorityPda.toString()
      );
      assert.equal(poolAccount.vaultA.toString(), vaultAPda.toString());
      assert.equal(poolAccount.vaultB.toString(), vaultBPda.toString());
      assert.equal(poolAccount.lpMint.toString(), lpMintPda.toString());

      assert.equal(poolAccount.activeLiquidity.toString(), "0");
      assert.equal(poolAccount.totalLpIssued.toString(), "0");

      assert.equal(poolAccount.sqrtPriceX64.toString(), currentPriceSqrtX64);
      assert.equal(poolAccount.currentTick, 0);

      // Verify vault accounts were created
      const vaultAInfo = await provider.connection.getAccountInfo(vaultAPda);
      const vaultBInfo = await provider.connection.getAccountInfo(vaultBPda);

      assert.isNotNull(vaultAInfo);
      assert.isNotNull(vaultBInfo);

      const lpMintInfo = await provider.connection.getAccountInfo(lpMintPda);
      assert.isNotNull(lpMintInfo);
    });

    it("should fail to initialize pool with same token mints", async () => {
      try {
        await program.methods
          .initializePool(new anchor.BN(100))
          .accounts({
            initializer: poolCreator.publicKey,
            tokenAMint: tokenAMint,
            tokenBMint: tokenAMint, // Same mint for both tokens
          })
          .rpc();

        // Should not reach here
        assert.fail("Expected transaction to fail");
      } catch (error) {
        assert.include(error.message, "Token A and Token B must be different");
      }
    });

    it("should fail to initialize pool twice", async () => {
      try {
        await program.methods
          .initializePool(new anchor.BN(200))
          .accounts({
            initializer: poolCreator.publicKey,
            tokenAMint: tokenAMint,
            tokenBMint: tokenBMint,
          })
          .rpc();

        // Should not reach here
        assert.fail("Expected transaction to fail");
      } catch (error) {
        // Pool already exists, so account creation should fail
        assert.include(error.message, "Simulation failed");
      }
    });
  });

  describe("Initialize Tick", () => {
    const i32ToLeBytes = (value: number): Buffer => {
      const buffer = Buffer.allocUnsafe(4);
      buffer.writeInt32LE(value, 0);
      return buffer;
    };

    it("should successfully initialize a tick with positive index", async () => {
      const tickIndex = 100;

      const tx = await program.methods
        .initializeTick(tickIndex)
        .accounts({
          payer: poolCreator.publicKey,
          tokenAMint: tokenAMint,
          tokenBMint: tokenBMint,
        })
        .rpc();

      console.log("Initialize tick transaction signature:", tx);

      const [tickPda, tickBump] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("tick"), poolPda.toBytes(), i32ToLeBytes(tickIndex)],
        program.programId
      );

      const tickAccount = await program.account.tick.fetch(tickPda);

      assert.equal(tickAccount.index, tickIndex);
      assert.equal(tickAccount.liquidityNet.toString(), "0");
      assert.equal(tickAccount.bump, tickBump);

      // Verify sqrt_price_x64 is calculated correctly
      const expectedSqrtPrice = tickToSqrtPriceX64(tickIndex);
      assert.equal(
        tickAccount.sqrtPriceX64.toString(),
        expectedSqrtPrice.toString()
      );
    });

    it("should successfully initialize a tick with negative index", async () => {
      const tickIndex = -50;

      const [tickPda, tickBump] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("tick"), poolPda.toBytes(), i32ToLeBytes(tickIndex)],
        program.programId
      );

      const tx = await program.methods
        .initializeTick(tickIndex)
        .accountsStrict({
          payer: poolCreator.publicKey,
          tokenAMint: tokenAMint,
          tokenBMint: tokenBMint,
          pool: poolPda,
          tick: tickPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      console.log("Initialize negative tick transaction signature:", tx);
      const tickAccount = await program.account.tick.fetch(tickPda);

      assert.equal(tickAccount.index, tickIndex);
      assert.equal(tickAccount.liquidityNet.toString(), "0");
      assert.equal(tickAccount.bump, tickBump);

      const expectedSqrtPrice = tickToSqrtPriceX64(tickIndex);
      assert.equal(
        tickAccount.sqrtPriceX64.toString(),
        expectedSqrtPrice.toString()
      );
    });

    it("should successfully initialize a tick with zero index", async () => {
      const tickIndex = 0;

      const [tickPda, tickBump] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("tick"), poolPda.toBytes(), i32ToLeBytes(tickIndex)],
        program.programId
      );

      const tx = await program.methods
        .initializeTick(tickIndex)
        .accounts({
          payer: poolCreator.publicKey,
          tokenAMint: tokenAMint,
          tokenBMint: tokenBMint,
        })
        .rpc();

      console.log("Initialize zero tick transaction signature:", tx);

      const tickAccount = await program.account.tick.fetch(tickPda);

      assert.equal(tickAccount.index, tickIndex);
      assert.equal(tickAccount.liquidityNet.toString(), "0");
      assert.equal(tickAccount.bump, tickBump);

      // For tick 0: sqrt_price_x64 should equal BASE_SQRT_PRICE_X64
      assert.equal(
        tickAccount.sqrtPriceX64.toString(),
        BASE_SQRT_PRICE_X64.toString()
      );
    });

    it("should initialize multiple different ticks successfully", async () => {
      const tickIndices = [200, -200, 1000, -1000];

      for (const tickIndex of tickIndices) {
        const [tickPda, tickBump] =
          anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("tick"), poolPda.toBytes(), i32ToLeBytes(tickIndex)],
            program.programId
          );

        const tx = await program.methods
          .initializeTick(tickIndex)
          .accountsStrict({
            payer: poolCreator.publicKey,
            tokenAMint: tokenAMint,
            tokenBMint: tokenBMint,
            pool: poolPda,
            tick: tickPda,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .rpc();

        console.log(`Initialize tick ${tickIndex} transaction signature:`, tx);

        const tickAccount = await program.account.tick.fetch(tickPda);
        assert.equal(tickAccount.index, tickIndex);
        assert.equal(tickAccount.liquidityNet.toString(), "0");
        assert.equal(tickAccount.bump, tickBump);
      }
    });

    it("should fail to initialize the same tick twice", async () => {
      const tickIndex = 300;

      const [tickPda] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("tick"), poolPda.toBytes(), i32ToLeBytes(tickIndex)],
        program.programId
      );

      // First initialization should succeed
      await program.methods
        .initializeTick(tickIndex)
        .accountsStrict({
          payer: poolCreator.publicKey,
          tokenAMint: tokenAMint,
          tokenBMint: tokenBMint,
          pool: poolPda,
          tick: tickPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      // Second initialization should fail
      try {
        await program.methods
          .initializeTick(tickIndex)
          .accountsStrict({
            payer: poolCreator.publicKey,
            tokenAMint: tokenAMint,
            tokenBMint: tokenBMint,
            pool: poolPda,
            tick: tickPda,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .rpc();

        assert.fail("Expected transaction to fail");
      } catch (error) {
        assert.include(error.message.toLowerCase(), "account");
      }
    });

    it("should handle large positive and negative tick indices", async () => {
      const largePositiveTickIndex = 10000;
      const largeNegativeTickIndex = -10000;

      // Test large positive tick
      const [positiveTickPda, positiveTickBump] =
        anchor.web3.PublicKey.findProgramAddressSync(
          [
            Buffer.from("tick"),
            poolPda.toBytes(),
            i32ToLeBytes(largePositiveTickIndex),
          ],
          program.programId
        );

      await program.methods
        .initializeTick(largePositiveTickIndex)
        .accountsStrict({
          payer: poolCreator.publicKey,
          tokenAMint: tokenAMint,
          tokenBMint: tokenBMint,
          pool: poolPda,
          tick: positiveTickPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      const positiveTickAccount = await program.account.tick.fetch(
        positiveTickPda
      );
      assert.equal(positiveTickAccount.index, largePositiveTickIndex);

      // Test large negative tick
      const [negativeTickPda, negativeTickBump] =
        anchor.web3.PublicKey.findProgramAddressSync(
          [
            Buffer.from("tick"),
            poolPda.toBytes(),
            i32ToLeBytes(largeNegativeTickIndex),
          ],
          program.programId
        );

      await program.methods
        .initializeTick(largeNegativeTickIndex)
        .accountsStrict({
          payer: poolCreator.publicKey,
          tokenAMint: tokenAMint,
          tokenBMint: tokenBMint,
          pool: poolPda,
          tick: negativeTickPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      const negativeTickAccount = await program.account.tick.fetch(
        negativeTickPda
      );
      assert.equal(negativeTickAccount.index, largeNegativeTickIndex);
    });

    it("should correctly calculate sqrt_price_x64 for various tick indices", async () => {
      const testCases = [
        { tickIndex: 500, description: "moderate positive tick" },
        { tickIndex: -500, description: "moderate negative tick" },
        { tickIndex: 2000, description: "large positive tick" },
        { tickIndex: -2000, description: "large negative tick" },
      ];

      for (const testCase of testCases) {
        const [tickPda] = anchor.web3.PublicKey.findProgramAddressSync(
          [
            Buffer.from("tick"),
            poolPda.toBytes(),
            i32ToLeBytes(testCase.tickIndex),
          ],
          program.programId
        );

        await program.methods
          .initializeTick(testCase.tickIndex)
          .accountsStrict({
            payer: poolCreator.publicKey,
            tokenAMint: tokenAMint,
            tokenBMint: tokenBMint,
            pool: poolPda,
            tick: tickPda,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .rpc();

        const tickAccount = await program.account.tick.fetch(tickPda);

        // Calculate expected sqrt_price_x64 using the same utility function as other tests
        const expectedSqrtPrice = tickToSqrtPriceX64(testCase.tickIndex);

        assert.equal(
          tickAccount.sqrtPriceX64.toString(),
          expectedSqrtPrice.toString(),
          `sqrt_price_x64 calculation failed for ${testCase.description}`
        );
      }
    });
  });
});