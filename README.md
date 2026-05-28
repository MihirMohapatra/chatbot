# Rust Chatbot
<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.80+-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/Web-Axum%20%2B%20Tokio-46E0B6?style=for-the-badge" alt="Web Framework">
</p>
A terminal chatbot in Rust supporting **Claude**, **OpenAI**, and **Ollama** (local models).

## Setup

1. **Install Rust:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Configure your provider:**
   ```bash
   cp .env.example .env
   # Edit .env to set your PROVIDER and API key
   ```

3. **Run:**

   **CLI mode (default):**
   ```bash
   cargo run
   ```

   **Web UI mode:**
   ```bash
   cargo run -- web
   # Open http://localhost:8080
   ```

   Custom port:
   ```bash
   PORT=3000 cargo run -- web
   ```

## Docker

```bash
# Build the image
docker build -t chatbot .

# Run with your .env file
docker run -it --rm -v .env:/data/.env chatbot
```

For **Ollama** (local), use host networking:
```bash
docker run -it --rm --network host -v .env:/data/.env chatbot
```

---

## Provider setup

### Claude (Anthropic)
```env
PROVIDER=claude
ANTHROPIC_API_KEY=sk-ant-...
```
Get a key at https://console.anthropic.com

### OpenAI
```env
PROVIDER=openai
OPENAI_API_KEY=sk-...
```
Get a key at https://platform.openai.com

### Ollama (free, local, no key needed)
```bash
# 1. Install Ollama: https://ollama.com
# 2. Pull a model:
ollama pull llama3.2
```
```env
PROVIDER=ollama
MODEL=llama3.2
```

### OpenRouter (access Claude/GPT/Gemini with one key)
```env
PROVIDER=openai
OPENAI_API_KEY=sk-or-...
BASE_URL=https://openrouter.ai/api
MODEL=anthropic/claude-sonnet-4
```

---

## All .env options

| Variable | Default | Description |
|---|---|---|
| `PROVIDER` | `claude` | `claude`, `openai`, or `ollama` |
| `ANTHROPIC_API_KEY` | — | Required for Claude |
| `OPENAI_API_KEY` | — | Required for OpenAI |
| `MODEL` | provider default | Override the model name |
| `BASE_URL` | provider default | Override the API endpoint |
| `MAX_TOKENS` | `1024` | Max response length |
| `SYSTEM_PROMPT` | built-in | Custom system prompt |

---

## Project structure

```
src/
├── main.rs          # REPL loop
├── client.rs        # Multi-provider HTTP client
├── conversation.rs  # Message history
└── config.rs        # Config + Provider enum
```
