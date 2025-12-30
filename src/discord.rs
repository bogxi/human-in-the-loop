use std::sync::{Arc, OnceLock};

use serenity::{
    all::{
        AutoArchiveDuration, ChannelId, ChannelType, Context, CreateMessage, CreateThread,
        EventHandler, GatewayIntents, Ready, UserId,
    },
    Client,
};
use tokio::sync::OnceCell;

use crate::tools::Human;

pub async fn start(discord_token: &str, handler: Handler) -> anyhow::Result<()> {
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(discord_token, intents)
        .event_handler(handler)
        .await?;
    Ok(client.start().await?)
}

#[derive(Clone)]
pub struct Handler {
    ctx: Arc<OnceLock<Context>>,
}

impl Default for Handler {
    fn default() -> Self {
        Self {
            ctx: Arc::new(OnceLock::new()),
        }
    }
}

#[async_trait::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _: Ready) {
        self.ctx.set(ctx).ok();
    }
}

pub struct HumanInDiscord {
    user_id: UserId,
    channel_id: ChannelId,
    handler: Handler,
    thread: OnceCell<ChannelId>,
}

impl HumanInDiscord {
    pub fn new(user_id: UserId, channel_id: ChannelId) -> Self {
        Self {
            user_id,
            channel_id,
            handler: Handler::default(),
            thread: OnceCell::new(),
        }
    }

    pub fn handler(&self) -> &Handler {
        &self.handler
    }
}

#[async_trait::async_trait]
impl Human for HumanInDiscord {
    async fn ask(&self, question: &str) -> anyhow::Result<String> {
        let (ctx, thread) = self.get_ctx_and_thread(question).await?;
        let message_text = format!("<@{}> {question}", self.user_id.get());
        thread
            .send_message(&ctx.http, CreateMessage::new().content(message_text))
            .await?;
        let message = thread
            .await_reply(ctx)
            .author_id(self.user_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to await message from the human in Discord"))?;
        Ok(message.content)
    }

    async fn notify(&self, message: &str) -> anyhow::Result<()> {
        let (ctx, thread) = self.get_ctx_and_thread(message).await?;
        let message_text = format!("<@{}> {message}", self.user_id.get());
        thread
            .send_message(&ctx.http, CreateMessage::new().content(message_text))
            .await?;
        Ok(())
    }
}

impl HumanInDiscord {
    async fn get_ctx_and_thread(&self, title: &str) -> anyhow::Result<(&Context, ChannelId)> {
        let ctx = self
            .handler
            .ctx
            .get()
            .ok_or_else(|| anyhow::anyhow!("The connection with Discord is not ready"))?;
        let thread = self
            .thread
            .get_or_try_init(|| async {
                let thread_title = title.chars().take(100).collect::<String>();
                let channel = self
                    .channel_id
                    .create_thread(
                        &ctx.http,
                        CreateThread::new(thread_title)
                            .auto_archive_duration(AutoArchiveDuration::OneDay)
                            .kind(ChannelType::PublicThread),
                    )
                    .await?;
                anyhow::Ok(channel.id)
            })
            .await?;
        Ok((ctx, *thread))
    }
}
