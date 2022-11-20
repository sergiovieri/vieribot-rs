use crate::{Context, Error};
use anyhow::Context as anyhowContext;
use serde_derive::Deserialize;
use serde_json::Value;

#[allow(dead_code)]
#[derive(Deserialize)]
struct TetrResponse<T> {
    success: bool,
    error: Option<String>,
    cache: Option<Value>,
    data: Option<T>,
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
    pub ts: Option<String>,
    pub xp: f64,
    pub gamesplayed: i32,
    pub gameswon: i32,
    pub gametime: f64,
    pub country: Option<String>,
    pub league: TetraLeagueStanding,
    pub connections: TetrUserConnections,
    pub friend_count: Option<i32>,
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

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct TetrUserRecord {
    records: TetrUserRankRecord,
    zen: TetrUserZenRecord,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct TetrUserRankRecord {
    #[serde(rename = "40l")]
    _40l: TetrUser40lRecord,
    blitz: TetrUserBlitzRecord,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct TetrUser40lRecord {
    record: Option<TetrRecord>,
    rank: Option<i32>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct TetrUserBlitzRecord {
    record: Option<TetrRecord>,
    rank: Option<i32>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct TetrUserZenRecord {
    level: i32,
    score: i32,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct TetrRecord {
    ts: String,
    endcontext: Value,
}

static TETR_API_BASE_URL: &str = "https://ch.tetr.io/api/";

pub async fn get_user(ctx: &Context<'_>, user: &str) -> Result<TetrUser, Error> {
    let reqwest = &ctx.data().reqwest;
    let response = reqwest
        .get(reqwest::Url::parse(TETR_API_BASE_URL)?.join(&format!("users/{}", user))?)
        .send()
        .await
        .context("error sending request to tetr.io")?
        .error_for_status()
        .context("tetr.io API call failed")?
        .json::<TetrResponse<UserResponseData>>()
        .await
        .with_context(|| format!("failed to parse tetr.io data for {}", user))?;
    if !response.success {
        Err(anyhow::anyhow!(
            "tetr.io API unsuccessful for `{}`:\n{}",
            user,
            response.error.as_deref().unwrap_or("unknown")
        ))?;
    }
    response
        .data
        .map(|r| r.user)
        .ok_or_else(|| anyhow::anyhow!(format!("user field now found for {}", user)))
}

pub async fn get_user_record(ctx: &Context<'_>, user: &str) -> Result<TetrUserRecord, Error> {
    let reqwest = &ctx.data().reqwest;
    let response = reqwest
        .get(reqwest::Url::parse(TETR_API_BASE_URL)?.join(&format!("users/{}/records", user))?)
        .send()
        .await
        .context("error sending request to tetr.io")?
        .error_for_status()
        .context("tetr.io API call failed")?
        .json::<TetrResponse<TetrUserRecord>>()
        .await
        .with_context(|| format!("failed to parse tetr.io data for {}", user))?;
    if !response.success {
        Err(anyhow::anyhow!(
            "tetr.io API unsuccessful for `{}`:\n{}",
            user,
            response.error.as_deref().unwrap_or("unknown")
        ))?;
    }
    response
        .data
        .ok_or_else(|| anyhow::anyhow!("user field not found for {}", user))
}

pub fn get_user_avatar_url(user_id: &str) -> String {
    format!("https://tetr.io/user-content/avatars/{}.jpg", user_id)
}
