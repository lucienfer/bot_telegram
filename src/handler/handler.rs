use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
    prelude::*,
    types::MessageId,
};
use crate::handler::commands::{
    Command::Command,
    endpoint::commands::{
        start,
        cancel,
        help,
        get_mc,
        balance
    },
    endpoint::callback::{
        receive_addr,
        enable_notification,
        invalid_state
    }
};

pub type MyDialogue = Dialogue<State, InMemStorage<State>>;
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    ReceiveAddr,
    EnableNotification{
        addr: String,
        user_id: i64,
        message_id: MessageId
    },
    Register,
    Exit
}

pub fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(
            case![State::Start]
                .branch(case![Command::Help].endpoint(help))
                .branch(case![Command::Start].endpoint(start))
                .branch(case![Command::Mc].endpoint(get_mc)),
        )
        .branch(
            case![State::Register]
                .branch(case![Command::Help].endpoint(help))
                .branch(case![Command::Mc].endpoint(get_mc))
                .branch(case![Command::Balance].endpoint(balance)),
        )
        .branch(
            case![State::Exit]
                .branch(case![Command::Help].endpoint(help))
                .branch(case![Command::Mc].endpoint(get_mc))
                .branch(case![Command::Start].endpoint(start)),
        )
        .branch(case![Command::Cancel].endpoint(cancel));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::ReceiveAddr].endpoint(receive_addr))
        .branch(dptree::endpoint(invalid_state));

    let callback_query_handler = Update::filter_callback_query()
    .branch(
        case![State::EnableNotification { addr, user_id, message_id }].endpoint(enable_notification),
    );

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
        .branch(callback_query_handler)
}