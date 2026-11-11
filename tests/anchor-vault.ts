import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorVault } from "../target/types/anchor_vault";
import { assert } from "chai";

describe("anchor-vault", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.anchorVault as Program<AnchorVault>;

  const provider = anchor.getProvider();

  // [pda_pubkey, bump]

  // Derive PDAs
  const vaultState = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("state"), provider.publicKey.toBytes()],
    program.programId
  )[0];

  const vault = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), vaultState.toBytes()],
    program.programId
  )[0];

  it("Is initialized!", async () => {
    const balance = await provider.connection.getBalance(provider.publicKey);
    console.log("Balance is:", balance / anchor.web3.LAMPORTS_PER_SOL);

    const tx = await program.methods
      .initialize()
      .accountsPartial({
        user: provider.wallet.publicKey,
        vaultState,
        vault,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([provider.wallet.payer])
      .rpc();
    console.log("\nYour transaction signature", tx);
  });

  it("Deposit 2 SOL", async () => {
    const tx = await program.methods
      .deposit(new anchor.BN(2 * anchor.web3.LAMPORTS_PER_SOL))
      .accountsPartial({
        user: provider.wallet.publicKey,
        vaultState,
        vault,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("\nYour transaction signature", tx);
    console.log("Vault info:", await provider.connection.getAccountInfo(vault));
    console.log("Vault Balance:", await provider.connection.getBalance(vault));
  });

  it("Withdraw 1 SOL", async () => {
    const tx = await program.methods
      .withdraw(new anchor.BN(1 * anchor.web3.LAMPORTS_PER_SOL))
      .accountsPartial({
        user: provider.wallet.publicKey,
        vaultState,
        vault,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("\nYour transaction signature", tx);
    console.log("Vault info:", await provider.connection.getAccountInfo(vault));
    console.log("Vault Balance:", await provider.connection.getBalance(vault));
  });

  it("Withdraw more than deposited -> should fail", async () => {
    try {
      await program.methods
        .withdraw(new anchor.BN(5 * anchor.web3.LAMPORTS_PER_SOL))
        .accountsPartial({
          user: provider.wallet.publicKey,
          vaultState,
          vault,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
      assert.fail("withdraw should have been rejected");
    } catch (e) {
      assert.include(String(e), "InsufficientFunds");
    }
  });

  it("Withdraw 1 SOL with wrong owner -> should fail", async () => {
    // A different user derives a different vault_state PDA, so passing
    // someone else's vault must fail the seeds check.
    const user = anchor.web3.Keypair.generate();

    const sig = await provider.connection.requestAirdrop(
      user.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(sig, "confirmed");

    try {
      await program.methods
        .withdraw(new anchor.BN(1 * anchor.web3.LAMPORTS_PER_SOL))
        .accountsPartial({
          user: user.publicKey,
          vaultState,
          vault,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([user])
        .rpc();
      assert.fail("withdraw by a different user should have been rejected");
    } catch (e) {
      assert.include(String(e), "ConstraintSeeds");
    }
  });

  it("Close Vault", async () => {
    const tx = await program.methods
      .close()
      .accountsPartial({
        user: provider.wallet.publicKey,
        vaultState,
        vault,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("\nYour transaction signature", tx);
    console.log("Vault Balance:", await provider.connection.getBalance(vault));
  });


});
