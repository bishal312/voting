// setup and program decleration

use anchor_lang::declare_program;
use anchor_litesvm::{AnchorContext, AnchorLiteSVM, AssertionHelpers, Pubkey, Signer, TestHelpers};

declare_program!(voting); // Automatically generates client bindings, instruction builders, and account structs based on your compiled Anchor program so your Rust tests can talk to it seamlessly.

use self::voting::accounts::{CandidateAccount, PollAccount};
use self::voting::client::{accounts, args};

const PROGRAM_BYTES: &[u8] = include_bytes!("../../../target/deploy/voting.so");


// Environment setup & helpers
// setup() Initializes a lightweight Solana Virtual Machine (LiteSVM) instance loaded with your program.
fn setup() -> AnchorContext {
    use anchor_lang::solana_program::clock::Clock;

    let mut ctx = AnchorLiteSVM::build_with_program(self::voting::ID, PROGRAM_BYTES);

    let clock = Clock { // mock clock 
        slot: 1000,
        epoch_start_timestamp: 0,
        epoch: 1,
        leader_schedule_epoch: 1,
        unix_timestamp: 1000,
    };

    ctx.svm.set_sysvar(&clock);
    ctx
}

//Address Generation (PDAs)

fn get_poll_pda(_poll_id: u64) -> Pubkey {
    Pubkey::find_program_address(&[b"pool", &_poll_id.to_le_bytes()], &self::voting::ID).0
}

fn get_candidate_pda(_poll_id: u64, candidate: &str) -> Pubkey {
    Pubkey::find_program_address(&[_poll_id.to_le_bytes().as_ref(), candidate.as_bytes()], &self::voting::ID).0
}

//Instruction wrapper helpers

fn initialize_poll(  //Builds the InitPoll transaction instruction, passes arguments (_poll_id, start, end, name, description), executes it with the signer, and asserts success.
    ctx: &mut AnchorContext,
    signer: &anchor_litesvm::Keypair,
    _poll_id: u64,
    start_time: u64,
    end_time: u64,
    name: &str,
    description: &str,
) {
    let poll_pda = get_poll_pda(_poll_id);

    let ix = ctx
        .program()
        .accounts(accounts::InitPoll {
            signer: signer.pubkey(),
            poll_account: poll_pda,
            system_program: anchor_lang::system_program::ID,
        })
        .args(args::InitPoll {
            _poll_id,
            start: start_time,
            end: end_time,
            name: name.to_string(),
            description: description.to_string(),
        })
        .instruction()
        .unwrap();

    let result = ctx.execute_instruction(ix, &[signer]).unwrap();
    result.assert_success();
    ctx.svm.assert_account_exists(&poll_pda);
}

fn initialize_candidate( //Builds the InitCandidate instruction to register a candidate (e.g., "Alice" or "Bob") under a specific poll.
    ctx: &mut AnchorContext,
    signer: &anchor_litesvm::Keypair,
    _poll_id: u64,
    candidate: &str,
) {
    let ix = ctx
        .program()
        .accounts(accounts::InitCandidate {
            signer: signer.pubkey(),
            poll_account: get_poll_pda(_poll_id),
            candidate_account: get_candidate_pda(_poll_id, candidate),
            system_program: anchor_lang::system_program::ID,
        })
        .args(args::InitCandidate {
            _poll_id,
            candidate: candidate.to_string(),
        })
        .instruction()
        .unwrap();

    let result = ctx.execute_instruction(ix, &[signer]).unwrap();
    result.assert_success();
}

fn cast_vote( // Simulates a user casting a vote for a candidate and returns the transaction result so tests can check for success or failure.
    ctx: &mut AnchorContext,
    signer: &anchor_litesvm::Keypair,
    _poll_id: u64,
    candidate: &str,
) -> anchor_litesvm::TransactionResult {
    let ix = ctx
        .program()
        .accounts(accounts::Vote {
            signer: signer.pubkey(),
            poll_account: get_poll_pda(_poll_id),
            candidate_account: get_candidate_pda(_poll_id, candidate),
        })
        .args(args::Vote {
            _poll_id,
            _candidate: candidate.to_string(),
        })
        .instruction()
        .unwrap();

    ctx.execute_instruction(ix, &[signer]).unwrap()
}

// Unit Tests Explained

#[test]
fn test_init_poll() { // it creates a new poll with an id of 1 and a lifespan of 0 to u64::MAX.
    let mut ctx = setup();
    let user = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    let _poll_id = 1;
    let poll_pda = get_poll_pda(_poll_id);
    let start_time = 0;
    let end_time = u64::MAX;
    let poll_name = "Test Poll";
    let poll_description = "A test poll for voting";

    initialize_poll(
        &mut ctx,
        &user,
        _poll_id,
        start_time,
        end_time,
        poll_name,
        poll_description,
    );

    let poll_account: PollAccount = ctx.get_account(&poll_pda).unwrap();
    assert_eq!(poll_account.poll_name, poll_name);
    assert_eq!(poll_account.poll_description, poll_description);
    assert_eq!(poll_account.poll_voting_start, start_time);
    assert_eq!(poll_account.poll_voting_end, end_time);
    assert_eq!(poll_account.poll_option_index, 0);
}

#[test]
fn test_init_candidates() { //Initializes a poll, then registers two candidates ("Alice" and "Bob").
    let mut ctx = setup();
    let user = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    let _poll_id = 1;
    initialize_poll(
        &mut ctx,
        &user,
        _poll_id,
        0,
        u64::MAX,
        "Test Poll",
        "A test poll for voting",
    );
    initialize_candidate(&mut ctx, &user, _poll_id, "Alice");
    initialize_candidate(&mut ctx, &user, _poll_id, "Bob");

    let poll_account: PollAccount = ctx.get_account(&get_poll_pda(_poll_id)).unwrap();
    let alice_account: CandidateAccount = ctx.get_account(&get_candidate_pda(_poll_id, "Alice")).unwrap();
    let bob_account: CandidateAccount = ctx.get_account(&get_candidate_pda(_poll_id, "Bob")).unwrap();

    assert_eq!(poll_account.poll_option_index, 2);
    assert_eq!(alice_account.candidate_name, "Alice");
    assert_eq!(alice_account.candidate_vote, 0);
    assert_eq!(bob_account.candidate_name, "Bob");
    assert_eq!(bob_account.candidate_vote, 0);
}

#[test]
fn test_vote() { //Sets up a poll where voting is open (timestamp 1000 falls between 0 and 2000), registers "Alice", and casts a vote from a separate voter account.
    let mut ctx = setup();
    let authority = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let voter = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    let _poll_id = 1;
    initialize_poll(
        &mut ctx,
        &authority,
        _poll_id,
        0,
        2_000,
        "Test Poll",
        "A test poll for voting",
    );
    initialize_candidate(&mut ctx, &authority, _poll_id, "Alice");

    let result = cast_vote(&mut ctx, &voter, _poll_id, "Alice");
    result.assert_success();

    let alice_account: CandidateAccount = ctx.get_account(&get_candidate_pda(_poll_id, "Alice")).unwrap();
    assert_eq!(alice_account.candidate_vote, 1);
}

#[test]
fn test_vote_before_start_fails() { // it creates a poll starting at time 1500 (while our mock clock is at 1000).
    let mut ctx = setup();
    let authority = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let voter = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    let _poll_id = 1;
    initialize_poll(
        &mut ctx,
        &authority,
        _poll_id,
        1_500,
        2_000,
        "Future Poll",
        "Voting has not started yet",
    );
    initialize_candidate(&mut ctx, &authority, _poll_id, "Alice");

    cast_vote(&mut ctx, &voter, _poll_id, "Alice")
        .assert_failure()
        .assert_anchor_error("VottingNotStarted");
}

#[test]
fn test_vote_after_end_fails() { //creates a poll that already ended at time 500.
    let mut ctx = setup();
    let authority = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let voter = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    let _poll_id = 1;
    initialize_poll(
        &mut ctx,
        &authority,
        _poll_id,
        0,
        500,
        "Closed Poll",
        "Voting has ended",
    );
    initialize_candidate(&mut ctx, &authority, _poll_id, "Alice");

    cast_vote(&mut ctx, &voter, _poll_id, "Alice")
        .assert_failure()
        .assert_anchor_error("VottingEnded");
}