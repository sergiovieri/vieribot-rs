use crate::{Context, Error};

/// Shuts down the bot
#[poise::command(prefix_command, owners_only, hide_in_help)]
pub async fn shutdown(ctx: Context<'_>) -> Result<(), Error> {
    ctx.framework()
        .shard_manager()
        .lock()
        .await
        .shutdown_all()
        .await;
    Ok(())
}

/// Spam?
#[poise::command(prefix_command, slash_command, guild_cooldown = 10)]
pub async fn spam(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("heheh").await?;
    Ok(())
}
