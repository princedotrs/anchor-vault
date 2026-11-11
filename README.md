# anchor-vault: a secure SOL vault on Solana, built with Anchor

A minimal per-user SOL vault written in Rust with Anchor. Each user gets their own vault that only they can withdraw from. It's part of my **Solana from first principles** course: the code here is what we build on camera, line by line.

> **Watch the lessons:** [Part 1: Architecture & logic](https://youtu.be/rQZJF6PGn3M) · [Part 2: Anchor implementation](https://youtu.be/0FGxIg3DIkw) · [Part 3: Testing with Surfpool](https://youtu.be/l_1j12ev598)
>
> Full course: [Solana from first principles](https://www.youtube.com/playlist?list=PLRR3UvNYzm00)

By the end you'll understand:

- what a PDA (program derived address) is and why a program needs one to hold funds
- how to derive one PDA from another so every user gets their own vault
- how a program "signs" for a PDA with its seeds to move SOL out (CPI with signer seeds)
- how to close accounts and give the rent back to the user
- how to test it end to end against a local validator (Surfpool)

## What the program does

| Instruction | What it does |
| --- | --- |
| `initialize()` | Creates the user's vault state account and stores the PDA bumps |
| `deposit(amount)` | Moves `amount` lamports from the user into their vault |
| `withdraw(amount)` | Moves `amount` lamports from the vault back to the user, if the vault holds enough |
| `close()` | Sends everything left in the vault to the user and closes the state account (rent refunded) |

## Accounts

```
VaultState PDA   seeds = ["state", user]          stores vault_bump, state_bump, amount
 └─ Vault PDA    seeds = ["vault", vault_state]   a plain system account that holds the SOL
```

- **VaultState** is owned by the program and remembers the bumps and how much has been deposited.
- **Vault** is a system account with no data. Because its address is a PDA of this program, only this program can sign for it, so only the program can move SOL out.
- Both seeds start from the user's public key, so another wallet derives a different vault. Passing someone else's vault fails the `seeds` check, which is what keeps one user from withdrawing another's SOL.

## How SOL moves

**Deposit** is an ordinary transfer: the user signs, the System Program moves lamports into the vault.

**Withdraw and close** need the vault to sign, but a PDA has no private key. Instead the program passes the vault's seeds and bump to the System Program (`CpiContext::new_with_signer`), and the runtime accepts that as the vault's signature:

```rust
let seeds = &[b"vault", vault_state.key().as_ref(), &[vault_bump]];
let cpi_ctx = CpiContext::new_with_signer(system_program, transfer_accounts, &[&seeds[..]]);
transfer(cpi_ctx, amount)?;
```

`withdraw` also checks `amount <= vault_state.amount` first and fails with `InsufficientFunds` otherwise.

## Run it

Prerequisites: Rust, the Solana CLI, Anchor, Node.js + Yarn, and [Surfpool](https://surfpool.run).

```bash
git clone https://github.com/princedotrs/anchor-vault.git
cd anchor-vault
yarn install
anchor build

# terminal 1: local validator
surfpool start

# terminal 2: run the tests
anchor test --skip-local-validator
```

The test suite (`tests/anchor-vault.ts`) initializes a vault, deposits 2 SOL, withdraws 1 SOL, checks that withdrawing more than the balance fails, checks that a different wallet can't withdraw from this vault, and closes it.

## Project layout

```
programs/anchor-vault/src/
└── lib.rs                # instructions, account structs, VaultState, errors
tests/anchor-vault.ts     # end-to-end tests
```

## Ideas to extend it

- Store SPL tokens instead of SOL, using a token account owned by the vault PDA
- Add a time lock so withdrawals only work after a set slot or timestamp
- Let the user name a second wallet that can also withdraw

## License

MIT
