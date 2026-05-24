import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Aeroscan } from "../target/types/aeroscan";

const HELIUS_RPC_URL = "https://devnet.helius-rpc.com/?api-key=<YOUR_API_KEY>";
const HELIUS_WS_URL = "wss://devnet.helius-rpc.com/?api-key=<YOUR_API_KEY>";

describe("aeroscan", () => {
  const connection = new anchor.web3.Connection(HELIUS_RPC_URL, {
    wsEndpoint: HELIUS_WS_URL,
    commitment: "confirmed",
  });

  const wallet = anchor.Wallet.local();

  const provider = new anchor.AnchorProvider(connection, wallet, {
    preflightCommitment: "confirmed",
  });

  anchor.setProvider(provider);

  const program = anchor.workspace.aeroscan as Program<Aeroscan>;

  const providerEphemeralRollup = new anchor.AnchorProvider(
    new anchor.web3.Connection(
      "https://devnet.magicblock.app/",
      {
        wsEndpoint: "wss://devnet.magicblock.app/",
      }
    ),
    anchor.Wallet.local()
  );

  const [sensor_reading] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("sensor_reading"), provider.wallet.publicKey.toBuffer()],
    program.programId
  );
  console.log("Program ID: ", program.programId.toString())
  console.log("position PDA: ", sensor_reading.toString())

  it("Initialize!", async () => {
    const start = Date.now();
    const txHash = await program.methods
      .initialize(
        0,
        0,
        0,
        0,
        0
      )
      .accountsPartial({
        sensorReading: sensor_reading,
        user: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc({ skipPreflight: true });
    const duration = Date.now() - start;
    console.log(`${duration}ms (Base Layer) Initialize txHash: ${txHash}`);
  });

  it.only("Update reading on Solana", async () => {
    const start = Date.now();
    const pm25 = 0;
    const pm10 = 0;
    const temperature = 0;
    const humidity = 0;
    const aqi = 0;

    const txHash = await program.methods
      .updateReading(provider.wallet.publicKey, pm25, pm10, temperature, humidity, aqi)
      .accountsPartial({
        sensorReading: sensor_reading,
      })
      .rpc();
    const duration = Date.now() - start;
    console.log(`${duration}ms (Base Layer) Increment txHash: ${txHash}`);
  });

  it("Delegate reading to ER", async () => {
    const start = Date.now();
    let tx = await program.methods
      .delegate()
      .accounts({
        user: provider.wallet.publicKey,
        sensorReading: sensor_reading,
      })
      .transaction();
    tx.feePayer = provider.wallet.publicKey;
    tx.recentBlockhash = (
      await provider.connection.getLatestBlockhash()
    ).blockhash;
    tx = await providerEphemeralRollup.wallet.signTransaction(tx);
    const txHash = await provider.sendAndConfirm(tx, [], {
      skipPreflight: true,
      commitment: "confirmed",
    });
    const duration = Date.now() - start;
    console.log(`${duration}ms (Base Layer) Delegate txHash: ${txHash}`);
  });

  it("Update reading on ER", async () => {
    try {
      const start = Date.now();
      const pm25 = 0;
      const pm10 = 0;
      const temperature = 0;
      const humidity = 0;
      const aqi = 0;

      let tx = await program.methods
        .updateReading(provider.wallet.publicKey, pm25, pm10, temperature, humidity, aqi)
        .accountsPartial({
          sensorReading: sensor_reading,
        })
        .signers([])
        .transaction();

      tx.feePayer = providerEphemeralRollup.wallet.publicKey;
      tx.recentBlockhash = (
        await providerEphemeralRollup.connection.getLatestBlockhash()
      ).blockhash;
      tx = await providerEphemeralRollup.wallet.signTransaction(tx);
      const txHash = await providerEphemeralRollup.sendAndConfirm(tx);
      const duration = Date.now() - start;
      console.log(`${duration}ms (ER) Increment txHash: ${txHash}`);
    } catch (error) {
      console.log(error)
    }
  });

  it("Undelegate reading on ER to Solana", async () => {
    const start = Date.now();
    let tx = await program.methods
      .undelegate()
      .accountsPartial({
        user: providerEphemeralRollup.wallet.publicKey,
        sensorReading: sensor_reading,
      })
      .transaction();
    tx.feePayer = provider.wallet.publicKey;
    tx.recentBlockhash = (
      await providerEphemeralRollup.connection.getLatestBlockhash()
    ).blockhash;
    tx = await providerEphemeralRollup.wallet.signTransaction(tx);

    const txHash = await providerEphemeralRollup.sendAndConfirm(tx);
    const duration = Date.now() - start;
    console.log(`${duration}ms (ER) Undelegate txHash: ${txHash}`);
  });
});
