use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use tauri::State;

use super::queries;
use super::schema::{
    AppendedMessage, Conversation, ConversationId, ConversationSummary, NewMessage, SystemPrompt,
};
use super::{Db, DbError};

async fn with_conn<F, T>(db: State<'_, Db>, f: F) -> Result<T, DbError>
where
    F: FnOnce(&mut Connection) -> Result<T, DbError> + Send + 'static,
    T: Send + 'static,
{
    let arc: Arc<Mutex<Connection>> = db.arc();
    tokio::task::spawn_blocking(move || {
        let mut guard = arc.lock().expect("db mutex poisoned");
        f(&mut guard)
    })
    .await
    .expect("db spawn_blocking join")
}

// -- chat history --------------------------------------------------------

#[tauri::command]
pub async fn list_conversation_summaries(
    db: State<'_, Db>,
) -> Result<Vec<ConversationSummary>, DbError> {
    with_conn(db, |c| queries::list_conversation_summaries(c)).await
}

#[tauri::command]
pub async fn load_conversation(
    db: State<'_, Db>,
    id: String,
) -> Result<Conversation, DbError> {
    with_conn(db, move |c| queries::load_conversation(c, &id)).await
}

#[tauri::command]
pub async fn start_conversation(
    db: State<'_, Db>,
    title: String,
) -> Result<ConversationId, DbError> {
    with_conn(db, move |c| queries::start_conversation(c, &title)).await
}

#[tauri::command]
pub async fn append_message(
    db: State<'_, Db>,
    conversation_id: String,
    message: NewMessage,
) -> Result<AppendedMessage, DbError> {
    with_conn(db, move |c| queries::append_message(c, &conversation_id, &message)).await
}

#[tauri::command]
pub async fn rename_conversation(
    db: State<'_, Db>,
    id: String,
    title: String,
) -> Result<(), DbError> {
    with_conn(db, move |c| queries::rename_conversation(c, &id, &title)).await
}

#[tauri::command]
pub async fn delete_conversation(db: State<'_, Db>, id: String) -> Result<(), DbError> {
    with_conn(db, move |c| queries::delete_conversation(c, &id)).await
}

#[tauri::command]
pub async fn delete_all_conversations(db: State<'_, Db>) -> Result<(), DbError> {
    with_conn(db, |c| queries::delete_all_conversations(c)).await
}

// -- system prompts ------------------------------------------------------

#[tauri::command]
pub async fn list_system_prompts(db: State<'_, Db>) -> Result<Vec<SystemPrompt>, DbError> {
    with_conn(db, |c| queries::list_system_prompts(c)).await
}

#[tauri::command]
pub async fn create_system_prompt(
    db: State<'_, Db>,
    name: String,
    prompt: String,
) -> Result<SystemPrompt, DbError> {
    with_conn(db, move |c| queries::create_system_prompt(c, &name, &prompt)).await
}

#[tauri::command]
pub async fn edit_system_prompt(
    db: State<'_, Db>,
    id: i64,
    name: Option<String>,
    prompt: Option<String>,
) -> Result<SystemPrompt, DbError> {
    with_conn(db, move |c| {
        queries::edit_system_prompt(c, id, name.as_deref(), prompt.as_deref())
    })
    .await
}

#[tauri::command]
pub async fn delete_system_prompt(db: State<'_, Db>, id: i64) -> Result<(), DbError> {
    with_conn(db, move |c| queries::delete_system_prompt(c, id)).await
}
