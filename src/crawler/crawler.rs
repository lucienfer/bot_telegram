use teloxide::{
    prelude::*,
};
use tokio::time::{sleep, Duration};
use serde_json::Value;
use std::sync::Arc;

use crate::config::AppState;

/*
    The main fonction of the thread, check the market cap and send notification if is necessary
*/
pub async fn crawler(app_state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let mut last_notified_market_cap: f64 = 0.0;
    loop {
        let response = client
            .get(app_state.dexscreener_url.clone())
            .send()
            .await?;

        if response.status().is_success() {
            let json: Value = response.json().await?;
            if let Some(new_market_cap) = json["pairs"][0]["marketCap"].as_f64() {
                let last_hundred_k = last_notified_market_cap / 100000.0;
                let new_hundred_k = new_market_cap / 100000.0;
                if new_hundred_k != last_hundred_k {
                    last_notified_market_cap = new_market_cap;
                    match get_users_id(&app_state.pool).await {
                        Ok(user_ids) => {
                            println!("Utilisateurs à notifier : {:?}", user_ids);
                            for user_id in user_ids {
                                let chat_id = ChatId(user_id);
                                let mc_str = get_mc(&mut last_notified_market_cap.clone());
                                if new_hundred_k < last_hundred_k {
                                    app_state.bot.send_message(chat_id, format!("The market cap of the BROTHER going down: {mc_str}")).await?;
                                } else {
                                    app_state.bot.send_message(chat_id, format!("The market cap of the BROTHER going up: {mc_str}")).await?;
                                }
                            }
                        },
                        Err(e) => eprintln!("Erreur lors de la récupération des utilisateurs : {}", e),
                    }
                }
            } else {
                println!("Market Cap non trouvé ou non valide");
            }
        } else {
            println!("La requête a échoué avec le statut: {}", response.status());
        }
        sleep(Duration::from_secs(1)).await;
    }
}


/*
    Get all id who have enable notification on the db
*/
async fn get_users_id(pool: &sqlx::PgPool) -> Result<Vec<i64>, sqlx::Error> {
    let user_ids = sqlx::query!(
        r#"SELECT user_id FROM "user" WHERE notification = true"#
    )
    .fetch_all(pool)
    .await;
    match user_ids{
        Ok(user_ids) => {
            let mut ids: Vec<i64> = vec![];
            for user in user_ids {
                ids.push(user.user_id);
            }
            return Ok(ids);
        },
        Err(_) => todo!(),
    }
}

/*
    Formats the market cap
*/
fn get_mc(market_cap: &mut f64) -> String {
    let mut count: u8 = 0;
    while *market_cap >= 10.0 && count < 6 {
        *market_cap = *market_cap / 10.0;
        count += 1;
    }
    if count > 4 {
        format!("{:.2}M", market_cap)
    } else if count > 2 {
        format!("{:.2}K", market_cap)
    } else {
        format!("{:.2}", market_cap)
    }
}