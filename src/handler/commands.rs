/*
    Module for defining state-dependent commands 
*/
pub mod Command {
    use teloxide::{
        utils::command::BotCommands,
    };

    #[derive(BotCommands, Clone)]
    #[command(rename_rule = "lowercase", description = "These commands are supported:")]
    pub enum Command {
        #[command(description = "display this text.")]
        Help,
        #[command(description = "handle a username.")]
        Start,
        #[command(description = "Get the Market cap of the degen token")]
        Mc,
        #[command(description = "Get the price of your balance in usd")]
        Balance,
        #[command(description = "cancel the purchase procedure.")]
        Cancel,
    }

    #[derive(BotCommands, Clone)]
    #[command(rename_rule = "lowercase", description = "These commands are supported:")]
    pub enum ExitCommand {
        #[command(description = "display this text.")]
        Help,
        #[command(description = "handle a username.")]
        Start,
        #[command(description = "Get the Market cap of the degen token")]
        Mc,
    }

    #[derive(BotCommands, Clone)]
    #[command(rename_rule = "lowercase", description = "These commands are supported:")]
    pub enum StartCommand {
        #[command(description = "display this text.")]
        Help,
        #[command(description = "cancel the purchase procedure.")]
        Cancel,
    }

    #[derive(BotCommands, Clone)]
    #[command(rename_rule = "lowercase", description = "These commands are supported:")]
    pub enum RegisterCommand {
        #[command(description = "display this text.")]
        Help,
        #[command(description = "Get the Market cap of the degen token")]
        Mc,
        #[command(description = "Get the price of your balance in usd")]
        Balance,
    }
}

/*
    Module to regroup all endpoint for the handler
*/
pub mod endpoint {

/*
    All endpoint can be call by a specifique command
*/
    pub mod commands {
        use teloxide::{
            prelude::*,
            utils::command::BotCommands,
        };
        use serde_json::Value;
        use std::sync::Arc;
        use crate::handler::{
            commands::Command::{Command, ExitCommand, StartCommand, RegisterCommand},
            handler::{State, MyDialogue, HandlerResult},
        };
        use crate::handler::commands::utils::update_commands;
        use crate::utils::starknet::read::get_balance_erc20;
        use crate::config::AppState;

        /*
            Define the command /start
        */
        pub async fn start(bot: Bot, dialogue: MyDialogue, msg: Message, app_state: Arc<AppState>) -> HandlerResult {
            let user = sqlx::query!(
                r#"SELECT chat_id FROM "user" WHERE user_id = $1"#,
                msg.from.unwrap().id.0 as i64,
            )
            .fetch_all(&app_state.pool)
            .await?;
            if !user.is_empty() {
                bot.send_message(msg.chat.id, "Your are already register").await?;
                dialogue.update(State::Register).await?;
                update_commands(&bot, &State::Start).await?;
            } else {
                bot.send_message(msg.chat.id, "Let's start! What's your address?").await?;
                dialogue.update(State::ReceiveAddr).await?;
                update_commands(&bot, &State::Start).await?;
            }
            Ok(())
        }

        /*
            Define the command /help
        */
        pub async fn help(bot: Bot, msg: Message, dialogue: MyDialogue) -> HandlerResult {
            match dialogue.get().await? {
                Some(State::Exit) => {bot.send_message(msg.chat.id, ExitCommand::descriptions().to_string()).await?;},
                Some(State::Register) => {bot.send_message(msg.chat.id, RegisterCommand::descriptions().to_string()).await?;},
                Some(_) => {bot.send_message(msg.chat.id, StartCommand::descriptions().to_string()).await?;},
                None => {bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;}
            };
            Ok(())
        }

        /*
            Define the command /cancel
        */
        pub async fn cancel(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
            bot.send_message(msg.chat.id, "Cancelling the dialogue.").await?;
            dialogue.update(State::Exit).await?;
            update_commands(&bot, &State::Exit).await?;
            Ok(())
        }

        /*
            Define the command /balance
        */
        pub async fn balance(bot: Bot, msg: Message, app_state: Arc<AppState>) -> HandlerResult {
            let user = sqlx::query!(
                r#"SELECT * FROM "user" WHERE user_id = $1"#,
                msg.from.unwrap().id.0 as i64,
            )
            .fetch_all(&app_state.pool)
            .await?;
            let balance = get_balance_erc20(app_state.degen_token.clone(), user[0].addr.clone(), true).await;
            bot.send_message(msg.chat.id, format!("Balance of brother = {balance}$")).await?;
            Ok(())
        }

        /*
            Define the command /mc
        */
        pub async fn get_mc(msg: Message, bot: Bot, app_state: Arc<AppState>) -> HandlerResult {
            let client = reqwest::Client::new();

            let response = client
                .get(app_state.dexscreener_url.clone())
                .send()
                .await?;

            if response.status().is_success() {
                let json: Value = response.json().await?;
                if let Some(market_cap) = json["pairs"][0]["marketCap"].as_f64() {
                    bot.send_message(msg.chat.id, format!("The market cap of the BROTHER is {market_cap}")).await?;
                } else {
                    println!("Market Cap non trouvé ou non valide");
                }
            } else {
                println!("La requête a échoué avec le statut: {}", response.status());
            }
            Ok(())
        }
    }
    
/*
    All endpoint reachable by an event or an dialogue
*/
    pub mod callback {
        use teloxide::{
            prelude::*,
            types::{InlineKeyboardButton, InlineKeyboardMarkup, MessageId}
        };
        use std::sync::Arc;
        use crate::config::AppState;
        
        use crate::handler::{
            handler::{State, MyDialogue, HandlerResult},
        };
        use crate::utils::db::create_new_user;
        use crate::handler::commands::utils::update_commands;
        use crate::utils::starknet::read::verify_addr;

        /*
            Define the behavior of the bot when you send your starknet address
        */
        pub async fn receive_addr(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
            match msg.text().map(ToOwned::to_owned) {
                Some(addr) => {
                    if !verify_addr(addr.clone()).await {
                        bot.send_message(msg.chat.id, "This address is invalid. Resend a valid address.").await?;
                        return Ok(());
                    }
                    let products = ["Yes", "No"]
                            .map(|product| InlineKeyboardButton::callback(product, product));
                    let enable_message = bot.send_message(msg.chat.id, "Do you want enable notification ?")
                        .reply_markup(InlineKeyboardMarkup::new([products]))
                        .await?;
                    let user_id = msg.from.unwrap().id.0 as i64;
                    let message_id = enable_message.id;
                    dialogue.update(State::EnableNotification {addr, user_id, message_id}).await?;
                }
                None => {
                    bot.send_message(msg.chat.id, "Please, send me your addr.").await?;
                }
            }
            Ok(())
        }
        
        /*
            Register the user and set or not the notification
        */
        pub async fn enable_notification(bot: Bot, (addr, user_id, message_id): (String, i64, MessageId), dialogue: MyDialogue, q: CallbackQuery, app_state: Arc<AppState>) -> HandlerResult {
            if let Some(enable) = &q.data {
                println!("tests: {}", enable);
                bot.delete_message(dialogue.chat_id(), message_id).await?;
                let mut notification: bool = false;
                if enable == "Yes" {
                    notification = true;
                }
                create_new_user(addr, user_id, dialogue.chat_id(), &app_state.pool, notification).await?;
                bot.send_message(dialogue.chat_id(), "Your are now register!")
                    .await?;
                dialogue.update(State::Register).await?;
                update_commands(&bot, &State::Register).await?;
            }
            Ok(())
        }

        pub async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
            bot.send_message(msg.chat.id, "Unable to handle the message. Type /help to see the usage.")
                .await?;
            Ok(())
        }
    }
}

/*
    Utils module for commands module.
*/
pub mod utils {
    use teloxide::{
        utils::command::BotCommands,
        Bot,
        prelude::*,
        requests::ResponseResult,
    };
    use crate::handler::handler::State;
    use crate::handler::commands::Command::{StartCommand, ExitCommand, RegisterCommand};

/*
    Update the list of command usable depend of the state given in param
*/
    pub async fn update_commands(bot: &Bot, state: &State) -> ResponseResult<()> {
        match state {
            State::Start => bot.set_my_commands(StartCommand::bot_commands()).await?,
            State::Exit => bot.set_my_commands(ExitCommand::bot_commands()).await?,
            State::Register => bot.set_my_commands(RegisterCommand::bot_commands()).await?,
            _ => bot.set_my_commands(StartCommand::bot_commands()).await?,
        };
        Ok(())
    }
}