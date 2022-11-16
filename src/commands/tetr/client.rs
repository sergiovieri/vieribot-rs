use crate::{Context, Error};
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
        .map_err(|e| {
            println!("{:?}", e);
            "Tetr.io api cannot be reached"
        })?
        .error_for_status()
        .map_err(|e| {
            println!("{:?}", e);
            format!("Tetr.io api call failed: {}", e.to_string())
        })?
        .json::<UserResponse>()
        .await
        .map_err(|e| {
            println!("{:?}", e);
            "Failed to parse tetr.io data"
        })?;
    if !response.success {
        println!(
            "Tetr.io api unsuccessful for {}: {:?}",
            user, response.error
        );
        Err(response.error.unwrap_or("Tetr.io api unsuccessful".into()))?
    }
    response
        .data
        .map(|r| Ok(r.user))
        .unwrap_or(Err("User field not found".into()))
}

pub fn get_user_avatar_url(user: &TetrUser) -> String {
    format!("https://tetr.io/user-content/avatars/{}.jpg", user._id)
}
