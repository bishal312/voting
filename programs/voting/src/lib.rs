use anchor_lang::prelude::*;

declare_id!("tHzPaMcD6RoBDU9uubRCV7x58uWpSDtfShiJ3S2ywgi");

#[program]
pub mod voting {
    use super::*;

    pub fn init_poll(ctx: Context<InitPoll>, _poll_id: u64, start: u64, end: u64, name: String, description: String,) -> Result<()> {
        let poll = &mut ctx.accounts.poll_account;
        poll.poll_name = name;
        poll.poll_description = description;
        poll.poll_voting_start = start;
        poll.poll_voting_end = end;
        poll.poll_option_index = 0;
        Ok(())
    }

    pub fn init_candidate(ctx: Context<InitCandidate>,
        _poll_id: u64,
        candidate: String,
    ) -> Result<()> {
        let candidate_account = &mut ctx.accounts.candidate_account;
        candidate_account.candidate_name = candidate;
        candidate_account.candidate_vote = 0;

        let poll_account = &mut ctx.accounts.poll_account;
        poll_account.poll_option_index += 1;
        Ok(())
    }

    pub fn vote(ctx: Context<Vote>, _poll_id: u64, _candidate: String) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let poll = &ctx.accounts.poll_account;

        if current_time < (poll.poll_voting_start as i64) {
            return Err(ErrorCode::VottingNotStarted.into());
        }
        
        if current_time > (poll.poll_voting_end as i64) {
            return Err(ErrorCode::VottingEnded.into());
        }
        
        let candidate = &mut ctx.accounts.candidate_account;
        candidate.candidate_vote += 1;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(_poll_id: u64)]
pub struct InitPoll<'info> { // Removed erroneous generic parameter
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        space = 8 + PollAccount::INIT_SPACE,
        seeds = [b"pool".as_ref(), _poll_id.to_le_bytes().as_ref()],
        bump
    )]
    pub poll_account: Account<'info, PollAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(_poll_id: u64, candidate: String)]
pub struct InitCandidate<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool".as_ref(), _poll_id.to_le_bytes().as_ref()],
        bump
    )]
    pub poll_account: Account<'info, PollAccount>,

    #[account(
        init,
        payer = signer,
        space = 8 + CandidateAccount::INIT_SPACE,
        seeds = [_poll_id.to_le_bytes().as_ref(), candidate.as_ref()],
        bump
    )]
    pub candidate_account: Account<'info, CandidateAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(_poll_id: u64, _candidate: String)]
pub struct Vote<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool".as_ref(), _poll_id.to_le_bytes().as_ref()], // Fixed: changed poll_id to _poll_id
        bump
    )]
    pub poll_account: Account<'info, PollAccount>,
    #[account(
        mut,
        seeds = [_poll_id.to_le_bytes().as_ref(), _candidate.as_ref()],
        bump
    )]
    pub candidate_account: Account<'info, CandidateAccount>,
}

#[account]
#[derive(InitSpace)]
pub struct PollAccount{
    #[max_len(32)]
    pub poll_name: String,
    #[max_len(280)]
    pub poll_description: String,
    pub poll_voting_start: u64,
    pub poll_voting_end: u64,
    pub poll_option_index: u64,
}

#[account]
#[derive(InitSpace)]
pub struct CandidateAccount {
    #[max_len(32)]
    pub candidate_name: String,
    pub candidate_vote: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Voting has not started yet")]
    VottingNotStarted,
    #[msg("Voting has ended")]
    VottingEnded,
}