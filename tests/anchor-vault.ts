import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorVault } from "../target/types/anchor_vault";

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

  it("Withdraw 1 SOL with wrong owner -> should fail", async () => {
    let user = anchor.web3.Keypair.generate();
    const user_pubkey = user.publicKey;

    console.log("user :", user_pubkey.toBase58());

    const airdrop_tx = await provider.connection.requestAirdrop(
      user_pubkey, 2* anchor.web3.LAMPORTS_PER_SOL
    );
    
    console.log("airdrop tx :", airdrop_tx);

    const tx = await program.methods
      .withdraw(new anchor.BN(1 * anchor.web3.LAMPORTS_PER_SOL))
      .accountsPartial({
        user: user_pubkey,
        vaultState,
        vault,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    console.log("\nYour transaction signature", tx);
    console.log("Vault info:", await provider.connection.getAccountInfo(vault));
    console.log("Vault Balance:", await provider.connection.getBalance(vault));
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
