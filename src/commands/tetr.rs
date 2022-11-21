use crate::{CommandResult, Context, Error};
use db::Monitor;

use anyhow::Context as anyhowContext;
use country_emoji::code_to_flag;
use futures::{stream, StreamExt};
use poise::serenity_prelude::CreateEmbed;
use pretty_duration::pretty_duration;
use std::time::Duration;

mod client;
mod db;

const MAX_CONCURRENT_REQUESTS: usize = 64;

fn format_tetr_user<'a>(user: &client::TetrUser, b: &'a mut CreateEmbed) -> &'a mut CreateEmbed {
    let join_time = user
        .ts
        .as_ref()
        .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
        .map(|t| chrono::Utc::now() - t.with_timezone(&chrono::Utc))
        .and_then(|d| d.to_std().ok());
    if let Some(d) = join_time {
        b.description(format!("Joined {}", timeago::Formatter::new().convert(d)));
    }
    b.field(
        "Play time",
        pretty_duration(&Duration::from_secs(user.gametime as u64), None),
        true,
    )
    .field("Online games", user.gamesplayed, true)
    .field("Games won", user.gameswon, true)
    .thumbnail(client::get_user_avatar_url(&user._id))
}

fn append_latency(b: &mut CreateEmbed, l: Duration) -> &mut CreateEmbed {
    b.footer(|b| {
        b.text(format!(
            "Time taken: {}.{}ms",
            l.as_millis(),
            l.subsec_nanos() % 1_000_000
        ))
    })
}

/// Tetr.io tracking
#[poise::command(
    prefix_command,
    slash_command,
    subcommands("list", "monitor", "test", "remove", "record", "refresh", "monitor2"),
    guild_cooldown = 5
)]
pub async fn tetr(ctx: Context<'_>) -> CommandResult {
    ctx.say("heheh").await?;
    Ok(())
}

/// List tetr.io monitored users
#[poise::command(prefix_command, slash_command, guild_cooldown = 5)]
pub async fn list(ctx: Context<'_>) -> CommandResult {
    let monitors = db::get_monitors_for_channel(&ctx.data().db_pool, ctx.channel_id().to_string())
        .await
        .context("failed to add user")?;

    if monitors.is_empty() {
        ctx.say("No monitored users").await?;
    } else {
        ctx.say(
            monitors
                .iter()
                .map(|m| m.username.as_str())
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .await?;
    }
    Ok(())
}

async fn fetch_new_monitor(ctx: &Context<'_>, user: String) -> Result<Monitor, Error> {
    let user_data = client::get_user(ctx, &user).await?;
    let m = Monitor {
        channel_id: ctx.channel_id().to_string(),
        user_id: user_data._id.clone(),
        username: user_data.username.clone(),
        game_time: user_data.gametime,
        games_played: user_data.gamesplayed,
        last_match_id: None,
        last_personal_best_40l: None,
        last_personal_best_blitz: None,
    };
    Ok(m)
}

/// Monitor tetr.io users
#[poise::command(prefix_command, slash_command, owners_only, guild_cooldown = 5)]
pub async fn monitor2(
    ctx: Context<'_>,
    #[description = "Tetr usernames/ids to monitor"] users: Vec<String>,
) -> CommandResult {
    let reply_handle = ctx.say("Fetching users from tetr.io").await?;
    let start = std::time::Instant::now();
    let monitors = stream::iter(users)
        .map(|u| async { (fetch_new_monitor(&ctx, u.clone()).await, u) })
        .buffer_unordered(MAX_CONCURRENT_REQUESTS)
        .collect::<Vec<_>>()
        .await;
    dbg!(start.elapsed());
    let mut failed_users = vec![];
    let mut fetched_users = vec![];
    for (m, u) in monitors.iter() {
        match m {
            Ok(m) => fetched_users.push(m),
            Err(why) => {
                println!("error fetching user {}: {:?}", u, why);
                failed_users.push(u.clone());
            }
        }
    }

    let mut num_inserted = 0;
    let mut num_duplicate = 0;
    for m in fetched_users {
        match db::insert_monitor(&ctx.data().db_pool, m).await {
            Ok(_) => {
                num_inserted += 1;
            }
            Err(e) => match e {
                db::DbError::Duplicate(_) => {
                    num_duplicate += 1;
                }
                db::DbError::Internal(_) => {
                    println!("failed to insert monitor to db {:?}", e);
                    failed_users.push(m.username.clone())
                }
            },
        }
    }

    let latency = start.elapsed();
    reply_handle
        .edit(ctx, |b| {
            b.embed(|b| {
                b.title(format!("Monitored {} new users", num_inserted));
                append_latency(b, latency);
                if !failed_users.is_empty() {
                    b.field("Failed users", failed_users.join("\n"), true);
                }
                if num_duplicate != 0 {
                    b.field("Duplicate users", num_duplicate, true);
                }
                b
            })
        })
        .await
        .context("failed to edit reply")?;

    Ok(())
}

/// Monitor a tetr.io user
#[poise::command(prefix_command, slash_command, guild_cooldown = 5)]
pub async fn monitor(
    ctx: Context<'_>,
    #[description = "Tetr username/id to monitor"] user: String,
) -> CommandResult {
    let user_data = client::get_user(&ctx, &user).await?;

    // Create new monitor
    let m = Monitor {
        channel_id: ctx.channel_id().to_string(),
        user_id: user_data._id.clone(),
        username: user_data.username.clone(),
        game_time: user_data.gametime,
        games_played: user_data.gamesplayed,
        last_match_id: None,
        last_personal_best_40l: None,
        last_personal_best_blitz: None,
    };

    match db::insert_monitor(&ctx.data().db_pool, &m).await {
        Ok(_) => {
            ctx.send(|b| {
                b.embed(|b| {
                    b.title(format!(
                        "Saved {} {}",
                        &user_data.username,
                        code_to_flag(user_data.country.as_deref().unwrap_or_default())
                            .unwrap_or_default()
                    ));
                    format_tetr_user(&user_data, b)
                })
            })
            .await?;
        }
        Err(e) => match e {
            db::DbError::Duplicate(_) => {
                ctx.send(|b| {
                    b.embed(|b| {
                        b.title(format!("{} was already added", user_data.username))
                            .description(&user_data._id)
                            .color((255, 0, 0))
                            .thumbnail(client::get_user_avatar_url(&user_data._id))
                    })
                })
                .await?;
            }
            db::DbError::Internal(_) => {
                Err(e).context("failed to insert monitor into DB")?;
            }
        },
    };
    Ok(())
}

/// Remove a tetr.io user from the monitor list
#[poise::command(prefix_command, slash_command, guild_cooldown = 5)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "Tetr username to remove"] user: String,
) -> CommandResult {
    let m = db::delete_monitor(&ctx.data().db_pool, &ctx.channel_id().to_string(), &user).await?;
    ctx.send(|b| {
        b.embed(|b| {
            b.title(format!("{} removed from the list", user))
                .description(&m.user_id)
                .thumbnail(client::get_user_avatar_url(&m.user_id))
                .footer(|b| b.text("By Vieri Corp.™ All Rights Reserved"))
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

#[poise::command(prefix_command, slash_command, guild_cooldown = 5)]
pub async fn record(
    ctx: Context<'_>,
    #[description = "Tetr username to get record"] user: String,
) -> CommandResult {
    let record = client::get_user_record(&ctx, &user).await?;
    println!("{:#?}", record);
    Ok(())
}

async fn refresh_single(ctx: &Context<'_>, m: &Monitor) -> Result<(), Error> {
    let record = client::get_user_record(ctx, &m.user_id)
        .await
        .context("failed to get user record")?;
    println!("{:#?}", record);
    Ok(())
}

#[poise::command(prefix_command, slash_command, guild_cooldown = 5)]
pub async fn refresh(ctx: Context<'_>) -> CommandResult {
    let reply_handle = ctx.say("Refreshing").await?;
    let start = std::time::Instant::now();
    let monitors = db::get_monitors_for_channel(&ctx.data().db_pool, ctx.channel_id().to_string())
        .await
        .context("failed to get monitored users from db")?;
    let results = stream::iter(monitors)
        .map(|m| async { (refresh_single(&ctx, &m).await, m) })
        .buffer_unordered(MAX_CONCURRENT_REQUESTS)
        .collect::<Vec<_>>()
        .await;
    let mut num_errors = 0;
    for (result, m) in results.iter() {
        if let Err(why) = result {
            println!(
                "error during refresh for {} {}: {:?}",
                m.channel_id, m.user_id, why
            );
            num_errors += 1;
        }
    }

    let latency = start.elapsed();
    let title: &str;
    let description: String;
    if num_errors > 0 {
        title = "Refresh finished with errors";
        description = format!(
            "Refreshed {:?}/{:?} users.",
            results.len() - num_errors,
            results.len(),
        );
    } else {
        title = "Refresh finished";
        description = format!("Refreshed {:?} users.", results.len());
    }
    reply_handle
        .edit(ctx, |b| {
            b.embed(|b| {
                b.title(title).description(description);
                append_latency(b, latency)
            })
        })
        .await?;
    Ok(())
}
