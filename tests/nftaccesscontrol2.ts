import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Nftaccesscontrol2 } from "../target/types/nftaccesscontrol2";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  getAssociatedTokenAddress,
  getAccount,
} from "@solana/spl-token";
import { assert } from "chai";

describe("nftaccesscontrol2", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Nftaccesscontrol2 as Program<Nftaccesscontrol2>;
  
  const authority = anchor.web3.Keypair.generate();
  const subscriber = anchor.web3.Keypair.generate();
  let licenseMint: anchor.web3.PublicKey;
  let subscriberTokenAccount: anchor.web3.PublicKey;
  let publisherPDA: anchor.web3.PublicKey;
  
  const licensePrice = new anchor.BN(1_000_000_000); 

  before(async () => {
    const connection = anchor.getProvider().connection;
    
    const authorityAirdrop = await connection.requestAirdrop(
      authority.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(authorityAirdrop);

    const subscriberAirdrop = await connection.requestAirdrop(
      subscriber.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(subscriberAirdrop);

    licenseMint = await createMint(
      connection,
      authority,
      authority.publicKey,
      null,
      0
    );

    [publisherPDA] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("publisher"), authority.publicKey.toBuffer()],
      program.programId
    );

    subscriberTokenAccount = await getAssociatedTokenAddress(
      licenseMint,
      subscriber.publicKey
    );
  });

  it("Initializes the publisher", async () => {
    await program.methods
      .initialize(licensePrice)
      .accounts({
        publisher: publisherPDA,
        licenseMint: licenseMint,
        authority: authority.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([authority])
      .rpc();

    const publisherAccount = await program.account.publisher.fetch(publisherPDA);
    assert.equal(publisherAccount.authority.toString(), authority.publicKey.toString());
    assert.equal(publisherAccount.licenseMint.toString(), licenseMint.toString());
    assert.equal(publisherAccount.licensePrice.toString(), licensePrice.toString());
    assert.equal(publisherAccount.totalSubscribers.toString(), "0");
  });

  it("Allows purchasing a license", async () => {
    const preBalance = await anchor.getProvider().connection.getBalance(subscriber.publicKey);

    await program.methods
      .purchaseLicense()
      .accounts({
        publisher: publisherPDA,
        licenseMint: licenseMint,
        authority: authority.publicKey,
        subscriber: subscriber.publicKey,
        subscriberTokenAccount: subscriberTokenAccount,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([subscriber])
      .rpc();

    const postBalance = await anchor.getProvider().connection.getBalance(subscriber.publicKey);
    assert(preBalance - postBalance >= licensePrice.toNumber());

    const tokenAccount = await getAccount(
      anchor.getProvider().connection,
      subscriberTokenAccount
    );
    assert.equal(tokenAccount.amount.toString(), "1");

    // Verify subscriber count increased
    const publisherAccount = await program.account.publisher.fetch(publisherPDA);
    assert.equal(publisherAccount.totalSubscribers.toString(), "1");
  });

  it("Can verify access", async () => {
    const result = await program.methods
      .verifyAccess()
      .accounts({
        publisher: publisherPDA,
        authority: authority.publicKey,
        subscriber: subscriber.publicKey,
        subscriberTokenAccount: subscriberTokenAccount,
        licenseMint: licenseMint,
      })
      .view();

    assert.isTrue(result);
  });

  it("Allows revoking a license", async () => {
    await program.methods
      .revokeLicense()
      .accounts({
        publisher: publisherPDA,
        licenseMint: licenseMint,
        authority: authority.publicKey,
        subscriber: subscriber.publicKey,
        subscriberTokenAccount: subscriberTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([authority])
      .rpc();

    const tokenAccount = await getAccount(
      anchor.getProvider().connection,
      subscriberTokenAccount
    );
    assert.equal(tokenAccount.amount.toString(), "0");

    const publisherAccount = await program.account.publisher.fetch(publisherPDA);
    assert.equal(publisherAccount.totalSubscribers.toString(), "0");
  });

  it("Verifies access is revoked", async () => {
    const result = await program.methods
      .verifyAccess()
      .accounts({
        publisher: publisherPDA,
        authority: authority.publicKey,
        subscriber: subscriber.publicKey,
        subscriberTokenAccount: subscriberTokenAccount,
        licenseMint: licenseMint,
      })
      .view();

    assert.isFalse(result);
  });

  it("Fails when non-authority tries to revoke license", async () => {
    try {
      await program.methods
        .revokeLicense()
        .accounts({
          publisher: publisherPDA,
          licenseMint: licenseMint,
          authority: subscriber.publicKey,
          subscriber: subscriber.publicKey,
          subscriberTokenAccount: subscriberTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([subscriber])
        .rpc();
      
      assert.fail("Expected the transaction to fail");
    } catch (error) {
      assert.include(error.toString(), "Error");
    }
  });
});