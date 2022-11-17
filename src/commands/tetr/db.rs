use crate::{DbPool, Error};
use anyhow::Context;

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

#[derive(thiserror::Error, Debug)]
pub enum DbError {
    #[error("{0}")]
    Duplicate(String),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

pub async fn insert_monitor(pool: &DbPool, monitor: &Monitor) -> Result<(), DbError> {
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
                return Err(anyhow::anyhow!(
                    "failed to insert monitor, rows affected: {}",
                    res.rows_affected()
                )
                .into());
            }
        }
        Err(e) => {
            if let Some(code) = e.as_database_error().and_then(|de| de.code()) {
                if code == "23505" {
                    Err(DbError::Duplicate(
                        "duplicate error while inserting monitor".into(),
                    ))?
                }
            }
            Err(e).context("failed to insert monitor")?;
        }
    };
    Ok(())
}

pub async fn delete_monitor(
    pool: &DbPool,
    channel_id: &str,
    username: &str,
) -> Result<Monitor, DbError> {
    Ok(sqlx::query_as!(
        Monitor,
        r#"
DELETE FROM monitor
WHERE channel_id = $1 AND username = $2
RETURNING *"#,
        channel_id,
        username
    )
    .fetch_one(pool)
    .await
    .context("failed to delete monitor")?)
}
