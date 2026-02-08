use anchor_lang::prelude::*;
use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

declare_id!("6FucFnpGdc84xS6rHEDh4V3oCwMLW3qdThMRoQuvgkmp");

#[program]
pub mod anchor_vault {
    use crate::instruction::{Close, Deposit};

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)?;
        Ok(())
    }

    pub fn deposit(ctx: Context<Payment>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)?;
        Ok(())
    }
    pub fn withdraw(ctx: Context<Payment>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)?;
        Ok(())
    }
    pub fn close(ctx: Context<CloseAccount>) -> Result<()> {
        ctx.accounts.close()?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>, // The user who signs the transaction to create the vault.

    #[account(
        init, // Creates a new account for the vault state.
        payer = user, // User pays the rent for this account.
        seeds = [b"state", user.key().as_ref()], // PDA seeds for uniqueness.
        bump, // Bump seed for PDA derivation.
        space = VaultState::INIT_SPACE, // Space allocated for the vault state data.
    )]
    pub vault_state: Account<'info, VaultState>, // Stores vault configuration.

    #[account(
        seeds = [b"vault", vault_state.key().as_ref()], // PDA seeds for the vault.
        bump, // Bump seed for the vault PDA.
    )]
    pub vault: SystemAccount<'info>, // Holds the SOL deposited into the vault.

    pub system_program: Program<'info, System>, // Required for account creation and transfers.
}

// Methods for the Initialize struct.
impl<'info> Initialize<'info> {
    // Sets up the vault state with bump seeds for PDAs.
    pub fn initialize(&mut self, bumps: &InitializeBumps) -> Result<()> {
        // self.vault_state.vault_bump = bumps.vault; // Stores vault PDA bump.
        // self.vault_state.state_bump = bumps.vault_state; // Stores state PDA bump.
        // self.vault_state.amount = 0; // set the amount as 0 for now

        self.vault_state.set_inner(VaultState {
            vault_bump: bumps.vault,
            state_bump: bumps.vault_state,
            amount: 0,
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Payment<'info> {
    #[account(mut)]
    pub user: Signer<'info>, // The user performing the deposit or withdrawal.

    #[account(
        mut,
        seeds = [b"state", user.key().as_ref()], // Matches the vault state PDA.
        bump = vault_state.state_bump // Ensures correct PDA with stored bump.
    )]
    pub vault_state: Account<'info, VaultState>, // Accesses vault configuration.

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()], // Matches the vault PDA.
        bump = vault_state.vault_bump // Ensures correct PDA with stored bump.
    )]
    pub vault: SystemAccount<'info>, // The account holding the SOL.

    pub system_program: Program<'info, System>, // Used for SOL transfers.
}

impl<'info> Payment<'info> {
    // Deposits SOL from the user to the vault using a CPI (Cross-Program Invocation).
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.system_program.to_account_info(); // System program for transfer.
        let cpi_accounts = Transfer {
            from: self.user.to_account_info(), // User's account as the source.
            to: self.vault.to_account_info(),  // Vault as the destination.
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts); // Context for the transfer.
        transfer(cpi_ctx, amount)?; // Executes the SOL transfer.

        self.vault_state.amount = amount;

        Ok(())
    }

    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.system_program.to_account_info(); // System program for transfer.
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(), // Vault as the source.
            to: self.user.to_account_info(),    // User's account as the destination.
        };
        // Seeds for PDA signing authority.
        let seeds = &[
            b"vault",
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump],
        ];
        let signer_seeds = &[&seeds[..]]; // Array of seeds for signing.
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds); // Signed context.
        transfer(cpi_ctx, amount)?; // Executes the SOL transfer.

        self.vault_state.amount -= amount;

        Ok(())
    }
}

// Defines the accounts required to close the vault.
#[derive(Accounts)]
pub struct CloseAccount<'info> {
    #[account(mut)]
    pub user: Signer<'info>, // The user closing the vault.

    #[account(
        mut,
        seeds = [b"state", user.key().as_ref()], // Matches the vault state PDA.
        bump = vault_state.state_bump, // Ensures correct PDA.
        close = user // Closes the state account, refunding rent to the user.
    )]
    pub vault_state: Account<'info, VaultState>, // Vault configuration to be closed.

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()], // Matches the vault PDA.
        bump = vault_state.vault_bump, // Ensures correct PDA.
    )]
    pub vault: SystemAccount<'info>, // Vault account to be drained.

    pub system_program: Program<'info, System>, // Required for transfers and closure.
}

// Methods for the CloseAccount struct.
impl<'info> CloseAccount<'info> {
    // Transfers all SOL from the vault to the user and closes the account.
    pub fn close(&mut self) -> Result<()> {
        let cpi_program = self.system_program.to_account_info(); // System program for transfer.
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(), // Vault as the source.
            to: self.user.to_account_info(),    // User's account as the destination.
        };
        // Seeds for PDA signing authority.
        let seeds = &[
            b"vault",
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump],
        ];
        let signer_seeds = &[&seeds[..]]; // Array of seeds for signing.
        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds); // Signed context.
        transfer(cpi_context, self.vault.lamports()) // Transfers all SOL, effectively closing the vault.
    }
}

// #[derive(Accounts)]
// pub struct Withdraw<'info> {
//     #[account(mut)]
//     pub user: Signer<'info>, // The user performing the deposit or withdrawal.

//     #[account(
//         mut,
//         seeds = [b"state", user.key().as_ref()], // Matches the vault state PDA.
//         bump = vault_state.state_bump // Ensures correct PDA with stored bump.
//     )]
//     pub vault_state: Account<'info, VaultState>, // Accesses vault configuration.

//     #[account(
//         mut,
//         seeds = [b"vault", vault_state.key().as_ref()], // Matches the vault PDA.
//         bump = vault_state.vault_bump // Ensures correct PDA with stored bump.
//     )]
//     pub vault: SystemAccount<'info>, // The account holding the SOL.

//     pub system_program: Program<'info, System> // Used for SOL transfers.
// }

// impl<'info> Withdraw<'info> {
//     // Withdraws SOL from the vault to the user, signed by the vault PDA.
//     pub fn withdraw(&mut self, amount: u64) -> Result<()> {
//         let cpi_program = self.system_program.to_account_info(); // System program for transfer.
//         let cpi_accounts = Transfer {
//             from: self.vault.to_account_info(), // Vault as the source.
//             to: self.user.to_account_info(), // User's account as the destination.
//         };
//         // Seeds for PDA signing authority.
//         let seeds = &[
//             b"vault",
//             self.vault_state.to_account_info().key.as_ref(),
//             &[self.vault_state.vault_bump],
//         ];
//         let signer_seeds = &[&seeds[..]]; // Array of seeds for signing.
//         let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds); // Signed context.
//         transfer(cpi_ctx, amount)?; // Executes the SOL transfer.

//         self.vault_state.amount -= amount;

//         Ok(())
//     }
// }

#[account]
// #[derive(InitSpace)]
pub struct VaultState {
    pub vault_bump: u8, // Bump for the vault PDA.
    pub state_bump: u8, // Bump for the vault state PDA.
    pub amount: u64,    // amount to be deposited in the actual vault
}

impl Space for VaultState {
    const INIT_SPACE: usize = 8 + 1 + 1 + 8; // 8 bytes for discriminator + 1 for vault_bump + 1 for state_bump + 8 for amount
}
