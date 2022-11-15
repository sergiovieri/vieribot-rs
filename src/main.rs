mod commands;
mod error;

use poise::serenity_prelude as serenity;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
type CommandResult<E = Error> = Result<(), E>;
type DbPool = Pool<Postgres>;

// User data, which is stored and accessible in all command invocations
pub struct Data {
    pub reqwest: reqwest::Client,
    pub db_pool: DbPool,
}

/// Show this help menu
#[poise::command(prefix_command, track_edits, slash_command)]
async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> CommandResult {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "\
VieriBot-rs",
            show_context_menu_commands: true,
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

#[poise::command(prefix_command)]
async fn register(ctx: Context<'_>) -> CommandResult {
    println!(
        "{} registered slash commands in {}",
        ctx.author().name,
        ctx.guild()
            .map_or("unknown".into(), |g| format!("{} ({})", g.name, g.id))
    );
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error);
            error::send_err_msg(
                ctx,
                format!("Internal error while processing {}", ctx.command().name),
                error.to_string(),
            )
            .await;
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

async fn init_db() -> DbPool {
    let pool = match PgPoolOptions::new()
        .max_connections(4)
        .connect(&std::env::var("DATABASE_URL").expect("missing DATABASE_URL"))
        .await
    {
        Ok(pool) => pool,
        Err(why) => {
            panic!("Cannot connect to db: {:?}", why)
        }
    };
    pool
}

#[tokio::main]
async fn main() {
    let data = Data {
        reqwest: reqwest::Client::new(),
        db_pool: init_db().await,
    };

    poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                help(),
                register(),
                commands::admin::shutdown(),
                commands::admin::spam(),
                commands::admin::ping(),
                commands::admin::dblatency(),
                commands::tetr::tetr(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!#".into()),
                ..Default::default()
            },
            /// The global error handler for all error cases that may occur
            on_error: |error| Box::pin(on_error(error)),
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .user_data_setup(move |ctx, ready, framework| {
            println!(
                "Ready {}, connected to ({}) guilds",
                ready.user.name,
                ready.guilds.len()
            );
            Box::pin(async {
                serenity::Command::set_global_application_commands(&ctx.http, |c| {
                    *c =
                        poise::builtins::create_application_commands(&framework.options().commands);
                    println!("I now have the following guild slash commands: \n{c:#?}");
                    c
                })
                .await
                .expect("Cannot set global application commands");
                println!("Done setting slash commands");
                Ok(data)
            })
        })
        .run()
        .await
        .unwrap();
}
