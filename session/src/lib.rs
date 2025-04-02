#![no_std]
#![allow(warnings)]
use gstd::{debug, exec, msg, prelude::*, ActorId, MessageId};
use session_game_io::*;
use wordle_game_io::WordleAction;

static mut SESSION: Option<Session> = None;

const MAX_ATTEMPTS: u8 = 5;

#[no_mangle]
extern "C" fn init() {
    let target_program_id = msg::load().expect("Unable to decode Init");
    let session = Session {
        target_program_id,
        session_status: SessionStatus::None,
        start_block_height: exec::block_height(),
        attempts: MAX_ATTEMPTS,
        msg_ids: (MessageId::zero(), MessageId::zero()),
    };
    unsafe {
        SESSION = Some(session);
    }
}

#[no_mangle]
extern "C" fn handle() {
    let mut session = unsafe { SESSION.as_mut().expect("Session is not initialized") }.clone();
    let action: SessionAction = msg::load().expect("Unable to decode `Action`");

    let current_status = session.session_status.clone();

    match current_status {
        SessionStatus::None | SessionStatus::Waiting => match action {
            SessionAction::StartGame => {
                session.start_block_height = exec::block_height();
                session.attempts = MAX_ATTEMPTS;

                unsafe {
                    let msg_id = msg::send(
                        session.target_program_id,
                        WordleAction::StartGame {
                            user: msg::source(),
                        },
                        0,
                    )
                    .expect("Error in sending a message");

                    session.msg_ids = (msg_id, msg::id());
                }
                session.session_status = SessionStatus::MessageSent;
                unsafe {
                    SESSION = Some(session);
                }
                exec::wait();
            }
            SessionAction::CheckWord(word) => {
                if session.attempts == 0 {
                    session.session_status = SessionStatus::GameOver(Outcome::Lose);
                    msg::reply(SessionEvent::GameError("No more attempts left".into()), 0)
                        .expect("Error in sending a reply");
                    return;
                }

                unsafe {
                    let msg_id = msg::send(
                        session.target_program_id,
                        WordleAction::CheckWord {
                            user: msg::source(),
                            word,
                        },
                        0,
                    )
                    .expect("Error in sending a message");

                    session.msg_ids = (msg_id, msg::id());
                }
                session.attempts -= 1;
                session.session_status = SessionStatus::MessageSent;
                unsafe {
                    SESSION = Some(session);
                }
                exec::wait();
            }
            SessionAction::CheckGameStatus => {
                let current_block_height = exec::block_height();
                let block_difference =
                    current_block_height.saturating_sub(session.start_block_height);

                if block_difference >= 200 {
                    session.session_status = SessionStatus::GameOver(Outcome::Lose);
                    msg::reply(SessionEvent::GameError("Game timeout".into()), 0)
                        .expect("Error in sending a reply");
                }
                unsafe {
                    SESSION = Some(session);
                }
            }
        },
        SessionStatus::MessageSent => {
            msg::reply(
                SessionEvent::GameError("Message has already been sent".into()),
                0,
            )
            .expect("Error in sending a reply");
        }
        SessionStatus::MessageReceived(event) => match event {
            Event::GameStarted { user } => {
                msg::send_delayed(exec::program_id(), SessionAction::CheckGameStatus, 0, 200)
                    .expect("Failed to send delayed message");

                msg::reply(SessionEvent::GameStarted { user }, 0)
                    .expect("Error in sending a reply");

                session.session_status = SessionStatus::Waiting;
                unsafe {
                    SESSION = Some(session);
                }
            }
            Event::WordChecked {
                user,
                correct_positions,
                contained_in_word,
            } => {
                if correct_positions.len() == 5 {
                    session.session_status = SessionStatus::GameOver(Outcome::Win);
                }

                msg::reply(
                    SessionEvent::WordChecked {
                        user,
                        correct_positions: correct_positions.to_vec(),
                        contained_in_word: contained_in_word.to_vec(),
                    },
                    0,
                )
                .expect("Error in sending a reply");

                session.session_status = SessionStatus::Waiting;
                unsafe {
                    SESSION = Some(session);
                }
            }
        },
        SessionStatus::GameOver(outcome) => {
            debug!("GAME ENDED: {:?}", outcome);
        }
    }
}

#[no_mangle]
extern "C" fn handle_reply() {
    let reply_to = msg::reply_to().expect("Failed to query reply_to data");
    let session = unsafe { SESSION.as_mut().expect("Session is not initialized") };
    let event: Event = msg::load().expect("Unable to decode `Event`");
    session.session_status = SessionStatus::MessageReceived(event);
    exec::wake(session.msg_ids.1).expect("Failed to wake up the message");
}

#[no_mangle]
extern "C" fn state() {
    let session = unsafe { SESSION.as_ref().expect("Session is not initialized") };
    msg::reply(session, 0).expect("Unable to get the state");
}
