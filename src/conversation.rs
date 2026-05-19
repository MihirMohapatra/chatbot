use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

pub struct Conversation {
    pub messages: Vec<Message>,
}

impl Conversation {
    pub fn new() -> Self {
        Conversation {
            messages: Vec::new(),
        }
    }

    pub fn add_user(&mut self, content: impl Into<String>) {
        self.messages.push(Message {
            role: "user".to_string(),
            content: content.into(),
        });
    }

    pub fn add_assistant(&mut self, content: impl Into<String>) {
        self.messages.push(Message {
            role: "assistant".to_string(),
            content: content.into(),
        });
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}
