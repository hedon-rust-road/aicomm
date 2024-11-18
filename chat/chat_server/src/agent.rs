use std::env;

use ai_sdk::{AiAdapter, AiService, OllamaAdapter, OpenAIAdapter, TestAdapter};
use chat_core::{AdapterType, Agent, AgentDecision, AgentError, AgentType, ChatAgent};

pub enum AgentVariant {
    Proxy(ProxyAgent),
    Reply(ReplyAgent),
    Tap(TapAgent),
}

#[allow(unused)]
pub struct ProxyAgent {
    pub name: String,
    pub adapter: AiAdapter,
    pub prompt: String,
    pub args: serde_json::Value,
}

#[allow(unused)]
pub struct ReplyAgent {
    pub name: String,
    pub adapter: AiAdapter,
    pub prompt: String,
    pub args: serde_json::Value,
}

#[allow(unused)]
pub struct TapAgent {
    pub name: String,
    pub adapter: AiAdapter,
    pub prompt: String,
    pub args: serde_json::Value,
}

impl Agent for ProxyAgent {
    async fn process(
        &self,
        msg: &str,
        _ctx: &chat_core::AgentContext,
    ) -> Result<AgentDecision, AgentError> {
        // If we need it to be fiexible: prompt should be a jinja2 template, and args is a json.
        let prompt = format!("{} {}", self.prompt, msg);
        let messages = vec![ai_sdk::Message::user(prompt)];
        let res = self.adapter.complete(&messages).await?;
        Ok(AgentDecision::Modify(res))
    }
}

impl Agent for ReplyAgent {
    async fn process(
        &self,
        msg: &str,
        _ctx: &chat_core::AgentContext,
    ) -> Result<AgentDecision, AgentError> {
        // TODO: enhance the reply agent promption
        // 1. create embedding for the message
        // 2. search related messages from vector db with embedding
        // 3. query llm with prompt and related messages
        let prompt = format!("{} {}", self.prompt, msg);
        let messages = vec![ai_sdk::Message::user(prompt)];
        let res = self.adapter.complete(&messages).await?;
        Ok(AgentDecision::Reply(res))
    }
}

impl Agent for TapAgent {
    async fn process(
        &self,
        _msg: &str,
        _ctx: &chat_core::AgentContext,
    ) -> Result<AgentDecision, AgentError> {
        Ok(AgentDecision::None)
    }
}

impl Agent for AgentVariant {
    async fn process(
        &self,
        msg: &str,
        ctx: &chat_core::AgentContext,
    ) -> Result<AgentDecision, AgentError> {
        match self {
            AgentVariant::Proxy(agent) => agent.process(msg, ctx).await,
            AgentVariant::Reply(agent) => agent.process(msg, ctx).await,
            AgentVariant::Tap(agent) => agent.process(msg, ctx).await,
        }
    }
}

impl From<ChatAgent> for AgentVariant {
    fn from(mut agent: ChatAgent) -> Self {
        let adapter = match agent.adapter {
            AdapterType::OpenAI => {
                let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY is not set");
                OpenAIAdapter::new(api_key, agent.model).into()
            }
            AdapterType::Ollama => OllamaAdapter::new_local(agent.model).into(),
            AdapterType::Test => TestAdapter::default().into(),
        };

        match agent.r#type {
            AgentType::Proxy => AgentVariant::Proxy(ProxyAgent {
                name: agent.name,
                adapter,
                prompt: agent.prompt,
                args: agent.args.take(),
            }),
            AgentType::Reply => AgentVariant::Reply(ReplyAgent {
                name: agent.name,
                adapter,
                prompt: agent.prompt,
                args: agent.args.take(),
            }),
            AgentType::Tap => AgentVariant::Tap(TapAgent {
                name: agent.name,
                adapter,
                prompt: agent.prompt,
                args: agent.args.take(),
            }),
        }
    }
}

impl From<ProxyAgent> for AgentVariant {
    fn from(value: ProxyAgent) -> Self {
        AgentVariant::Proxy(value)
    }
}

impl From<ReplyAgent> for AgentVariant {
    fn from(value: ReplyAgent) -> Self {
        AgentVariant::Reply(value)
    }
}

impl From<TapAgent> for AgentVariant {
    fn from(value: TapAgent) -> Self {
        AgentVariant::Tap(value)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use chat_core::AgentContext;

    use crate::AppState;

    use super::*;

    #[ignore]
    #[tokio::test]
    async fn agent_variant_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let agents = state.list_agents(1).await.expect("list agents failed");
        let agent = agents[0].clone();
        let agent: AgentVariant = agent.into();
        let decision = agent.process("hello", &AgentContext::default()).await?;
        if let AgentDecision::Modify(res) = decision {
            println!("{}", res);
        } else {
            panic!("agent decision is not modify");
        }
        Ok(())
    }
}
