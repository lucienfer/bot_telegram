use sqlx::postgres::PgPoolOptions;
use dotenv::dotenv;
use teloxide::{
    dispatching::{dialogue::InMemStorage},
    prelude::*,
    utils::command::BotCommands,
};
use std::sync::Arc;

use bot_telegram::handler::commands::Command::StartCommand;
use bot_telegram::handler::handler::schema;
use bot_telegram::handler::handler::State;
use bot_telegram::crawler::crawler::crawler;
use bot_telegram::config::AppState;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Connection to the db
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create database pool");
    // Make migrations
    sqlx::migrate!("./migrations").run(&pool).await.unwrap(); 

    let app_state = Arc::new(AppState::new(pool.clone()));

    // Set the command
    if let Err(e) = app_state.bot.set_my_commands(StartCommand::bot_commands()).await{
        log::error!("Erreur lors de l'enregistrement des commandes : {:?}", e);
    }

    // Spawn new thread and launch the crawler
    let app_state_clone = Arc::clone(&app_state);
    tokio::spawn(crawler(app_state_clone));
    
    // Start the bot
    Dispatcher::builder(app_state.bot.clone(), schema())
        .dependencies(dptree::deps![InMemStorage::<State>::new(), app_state.clone()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}