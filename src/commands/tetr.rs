use crate::{CommandResult, Context};

use db::Monitor;

mod client;
mod db;

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
    let monitors = db::get_monitors_for_channel(&ctx.data().db_pool, ctx.channel_id().to_string())
        .await
        .map_err(|e| format!("Failed to add user: {e:?}"))?;

    if monitors.len() == 0 {
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

/// Monitor a tetr.io user
#[poise::command(prefix_command, slash_command, guild_cooldown = 5)]
pub async fn monitor(
    ctx: Context<'_>,
    #[description = "Tetr username/id to monitor"] user: String,
) -> CommandResult {
    let user_data = client::get_user(ctx, &user).await?;

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

    match db::insert_monitor(&ctx.data().db_pool, &m).await? {
        db::InsertResult::Duplicate => {
            ctx.send(|b| {
                b.embed(|b| {
                    b.title(format!("{} was already added", user_data.username))
                        .description(&user_data._id)
                        .color((255, 0, 0))
                        .thumbnail(client::get_user_avatar_url(&user_data))
                })
            })
            .await?
        }
        db::InsertResult::Success => {
            ctx.send(|b| {
                b.embed(|b| {
                    b.title(format!("Saved {}", user_data.username))
                        .description(&user_data._id)
                        .thumbnail(client::get_user_avatar_url(&user_data))
                })
            })
            .await?
        }
    };
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
