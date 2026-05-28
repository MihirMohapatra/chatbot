use std::sync::{Arc, Mutex};

use anyhow::Result;
use axum::{
    Router,
    extract::State,
    response::{Html, Json},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::client::ChatClient;
use crate::conversation::Conversation;

#[derive(Clone)]
struct AppState {
    client: ChatClient,
    conv: Arc<Mutex<Conversation>>,
}

#[derive(Deserialize)]
struct ChatRequest {
    message: String,
}

#[derive(Serialize)]
struct ChatResponse {
    reply: String,
}

async fn chat(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> Json<ChatResponse> {
    let messages = {
        let mut conv = state.conv.lock().unwrap();
        conv.add_user(&req.message);
        conv.messages.clone()
    };

    match state.client.send(&messages).await {
        Ok(reply) => {
            let mut conv = state.conv.lock().unwrap();
            conv.add_assistant(&reply);
            Json(ChatResponse { reply })
        }
        Err(e) => {
            let mut conv = state.conv.lock().unwrap();
            conv.messages.pop();
            Json(ChatResponse {
                reply: format!("Error: {}", e),
            })
        }
    }
}

async fn index() -> Html<&'static str> {
    Html(HTML)
}

pub async fn start(client: ChatClient, port: u16) -> Result<()> {
    let state = AppState {
        client,
        conv: Arc::new(Mutex::new(Conversation::new())),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/api/chat", post(chat))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    println!("Web UI at http://localhost:{}/", port);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

const HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Chatbot</title>
<style>
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #111; color: #e0e0e0; height: 100vh; display: flex; flex-direction: column; }
  #messages { flex: 1; overflow-y: auto; padding: 20px; display: flex; flex-direction: column; gap: 12px; }
  .msg { max-width: 75%; padding: 10px 14px; border-radius: 12px; line-height: 1.5; white-space: pre-wrap; }
  .user { align-self: flex-end; background: #2563eb; color: #fff; }
  .assistant { align-self: flex-start; background: #2a2a2a; color: #e0e0e0; }
  .error { align-self: flex-start; background: #3b1010; color: #f87171; }
  #input-bar { display: flex; gap: 8px; padding: 12px 20px; border-top: 1px solid #2a2a2a; background: #181818; }
  #input { flex: 1; padding: 10px 14px; border: 1px solid #333; border-radius: 8px; background: #222; color: #e0e0e0; font-size: 14px; outline: none; }
  #input:focus { border-color: #2563eb; }
  #send { padding: 10px 20px; border: none; border-radius: 8px; background: #2563eb; color: #fff; font-size: 14px; cursor: pointer; }
  #send:hover { background: #1d4ed8; }
  #send:disabled { opacity: .5; cursor: not-allowed; }
  .loading { align-self: flex-start; color: #888; font-style: italic; }
</style>
</head>
<body>
<div id="messages"></div>
<div id="input-bar">
  <input id="input" type="text" placeholder="Type a message..." autofocus>
  <button id="send">Send</button>
</div>
<script>
const messages = document.getElementById('messages');
const input = document.getElementById('input');
const send = document.getElementById('send');

function addMsg(role, text) {
  const div = document.createElement('div');
  div.className = 'msg ' + role;
  div.textContent = text;
  messages.appendChild(div);
  messages.scrollTop = messages.scrollHeight;
}

async function sendMsg() {
  const text = input.value.trim();
  if (!text) return;
  input.value = '';
  addMsg('user', text);
  send.disabled = true;
  const loader = document.createElement('div');
  loader.className = 'loading';
  loader.textContent = 'Assistant is thinking...';
  messages.appendChild(loader);
  try {
    const res = await fetch('/api/chat', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ message: text }) });
    const data = await res.json();
    loader.remove();
    addMsg(data.reply.startsWith('Error:') ? 'error' : 'assistant', data.reply);
  } catch {
    loader.remove();
    addMsg('error', 'Error: Network request failed');
  }
  send.disabled = false;
  input.focus();
}

send.addEventListener('click', sendMsg);
input.addEventListener('keydown', e => { if (e.key === 'Enter') sendMsg(); });
</script>
</body>
</html>"#;
