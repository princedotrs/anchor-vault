use anchor_lang::prelude::*;

declare_id!("6FucFnpGdc84xS6rHEDh4V3oCwMLW3qdThMRoQuvgkmp");

#[program]
pub mod anchor_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
