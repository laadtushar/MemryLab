use std::sync::Arc;

use rusqlite::params;
use serde::{Deserialize, Serialize};

use super::connection::SqliteConnection;
use crate::error::AppError;
use crate::query::rag_pipeline::RagSource;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatConversation {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub message_count: usize,
    pub last_message_preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub sources: Vec<RagSource>,
    pub created_at: String,
}

pub struct SqliteChatStore {
    db: Arc<SqliteConnection>,
}

impl SqliteChatStore {
    pub fn new(db: Arc<SqliteConnection>) -> Self {
        Self { db }
    }

    pub fn create_conversation(&self, title: &str) -> Result<ChatConversation, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO chat_conversations (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
                params![id, title, now, now],
            )?;
            Ok(ChatConversation {
                id,
                title: title.to_string(),
                created_at: now.clone(),
                updated_at: now,
                message_count: 0,
                last_message_preview: String::new(),
            })
        })
    }

    pub fn list_conversations(&self, limit: usize) -> Result<Vec<ChatConversation>, AppError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT c.id, c.title, c.created_at, c.updated_at,
                        (SELECT COUNT(*) FROM chat_messages WHERE conversation_id = c.id) as msg_count,
                        COALESCE((SELECT content FROM chat_messages WHERE conversation_id = c.id ORDER BY created_at DESC LIMIT 1), '') as last_msg
                 FROM chat_conversations c
                 ORDER BY c.updated_at DESC
                 LIMIT ?1",
            )?;
            let rows = stmt.query_map(params![limit as i64], |row| {
                let last_msg: String = row.get(5)?;
                let preview = if last_msg.len() > 100 {
                    format!("{}...", &last_msg[..100])
                } else {
                    last_msg
                };
                Ok(ChatConversation {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                    message_count: row.get::<_, i64>(4)? as usize,
                    last_message_preview: preview,
                })
            })?;

            let mut entries = Vec::new();
            for row in rows {
                entries.push(row.map_err(AppError::Database)?);
            }
            Ok(entries)
        })
    }

    pub fn get_messages(&self, conversation_id: &str) -> Result<Vec<ChatMessage>, AppError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, conversation_id, role, content, sources, created_at
                 FROM chat_messages
                 WHERE conversation_id = ?1
                 ORDER BY created_at ASC",
            )?;
            let rows = stmt.query_map(params![conversation_id], |row| {
                let sources_str: String = row.get(4)?;
                let sources: Vec<RagSource> =
                    serde_json::from_str(&sources_str).unwrap_or_default();
                Ok(ChatMessage {
                    id: row.get(0)?,
                    conversation_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    sources,
                    created_at: row.get(5)?,
                })
            })?;

            let mut entries = Vec::new();
            for row in rows {
                entries.push(row.map_err(AppError::Database)?);
            }
            Ok(entries)
        })
    }

    pub fn add_message(
        &self,
        conversation_id: &str,
        role: &str,
        content: &str,
        sources: &[RagSource],
    ) -> Result<ChatMessage, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let sources_json = serde_json::to_string(sources)?;
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO chat_messages (id, conversation_id, role, content, sources, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![id, conversation_id, role, content, sources_json, now],
            )?;
            // Update conversation timestamp
            conn.execute(
                "UPDATE chat_conversations SET updated_at = ?1 WHERE id = ?2",
                params![now, conversation_id],
            )?;
            Ok(ChatMessage {
                id,
                conversation_id: conversation_id.to_string(),
                role: role.to_string(),
                content: content.to_string(),
                sources: sources.to_vec(),
                created_at: now,
            })
        })
    }

    pub fn update_conversation_title(&self, id: &str, title: &str) -> Result<(), AppError> {
        self.db.with_conn(|conn| {
            conn.execute(
                "UPDATE chat_conversations SET title = ?1 WHERE id = ?2",
                params![title, id],
            )?;
            Ok(())
        })
    }

    pub fn delete_conversation(&self, id: &str) -> Result<(), AppError> {
        self.db.with_conn(|conn| {
            conn.execute(
                "DELETE FROM chat_messages WHERE conversation_id = ?1",
                params![id],
            )?;
            conn.execute(
                "DELETE FROM chat_conversations WHERE id = ?1",
                params![id],
            )?;
            Ok(())
        })
    }
}
