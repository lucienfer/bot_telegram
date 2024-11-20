use teloxide::prelude::*;

pub struct AppState {
    pub bot: Bot,
    pub pool: sqlx::PgPool,
    pub dexscreener_url: String,
    pub degen_token: String,
}

impl AppState {
    pub fn new(pool: sqlx::PgPool) -> Self {
        let dexscreener_pair = std::env::var("DEXSCREENER_PAIR").expect("DEXSCREENER_PAIR must be set");

        AppState {
            bot: Bot::from_env(),
            pool: pool,
            dexscreener_url: format!("https://api.dexscreener.com/latest/dex/pairs/starknet/{dexscreener_pair}"),
            degen_token: std::env::var("DEGEN_TOKEN").expect("DEGEN_TOKEN must be seet"),
        }
    }
}