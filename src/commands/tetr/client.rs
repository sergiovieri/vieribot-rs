use crate::{Context, Error};
use anyhow::Context as anyhowContext;
use serde_derive::Deserialize;
use serde_json::Value;

#[allow(dead_code)]
#[derive(Deserialize)]
struct UserResponse {
    success: bool,
    error: Option<String>,
    cache: Option<Value>,
    data: Option<UserResponseData>,
}

#[derive(Deserialize)]
struct UserResponseData {
    user: TetrUser,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct TetrUser {
    pub _id: String,
    pub username: String,
    pub role: String,
    pub gamesplayed: i32,
    pub gametime: f64,
    pub country: Option<String>,
    pub league: TetraLeagueStanding,
    pub connections: TetrUserConnections,
}

#[derive(Deserialize)]
pub struct TetraLeagueStanding {
    pub gamesplayed: i32,
    pub rating: f64,
    pub rank: String,
}

#[derive(Deserialize)]
pub struct TetrUserConnections {
    pub discord: Option<TetrUserDiscordConnection>,
}

#[derive(Deserialize)]
pub struct TetrUserDiscordConnection {
    pub id: String,
    pub username: String,
}

static TETR_API_BASE_URL: &'static str = "https://ch.tetr.io/api/";

pub async fn get_user(ctx: Context<'_>, user: &str) -> Result<TetrUser, Error> {
    let reqwest = &ctx.data().reqwest;
    let response = reqwest
        .get(reqwest::Url::parse(TETR_API_BASE_URL)?.join(&format!("users/{}", user))?)
        .send()
        .await
        .context("error sending request to tetr.io")?
        .error_for_status()
        .context("tetr.io api call failed")?
        .json::<UserResponse>()
        .await
        .context("failed to parse tetr.io data")?;
    if !response.success {
        Err(anyhow::anyhow!(
            "tetr.io API unsuccessful for `{}`:\n{}",
            user,
            response.error.unwrap_or("unknown".into())
        ))?;
    }
    response
        .data
        .map(|r| Ok(r.user))
        .unwrap_or(Err(anyhow::anyhow!("User field not found for `{}`", user)))
}

pub fn get_user_avatar_url(user_id: &str) -> String {
    format!("https://tetr.io/user-content/avatars/{}.jpg", user_id)
}
