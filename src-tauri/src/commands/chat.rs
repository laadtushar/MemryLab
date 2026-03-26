use tauri::State;

use crate::adapters::sqlite::chat_store::{ChatConversation, ChatMessage};
use crate::app_state::AppState;

#[tauri::command]
pub fn list_conversations(
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<ChatConversation>, String> {
    let limit = limit.unwrap_or(50);
    state
        .chat_store
        .list_conversations(limit)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_conversation_messages(
    conversation_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ChatMessage>, String> {
    state
        .chat_store
        .get_messages(&conversation_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_conversation(
    state: State<'_, AppState>,
) -> Result<ChatConversation, String> {
    state
        .chat_store
        .create_conversation("New Chat")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_conversation(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .chat_store
        .delete_conversation(&id)
        .map_err(|e| e.to_string())
}
