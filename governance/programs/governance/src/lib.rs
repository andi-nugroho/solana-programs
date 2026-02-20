use anchor_lang::prelude::*;

declare_id!("nWhEAJUqEqBiBzo8BKVfRK9aYMFCjLkjy9rhP2C6hia");

#[program]
pub mod governance {
    use super::*;

    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        proposal_id: u64,
        title: [u8; 108],
        votes_needed_to_pass: u64,
        voting_duration: i64,
    ) -> Result<()> {
        let proposal_account = &mut ctx.accounts.proposal;

        if core::str::from_utf8(&title).is_err() {
            return Err(error!(CustomError::InvalidUtf8));
        }
        proposal_account.title = title;
        proposal_account.creator = ctx.accounts.creator.key();
        proposal_account.votes_needed_to_pass = votes_needed_to_pass;
        proposal_account.voting_duration = voting_duration;
        proposal_account.proposal_status = ProposalStatus::Draft;
        proposal_account.voting_started = 0;
        proposal_account.voting_ended = 0;
        proposal_account.active_voting_count = 0;
        proposal_account.bump = ctx.bumps.proposal;

        Ok(())
    }

    pub fn start_voting(ctx: Context<StartVoting>, proposal_id: u64) -> Result<()> {
        let clock = &ctx.accounts.clock;
        let proposal = &mut ctx.accounts.proposal;

        proposal.proposal_status = ProposalStatus::Voting;
        proposal.voting_started = clock.unix_timestamp;
        proposal.voting_ended = clock.unix_timestamp + proposal.voting_duration;
        proposal.active_voting_count = 0;

        Ok(())
    }

    pub fn vote(ctx: Context<Vote>, proposal_id: u64) -> Result<()> {
        let clock = &ctx.accounts.clock;
        let proposal = &mut ctx.accounts.proposal;
        let voter_record = &mut ctx.accounts.voter_record;

        if proposal.proposal_status != ProposalStatus::Voting {
            return Err(error!(CustomError::VotingNotStarted));
        }

        if clock.unix_timestamp > proposal.voting_ended {
            return Err(error!(CustomError::VotingExpired));
        }

        if proposal.active_voting_count >= proposal.votes_needed_to_pass {
            return Err(error!(CustomError::VotingLimitReached));
        }

        if voter_record.voted {
            return Err(error!(CustomError::AlreadyVoted));
        }

        voter_record.voter = ctx.accounts.voter.key();
        voter_record.proposal = proposal.key();
        voter_record.voted = true;
        voter_record.bump = ctx.bumps.voter_record;

        proposal.active_voting_count += 1;

        Ok(())
    }

    pub fn finalize_proposal(ctx: Context<FinalizeProposal>, proposal_id: u64) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        let clock = &ctx.accounts.clock;

        if proposal.proposal_status != ProposalStatus::Voting {
            return Err(error!(CustomError::VotingNotStarted));
        }

        if clock.unix_timestamp < proposal.voting_ended {
            return Err(error!(CustomError::VotingNotFinished));
        }

        if proposal.active_voting_count >= proposal.votes_needed_to_pass {
            proposal.proposal_status = ProposalStatus::Passed;
        } else {
            proposal.proposal_status = ProposalStatus::Failed;
        }

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        init,
        payer = creator,
        space = 8 + std::mem::size_of::<Proposal>(),
        seeds = [b"proposal", creator.key().as_ref(), &proposal_id.to_le_bytes()],
        bump
    )]
    pub proposal: Account<'info, Proposal>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct StartVoting<'info> {
    #[account()]
    pub creator: Signer<'info>,

    #[account(
        mut,
        seeds = [b"proposal", creator.key().as_ref(), &proposal_id.to_le_bytes()],
        bump = proposal.bump,
        constraint = proposal.creator == creator.key()
    )]
    pub proposal: Account<'info, Proposal>,

    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct Vote<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,

    #[account(
        init,
        payer = voter,
        space = 8 + std::mem::size_of::<VoterRecord>(),
        seeds = [b"voter_record", voter.key().as_ref(), proposal.key().as_ref()],
        bump
    )]
    pub voter_record: Account<'info, VoterRecord>,

    #[account()]
    pub creator: Signer<'info>,

    #[account(
        mut,
        seeds = [b"proposal", creator.key().as_ref(), &proposal_id.to_le_bytes()],
        bump = proposal.bump,
        constraint = proposal.creator == creator.key()
    )]
    pub proposal: Account<'info, Proposal>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct FinalizeProposal<'info> {
    /// CHECK: This is the creator of the proposal, used only for constraint checking
    #[account()]
    pub creator: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"proposal", creator.key().as_ref(), &proposal_id.to_le_bytes()],
        bump = proposal.bump,
        constraint = proposal.creator == creator.key()
    )]
    pub proposal: Account<'info, Proposal>,

    pub clock: Sysvar<'info, Clock>,
}

#[account]
pub struct Proposal {
    pub title: [u8; 108],
    pub creator: Pubkey,
    pub voting_started: i64,
    pub voting_ended: i64,
    pub voting_duration: i64,
    pub active_voting_count: u64,
    pub votes_needed_to_pass: u64,
    pub proposal_status: ProposalStatus,
    pub bump: u8,
}

#[account]
pub struct VoterRecord {
    pub voter: Pubkey,
    pub proposal: Pubkey,
    pub voted: bool,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum ProposalStatus {
    Draft,
    Voting,
    Passed,
    Failed,
}

#[error_code]
pub enum CustomError {
    #[msg("The provided title is not valid UTF-8.")]
    InvalidUtf8,

    #[msg("Voting limit has been reached.")]
    VotingLimitReached,

    #[msg("You have already voted.")]
    AlreadyVoted,

    #[msg("Voting hasn't started yet.")]
    VotingNotStarted,

    #[msg("Voting duration has expired.")]
    VotingExpired,

    #[msg("Voting is not yet finished.")]
    VotingNotFinished,
}