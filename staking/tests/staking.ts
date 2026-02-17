import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Staking } from "../target/types/staking";
import { assert } from "chai";
import * as spl from "@solana/spl-token";

describe("Staking Program", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();
  const program = anchor.workspace.Staking as Program<Staking>;

  let user1 = anchor.web3.Keypair.generate();
  let user2 = anchor.web3.Keypair.generate();

  let user1StakeAccountPda: anchor.web3.PublicKey;
  let user1StakeAccountBump: number;
  let user2StakeAccountPda: anchor.web3.PublicKey;
  let user2StakeAccountBump: number;

  let rewardMint: anchor.web3.PublicKey;
  let mintAuthority: anchor.web3.PublicKey;
  let user1TokenAccount: anchor.web3.PublicKey;
  let user2TokenAccount: anchor.web3.PublicKey;

  before(async () => {
    const airdropTx1 = await provider.connection.requestAirdrop(
      user1.publicKey,
      5 * anchor.web3.LAMPORTS_PER_SOL
    );
    const airdropTx2 = await provider.connection.requestAirdrop(
      user2.publicKey,
      5 * anchor.web3.LAMPORTS_PER_SOL
    );

    await provider.connection.confirmTransaction(airdropTx1);
    await provider.connection.confirmTransaction(airdropTx2);

    [user1StakeAccountPda, user1StakeAccountBump] =
      anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("staked_account"), user1.publicKey.toBytes()],
        program.programId
      );

    [user2StakeAccountPda, user2StakeAccountBump] =
      anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("staked_account"), user2.publicKey.toBytes()],
        program.programId
      );

    [mintAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("mint_authority")],
      program.programId
    );

    rewardMint = await spl.createMint(
      provider.connection,
      user1,
      mintAuthority,
      null,
      6
    );

    user1TokenAccount = await spl.createAssociatedTokenAccount(
      provider.connection,
      user1,
      rewardMint,
      user1.publicKey
    );

    user2TokenAccount = await spl.createAssociatedTokenAccount(
      provider.connection,
      user2,
      rewardMint,
      user2.publicKey
    );
  });

  describe("Stake Account Creation", () => {
    it("Should create stake account for user1", async () => {
      const tx = await program.methods
        .createStakeAccount()
        .accounts({
          user: user1.publicKey,
        })
        .signers([user1])
        .rpc();

      const userStakedAccount = await program.account.stakeAccount.fetch(
        user1StakeAccountPda
      );

      assert.strictEqual(
        userStakedAccount.owner.toBase58(),
        user1.publicKey.toBase58()
      );
      assert.strictEqual(userStakedAccount.stakedAmount.toNumber(), 0);
      assert.strictEqual(userStakedAccount.totalPoints.toNumber(), 0);
      assert.strictEqual(userStakedAccount.stakeTimestamp.toNumber(), 0);
      assert.strictEqual(userStakedAccount.bump, user1StakeAccountBump);
    });

    it("Should create stake account for user2", async () => {
      const tx = await program.methods
        .createStakeAccount()
        .accounts({
          user: user2.publicKey,
        })
        .signers([user2])
        .rpc();

      const userStakedAccount = await program.account.stakeAccount.fetch(
        user2StakeAccountPda
      );

      assert.strictEqual(
        userStakedAccount.owner.toBase58(),
        user2.publicKey.toBase58()
      );
      assert.strictEqual(userStakedAccount.stakedAmount.toNumber(), 0);
      assert.strictEqual(userStakedAccount.totalPoints.toNumber(), 0);
      assert.strictEqual(userStakedAccount.stakeTimestamp.toNumber(), 0);
      assert.strictEqual(userStakedAccount.bump, user2StakeAccountBump);
    });

    it("Should fail to create duplicate stake account", async () => {
      try {
        await program.methods
          .createStakeAccount()
          .accounts({
            user: user1.publicKey,
          })
          .signers([user1])
          .rpc();
        assert.fail("Should have failed to create duplicate account");
      } catch (error) {
        assert.include(error.toString(), "already in use");
      }
    });
  });

  describe("Staking Operations", () => {
    it("Should stake 1 SOL successfully", async () => {
      const stakeAmount = new anchor.BN(anchor.web3.LAMPORTS_PER_SOL);

      const tx = await program.methods
        .stake(stakeAmount)
        .accounts({
          user: user1.publicKey,
        })
        .signers([user1])
        .rpc();

      const userStakedAccount = await program.account.stakeAccount.fetch(
        user1StakeAccountPda
      );

      assert.strictEqual(
        userStakedAccount.stakedAmount.toNumber(),
        anchor.web3.LAMPORTS_PER_SOL
      );
      assert.isTrue(userStakedAccount.stakeTimestamp.toNumber() > 0);
    });

    it("Should stake additional amount", async () => {
      const additionalStake = new anchor.BN(0.5 * anchor.web3.LAMPORTS_PER_SOL);

      const tx = await program.methods
        .stake(additionalStake)
        .accounts({
          user: user1.publicKey,
        })
        .signers([user1])
        .rpc();

      const userStakedAccount = await program.account.stakeAccount.fetch(
        user1StakeAccountPda
      );

      assert.strictEqual(
        userStakedAccount.stakedAmount.toNumber(),
        1.5 * anchor.web3.LAMPORTS_PER_SOL
      );
    });

    it("Should fail to stake zero amount", async () => {
      try {
        await program.methods
          .stake(new anchor.BN(0))
          .accounts({
            user: user1.publicKey,
          })
          .signers([user1])
          .rpc();
        assert.fail("Should have failed with zero amount");
      } catch (error) {
        assert.include(error.toString(), "InvalidAmount");
      }
    });

    it("Should stake from different user", async () => {
      const stakeAmount = new anchor.BN(2 * anchor.web3.LAMPORTS_PER_SOL);

      const tx = await program.methods
        .stake(stakeAmount)
        .accounts({
          user: user2.publicKey,
        })
        .signers([user2])
        .rpc();

      const userStakedAccount = await program.account.stakeAccount.fetch(
        user2StakeAccountPda
      );

      assert.strictEqual(
        userStakedAccount.stakedAmount.toNumber(),
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
    });
  });

  describe("Unstaking Operations", () => {
    it("Should unstake partial amount", async () => {
      const unstakeAmount = new anchor.BN(0.5 * anchor.web3.LAMPORTS_PER_SOL);

      const tx = await program.methods
        .unstake(unstakeAmount)
        .accountsPartial({
          user: user1.publicKey,
        })
        .signers([user1])
        .rpc();

      const userStakedAccount = await program.account.stakeAccount.fetch(
        user1StakeAccountPda
      );

      assert.strictEqual(
        userStakedAccount.stakedAmount.toNumber(),
        1 * anchor.web3.LAMPORTS_PER_SOL
      );
    });

    it("Should fail to unstake more than staked", async () => {
      const excessiveAmount = new anchor.BN(10 * anchor.web3.LAMPORTS_PER_SOL);

      try {
        await program.methods
          .unstake(excessiveAmount)
          .accountsPartial({
            user: user1.publicKey,
          })
          .signers([user1])
          .rpc();
        assert.fail("Should have failed with insufficient stake");
      } catch (error) {
        assert.include(error.toString(), "InsufficientStake");
      }
    });

    it("Should fail to unstake zero amount", async () => {
      try {
        await program.methods
          .unstake(new anchor.BN(0))
          .accountsPartial({
            user: user1.publicKey,
          })
          .signers([user1])
          .rpc();
        assert.fail("Should have failed with zero amount");
      } catch (error) {
        assert.include(error.toString(), "InvalidAmount");
      }
    });

    it("Should unstake all remaining amount", async () => {
      const tx = await program.methods
        .unstake(new anchor.BN(anchor.web3.LAMPORTS_PER_SOL))
        .accountsPartial({
          user: user1.publicKey,
        })
        .signers([user1])
        .rpc();

      const userStakedAccount = await program.account.stakeAccount.fetch(
        user1StakeAccountPda
      );

      assert.strictEqual(userStakedAccount.stakedAmount.toNumber(), 0);
    });
  });

  describe("Points and Rewards System", () => {
    it("Should accumulate points over time", async () => {
      const stakeAmount = new anchor.BN(anchor.web3.LAMPORTS_PER_SOL);
      await program.methods
        .stake(stakeAmount)
        .accounts({
          user: user1.publicKey,
        })
        .signers([user1])
        .rpc();

      await new Promise((resolve) => setTimeout(resolve, 2000));

      const additionalStake = new anchor.BN(0.1 * anchor.web3.LAMPORTS_PER_SOL);
      await program.methods
        .stake(additionalStake)
        .accounts({
          user: user1.publicKey,
        })
        .signers([user1])
        .rpc();

      const userStakedAccount = await program.account.stakeAccount.fetch(
        user1StakeAccountPda
      );

      assert.isTrue(userStakedAccount.totalPoints.toNumber() > 0);
    });

    it("Should claim rewards when sufficient points", async () => {
      const tx = await provider.connection.requestAirdrop(
        user2.publicKey,
        1000 * anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(tx);
      const largeStake = new anchor.BN(100 * anchor.web3.LAMPORTS_PER_SOL);
      await program.methods
        .stake(largeStake)
        .accounts({
          user: user2.publicKey,
        })
        .signers([user2])
        .rpc();

      await new Promise((resolve) => setTimeout(resolve, 3000));

      await program.methods
        .stake(new anchor.BN(0.01 * anchor.web3.LAMPORTS_PER_SOL))
        .accounts({
          user: user2.publicKey,
        })
        .signers([user2])
        .rpc();

      const accountBefore = await program.account.stakeAccount.fetch(
        user2StakeAccountPda
      );

      if (accountBefore.totalPoints.toNumber() >= 10) {
        const initialTokenBalance = await spl.getAccount(
          provider.connection,
          user2TokenAccount
        );

        await program.methods
          .claimReward()
          .accounts({
            user: user2.publicKey,
            rewardMint: rewardMint,
            userTokenAccount: user2TokenAccount,
          })
          .signers([user2])
          .rpc();

        const finalTokenBalance = await spl.getAccount(
          provider.connection,
          user2TokenAccount
        );

        assert.isTrue(
          Number(finalTokenBalance.amount) > Number(initialTokenBalance.amount)
        );
      }
    });

    it("Should fail to claim rewards with insufficient points", async () => {
      const newUser = anchor.web3.Keypair.generate();
      const airdropTx = await provider.connection.requestAirdrop(
        newUser.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(airdropTx);

      await program.methods
        .createStakeAccount()
        .accounts({
          user: newUser.publicKey,
        })
        .signers([newUser])
        .rpc();

      await program.methods
        .stake(new anchor.BN(0.001 * anchor.web3.LAMPORTS_PER_SOL))
        .accounts({
          user: newUser.publicKey,
        })
        .signers([newUser])
        .rpc();

      const newUserTokenAccount = await spl.createAssociatedTokenAccount(
        provider.connection,
        newUser,
        rewardMint,
        newUser.publicKey
      );

      try {
        await program.methods
          .claimReward()
          .accounts({
            user: newUser.publicKey,
            rewardMint: rewardMint,
            userTokenAccount: newUserTokenAccount,
          })
          .signers([newUser])
          .rpc();
        assert.fail("Should have failed with insufficient points");
      } catch (error) {
        assert.include(error.toString(), "InsufficientPoints");
      }
    });
  });

  describe("Edge Cases and Security", () => {
    it("Should handle multiple stake/unstake cycles", async () => {
      const user = anchor.web3.Keypair.generate();
      const airdropTx = await provider.connection.requestAirdrop(
        user.publicKey,
        5 * anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(airdropTx);

      await program.methods
        .createStakeAccount()
        .accounts({
          user: user.publicKey,
        })
        .signers([user])
        .rpc();

      for (let i = 0; i < 3; i++) {
        await program.methods
          .stake(new anchor.BN(0.5 * anchor.web3.LAMPORTS_PER_SOL))
          .accounts({
            user: user.publicKey,
          })
          .signers([user])
          .rpc();

        await program.methods
          .unstake(new anchor.BN(0.3 * anchor.web3.LAMPORTS_PER_SOL))
          .accounts({
            user: user.publicKey,
          })
          .signers([user])
          .rpc();
      }

      const [userStakeAccountPda] =
        anchor.web3.PublicKey.findProgramAddressSync(
          [Buffer.from("staked_account"), user.publicKey.toBytes()],
          program.programId
        );

      const userStakedAccount = await program.account.stakeAccount.fetch(
        userStakeAccountPda
      );

      // should have accumulated 0.6 SOL (3 * 0.5 - 3 * 0.3)
      assert.strictEqual(
        userStakedAccount.stakedAmount.toNumber(),
        0.6 * anchor.web3.LAMPORTS_PER_SOL
      );
    });

    it("Should maintain separate accounts for different users", async () => {
      const account1 = await program.account.stakeAccount.fetch(
        user1StakeAccountPda
      );
      const account2 = await program.account.stakeAccount.fetch(
        user2StakeAccountPda
      );

      assert.notEqual(account1.owner.toBase58(), account2.owner.toBase58());
      assert.strictEqual(account1.owner.toBase58(), user1.publicKey.toBase58());
      assert.strictEqual(account2.owner.toBase58(), user2.publicKey.toBase58());
    });
  });
});