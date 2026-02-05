use ureq::{Agent, AgentBuilder};
use std::time::Duration;
pub struct BrowserClient {
    agent: Agent,
}

impl BrowserClient {
    pub fn new() -> Self {
        let agent = AgentBuilder::new()
            .timeout_read(Duration::from_secs(10))
            .timeout_write(Duration::from_secs(10))
            .user_agent("FAGA Browser/0.1")
            .build();
        Self { agent }
    }
    pub fn fetch(&self, url: &str) -> Result<String, ureq::Error> {
        let response = self.agent.get(url).call()?;
        if response.status() == 200 {
            Ok(response.into_string()?)
        } else {
            Err(ureq::Error::Status(response.status(), response))
        }
    }
}