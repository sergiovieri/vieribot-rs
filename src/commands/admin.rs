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

/// Ping
#[poise::command(prefix_command, slash_command, guild_cooldown = 5)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let start = std::time::Instant::now();
    let reply_handle = ctx.say("Pon").await?;
    let latency = start.elapsed();
    reply_handle
        .edit(ctx, |b| {
            b.content("Pon").embed(|b| {
                b.title("Pon").description(format!(
                    "{}.{}ms",
                    latency.as_millis(),
                    latency.subsec_nanos() % 1_000_000
                ))
            })
        })
        .await?;
    Ok(())
}

/// Measure db latency
#[poise::command(prefix_command, slash_command, guild_cooldown = 5)]
pub async fn dblatency(ctx: Context<'_>) -> Result<(), Error> {
    let start = std::time::Instant::now();
    let pool = &ctx.data().db_pool;
    let rows = sqlx::query!(
        r#"
    SELECT * FROM monitor WHERE channel_id = $1"#,
        ctx.channel_id().to_string()
    )
    .fetch_all(pool)
    .await?
    .len();
    let latency = start.elapsed();
    ctx.send(|b| {
        b.embed(|b| {
            b.title("DB scan latency").description(format!(
                "{}.{}ms\nScanned {rows:?} rows",
                latency.as_millis(),
                latency.subsec_nanos() % 1_000_000
            ))
        })
    })
    .await?;
    Ok(())
}
