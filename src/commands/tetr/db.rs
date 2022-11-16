use crate::{DbPool, Error};

#[derive(Debug)]
pub struct Monitor {
    pub channel_id: String,
    pub user_id: String,
    pub username: String,
    pub last_match_id: Option<String>,
    pub game_time: f64,
    pub games_played: i32,
    pub last_personal_best_blitz: Option<i32>,
    pub last_personal_best_40l: Option<i32>,
}

pub async fn get_monitors_for_channel(
    pool: &DbPool,
    channel_id: String,
) -> Result<Vec<Monitor>, Error> {
    Ok(sqlx::query_as!(
        Monitor,
        r#"
SELECT * FROM monitor WHERE channel_id = $1"#,
        channel_id
    )
    .fetch_all(pool)
    .await?)
}

pub enum InsertResult {
    Success,
    Duplicate,
}

pub async fn insert_monitor(pool: &DbPool, monitor: &Monitor) -> Result<InsertResult, Error> {
    let res = sqlx::query!(
        r#"
INSERT INTO monitor (channel_id, user_id, username, last_match_id, game_time, games_played,
    last_personal_best_blitz, last_personal_best_40l)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        monitor.channel_id,
        monitor.user_id,
        monitor.username,
        monitor.last_match_id,
        monitor.game_time,
        monitor.games_played,
        monitor.last_personal_best_blitz,
        monitor.last_personal_best_40l,
    )
    .execute(pool)
    .await;

    match res {
        Ok(res) => {
            if res.rows_affected() != 1 {
                println!(
                    "Failed to insert monitor, rows affected: {:?}",
                    res.rows_affected()
                );
                Err("Failed to insert".into())
            } else {
                Ok(InsertResult::Success)
            }
        }
        Err(e) => {
            if let Some(code) = e.as_database_error().and_then(|de| de.code()) {
                if code == "23505" {
                    return Ok(InsertResult::Duplicate);
                }
            }
            Err(e.into())
        }
    }
}
