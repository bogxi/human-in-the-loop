use rmcp::{
    handler::server::tool::{Parameters, ToolRouter},
    model::{ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router, ServerHandler,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::future::Future;

#[async_trait::async_trait]
pub trait Human: Send + Sync + 'static {
    async fn ask(&self, question: &str) -> anyhow::Result<String>;
    async fn notify(&self, message: &str) -> anyhow::Result<()>;
}

pub struct HumanInTheLoop<H> {
    human: H,
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AskHumanRequest {
    #[schemars(
        description = "The question to ask the human. Be specific and provide context to help the human understand what information you need. Ask the only one question at once."
    )]
    question: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct NotifyHumanRequest {
    #[schemars(
        description = "The message to send to the human. Use this to provide updates, notifications, or information that doesn't require a direct response."
    )]
    message: String,
}

#[tool_router]
impl<H> HumanInTheLoop<H>
where
    H: Human,
{
    pub fn new(human: H) -> Self {
        Self {
            human,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Ask a human for information that only they would know, such as personal preferences, project-specific context, local environment details, or non-public information"
    )]
    async fn ask_human(
        &self,
        Parameters(AskHumanRequest { question }): Parameters<AskHumanRequest>,
    ) -> Result<String, rmcp::Error> {
        self.human
            .ask(&question)
            .await
            .map_err(|e| rmcp::Error::internal_error(e.to_string(), None))
    }

    #[tool(
        description = "Notify a human with a message. Use this for updates or information that doesn't require a response."
    )]
    async fn notify_human(
        &self,
        Parameters(NotifyHumanRequest { message }): Parameters<NotifyHumanRequest>,
    ) -> Result<String, rmcp::Error> {
        self.human
            .notify(&message)
            .await
            .map_err(|e| rmcp::Error::internal_error(e.to_string(), None))?;
        Ok("Notification sent successfully".to_string())
    }
}

#[tool_handler]
impl<H> ServerHandler for HumanInTheLoop<H>
where
    H: Human,
{
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "This is a Human-in-the-Loop MCP server that enables AI assistants to request \
             information from or notify humans via Discord. \
             Use the 'ask_human' tool when you need information that only a human would know, \
             such as: personal preferences, project-specific context, local environment details, \
             or any information that is not publicly available or documented. \
             Use the 'notify_human' tool to send notifications or updates that don't require \
             a response. The human will be notified in Discord."
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
