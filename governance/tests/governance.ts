import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Governance } from "../target/types/governance";
import { web3 } from "@coral-xyz/anchor";
import { assert } from "chai";
import { publicKey } from "@coral-xyz/anchor/dist/cjs/utils";

describe("Governance", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.governance as Program<Governance>;
  const provider = anchor.getProvider();

  let proposalPda: web3.PublicKey;
  let voterRecordPda: web3.PublicKey;
  let voterRecordBump: number;
  let proposalBump: number;

  const title = Array.from(Buffer.from("Test Proposal".padEnd(108, "\0")));
  const votesNeededToPass = new anchor.BN(1);
  const votingDuration = new anchor.BN(1);
  const proposalId = new anchor.BN(1);
  const creator = provider.wallet;
  const voter = web3.Keypair.generate();

  before(async () => {
    const tx1 = await provider.connection.requestAirdrop(
      voter.publicKey,
      web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(tx1);

    [proposalPda, proposalBump] = web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("proposal"),
        creator.publicKey.toBytes(),
        Buffer.from(proposalId.toArray("le", 8)),
      ],
      program.programId
    );

    [voterRecordPda, voterRecordBump] = web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("voter_record"),
        voter.publicKey.toBytes(),
        proposalPda.toBytes(),
      ],
      program.programId
    );
  });

  it("Create Proposal", async () => {
    try {
      const tx = await program.methods
        .createProposal(proposalId, title, votesNeededToPass, votingDuration)
        .accounts({
          creator: creator.publicKey,
        })
        .rpc();

      const proposal = await program.account.proposal.fetch(proposalPda);
      assert.strictEqual(
        proposal.creator.toBase58(),
        creator.publicKey.toBase58()
      );
      assert.strictEqual(proposal.votesNeededToPass.toNumber(), 1);
      assert.deepEqual(proposal.proposalStatus, { draft: {} });

      console.log("Create Proposal test passed!");
    } catch (error) {
      console.error("Full error details:", error);
      if (error.logs) {
        console.log("Program logs:", error.logs);
      }
      throw error;
    }
  });

  it("Start Voting", async () => {
    try {
      const tx = await program.methods
        .startVoting(proposalId)
        .accounts({
          creator: creator.publicKey,
        })
        .rpc();

      const proposal = await program.account.proposal.fetch(proposalPda);
      assert.deepEqual(proposal.proposalStatus, { voting: {} });
      assert.isTrue(proposal.votingStarted.toNumber() > 0);

      console.log("Start Voting test passed!");
    } catch (error) {
      console.error("Error in start voting:", error);
      throw error;
    }
  });

  it("Vote", async () => {
    try {
      const tx = await program.methods
        .vote(proposalId)
        .accounts({
          voter: voter.publicKey,
          creator: creator.publicKey,
        })
        .signers([voter])
        .rpc();

      const proposal = await program.account.proposal.fetch(proposalPda);
      assert.strictEqual(proposal.activeVotingCount.toNumber(), 1);

      const voter_record = await program.account.voterRecord.fetch(
        voterRecordPda
      );
      assert.strictEqual(
        voter_record.proposal.toBase58(),
        proposalPda.toBase58()
      );
      assert.equal(voter_record.voted, true);
      assert.strictEqual(
        voter_record.voter.toBase58(),
        voter.publicKey.toBase58()
      );
      assert.isTrue(voter_record.bump > 0);
    } catch (error) {
      console.error("Error while voting", error);
      throw error;
    }
  });

  it("Finalize Proposal", async () => {
    try {
      const tx = await program.methods
        .finalizeProposal(proposalId)
        .accounts({
          creator: creator.publicKey,
        })
        .rpc();

      const proposal = await program.account.proposal.fetch(proposalPda);
      console.log();
      assert.deepEqual(proposal.proposalStatus, { passed: {} });
    } catch (error) {
      console.error("Error", error);
      throw error;
    }
  });
});