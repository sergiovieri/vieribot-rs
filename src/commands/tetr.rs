use crate::{Context, Error};

/// Tetr tracking
#[poise::command(
    prefix_command,
    slash_command,
    subcommands("list", "monitor"),
    guild_cooldown = 10
)]
pub async fn tetr(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("heheh").await?;
    Ok(())
}

/// List tetr monitored users
#[poise::command(prefix_command, slash_command)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("No monitored users").await?;
    Ok(())
}

/// Monitor a user
#[poise::command(prefix_command, slash_command, guild_cooldown = 10)]
pub async fn monitor(
    ctx: Context<'_>,
    #[description = "Tetr username to monitor"] user: String,
) -> Result<(), Error> {
    ctx.say(format!("Added {}", user)).await?;
    Ok(())
}
