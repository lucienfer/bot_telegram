use teloxide::types::ChatId;
use crate::handler::handler::HandlerResult;

pub async fn create_new_user(addr: String, user_id: i64, chat: ChatId, pool: &sqlx::PgPool, notification: bool) -> HandlerResult {
    let chat_id = chat.0;
    let result = sqlx::query!(
        r#"INSERT INTO "user" (user_id, chat_id, addr, notification) VALUES ($1, $2, $3, $4)"#,
            user_id,
            chat_id,
            addr,
            notification
        )
        .execute(pool)
        .await;
    match result {
        Ok(_) => println!("Insertion réussie"),
        Err(e) => eprintln!("Erreur lors de l'insertion : {:?}", e),
    }
    Ok(())
}