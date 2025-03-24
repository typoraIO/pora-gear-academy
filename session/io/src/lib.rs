#![no_std]
use gmeta::{In, InOut, Metadata};
use gstd::{collections::HashMap, prelude::*, ActorId, MessageId};

pub struct WordleMetadata;

impl Metadata for WordleMetadata {
    type Init = In<ActorId>;
    type Handle = InOut<Action, SessionStatus>;
    type Others = ();
    type Reply = ();
    type Signal = ();
    type State = InOut<StateQuery, StateQueryReply>;
}

pub type State = HashMap<ActorId, Session>;

type SentMessageId = MessageId;
type OriginalMessageId = MessageId;

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct Session {
    pub msg_ids: (SentMessageId, OriginalMessageId),
    pub session_status: SessionStatus,
    pub tries_number: u8,
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum SessionStatus {
    None,
    GameStarted,
    Waiting,
    MessageSent,
    GameOver(Outcome),
    WordChecked {
        user: ActorId,
        correct_positions: Vec<u8>,
        contained_in_word: Vec<u8>,
    },
    InvalidWord,
    NoReplyReceived,
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum Action {
    StartGame,
    CheckWord(String),
    CheckGameStatus,
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum Outcome {
    Win,
    Lose,
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum StateQuery {
    All,
    Player(ActorId),
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum StateQueryReply {
    All(Vec<ActorId>),
    Game(Session),
}
