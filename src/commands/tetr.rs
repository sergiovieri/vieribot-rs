use crate::{CommandResult, Context, Error};
use serde_derive::Deserialize;
use serde_json::Value;

#[derive(Debug)]
struct Monitor {
    channel_id: String,
    user_id: String,
    username: String,
    last_match_id: Option<String>,
    game_time: f64,
    last_personal_best_blitz: Option<i32>,
    last_personal_best_40l: Option<i32>,
}

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
    user: User,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct User {
    _id: String,
    username: String,
    role: String,
}

static TETR_API_BASE_URL: &'static str = "https://ch.tetr.io/api/";

async fn get_user(ctx: Context<'_>, user: &str) -> Result<User, Error> {
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

fn get_user_avatar_url(user: &User) -> String {
    format!("https://tetr.io/user-content/avatars/{}.jpg", user._id)
}

/// Tetr.io tracking
#[poise::command(
    prefix_command,
    slash_command,
    subcommands("list", "monitor", "test"),
    guild_cooldown = 5
)]
pub async fn tetr(ctx: Context<'_>) -> CommandResult {
    ctx.say("heheh").await?;
    Ok(())
}

/// List tetr.io monitored users
#[poise::command(prefix_command, slash_command, guild_cooldown = 5)]
pub async fn list(ctx: Context<'_>) -> CommandResult {
    let recs = sqlx::query_as!(
        Monitor,
        r#"
    SELECT * FROM monitor WHERE channel_id = $1"#,
        ctx.channel_id().to_string()
    )
    .map(|m| m.username)
    .fetch_all(&ctx.data().db_pool)
    .await?;

    if recs.len() == 0 {
        ctx.say("No monitored users").await?;
    } else {
        ctx.say(recs.join("\n")).await?;
    }
    Ok(())
}

/// Monitor a tetr.io user
#[poise::command(prefix_command, slash_command, guild_cooldown = 5)]
pub async fn monitor(
    ctx: Context<'_>,
    #[description = "Tetr username to monitor"] user: String,
) -> CommandResult {
    let user_data = get_user(ctx, &user).await?;

    let res = sqlx::query!(
        r#"
    INSERT INTO monitor (channel_id, user_id, username, game_time)
    VALUES ($1, $2, $3, $4)"#,
        ctx.channel_id().to_string(),
        user_data._id,
        user,
        0_f64
    )
    .execute(&ctx.data().db_pool)
    .await?
    .rows_affected();

    if res != 1 {
        return Err("Failed to add user".into());
    }

    ctx.send(|b| {
        b.embed(|b| {
            b.title(format!("Saved {}", user))
                .description(&user_data._id)
                .thumbnail(get_user_avatar_url(&user_data))
        })
    })
    .await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command, guild_cooldown = 5)]
pub async fn test(ctx: Context<'_>, #[description = "int test"] id: i32) -> CommandResult {
    ctx.send(|b| {
        b.embed(|b| {
            b.title("Test")
                .color(234748)
                .description(id)
                .footer(|b| b.text("By Vieri Corp.™ All Rights Reserved"))
        })
    })
    .await?;
    Ok(())
}
