use crate::Context;

pub async fn send_err_msg(
    ctx: Context<'_>,
    title: impl Into<String>,
    description: impl Into<String>,
) {
    let response = ctx
        .send(|b| {
            b.embed(|b| {
                b.title(title.into())
                    .color((255, 0, 0))
                    .description(description.into())
            })
        })
        .await;
    if let Err(e) = response {
        println!("Failed to send message while handling error: {}", e);
    }
}
