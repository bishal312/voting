use anchor_lang::prelude::*;

declare_id!("tHzPaMcD6RoBDU9uubRCV7x58uWpSDtfShiJ3S2ywgi");

#[program]
pub mod voting {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        crate::instructions::initialize::handle_initialize(ctx)
    }

}

#[account]
#[derive(InitSpace)]
pub struct PollAccount{
    #[max_len(32)]
    pub poll_name: String,
}