# Coding Practice

A local-first, AI-driven programming learning tool. Configure your preferred language and skill level, and the app automatically fetches source materials, generates interactive exercises with TODO markers, and evaluates your solutions with personalized feedback and scoring.

## What It Does

1. **Configure** — Set your preferred programming language (Python, Rust, Go, C++), skill level, and daily exercise quota through a first-run wizard.
2. **Generate** — The system fetches source code from GitHub and curated tutorial sites, then uses AI to split the code into exercises with `TODO` markers replacing key sections.
3. **Practice** — Complete the TODO sections in a Monaco code editor with syntax highlighting, then submit your solution.
4. **Get Feedback** — AI evaluates your code for structural completeness (0–100 score, A–F letter grade) and provides specific strengths and areas for improvement.
5. **Track Progress** — Dashboard shows daily grades, a 7-day score trend, and per-exercise submission history.

## Prerequisites

- **Rust** — 2024 edition toolchain ([rustup](https://rustup.rs/))
- **Node.js** — v18+ and npm
- **AI Service** — one of:
  - [Ollama](https://ollama.com) running locally (default: `llama3.2`)
  - OpenAI API key (`gpt-4o`, `gpt-4o-mini`, etc.)
  - Anthropic API key (`claude-sonnet-4-20250514`, etc.)

## Getting Started

### 1. Start the backend

```bash
cargo run
```

The server starts at `http://127.0.0.1:8001`. On first launch it creates the SQLite database and runs all migrations automatically.

To use a different port, create a `config.toml` in the project root:

```toml
[server]
port = 8080
```

### 2. Start the frontend

```bash
cd frontend
npm install
npm run dev
```

The frontend starts at `http://localhost:5173` and proxies all `/api` requests to the backend.

### 3. Configure

Open `http://localhost:5173` in your browser. On first visit you'll be redirected to the configuration wizard to select your language, skill level, AI model, and daily quotas.

### 4. Set up AI provider (if using API-based models)

For OpenAI models:

```bash
export OPENAI_API_KEY=sk-...
```

For Anthropic models:

```bash
export ANTHROPIC_API_KEY=sk-ant-...
```

For Ollama, just make sure it's running locally at `http://localhost:11434`. No API key needed.

## Project Structure

```
├── migrations/              # SQLite schema migrations (auto-run on startup)
├── src/                     # Rust backend (Axum)
│   ├── main.rs              # Server entry point
│   ├── api/                 # HTTP route handlers
│   ├── ai/                  # AI provider abstraction (Ollama, OpenAI, Anthropic)
│   ├── config/              # User configuration service
│   ├── db/                  # Database pool, repositories, models
│   ├── exercise/            # Exercise generation pipeline
│   ├── material/            # Source code fetching & caching (GitHub, tutorials)
│   ├── scoring/             # Score-to-grade logic & validation
│   └── submission/          # Submission processing & AI evaluation
├── frontend/                # Vue 3 + TypeScript frontend
│   └── src/
│       ├── api/             # Axios API clients
│       ├── components/      # Vue components (editor, grade display, etc.)
│       ├── stores/          # Pinia state stores
│       ├── views/           # Page views (dashboard, editor, results)
│       └── router/          # Vue Router with config guard
└── config.toml             # Optional server config (port override)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/config` | Get user configuration |
| POST | `/api/config` | Save user configuration |
| GET | `/api/ollama/models` | List available local Ollama models |
| GET | `/api/materials` | List cached source materials |
| POST | `/api/materials/fetch` | Fetch new materials from GitHub/tutorials |
| DELETE | `/api/materials/{id}` | Delete a cached material |
| GET | `/api/exercises` | List generated exercises |
| POST | `/api/exercises/generate` | Generate exercises from cached materials |
| GET | `/api/exercises/{id}` | Get a specific exercise |
| DELETE | `/api/exercises/{id}` | Delete an exercise |
| POST | `/api/submissions` | Submit code for AI evaluation |
| GET | `/api/submissions` | List submissions (optional `?exercise_id=` filter) |
| GET | `/api/submissions/{id}` | Get submission with score and feedback |
| GET | `/api/submissions/{id}/solution` | View original solution code |
| GET | `/api/progress/daily` | Daily progress (average score, grade) |
| GET | `/api/progress/trend` | 7-day score trend |

## Database

SQLite is used for all data storage. The database is created automatically at:

- **Linux:** `~/.local/share/coding-practice/app.db`
- **macOS:** `~/Library/Application Support/coding-practice/app.db`

No manual setup required — migrations run on every startup.

## Development

### Backend

```bash
cargo check          # Type-check without building
cargo test --lib     # Run unit tests
cargo run            # Start server with auto-recompile
```

### Frontend

```bash
cd frontend
npm run dev          # Dev server with HMR
npm run build        # Production build
npm run type-check   # TypeScript type checking
npm test             # Run Vitest
```

## Tech Stack

| Layer | Technologies |
|-------|-------------|
| Backend | Rust 2024, Axum 0.8, SQLx 0.9 (SQLite), Tokio |
| Frontend | Vue 3.5, TypeScript, Vite 5, Pinia, Tailwind CSS 4 |
| Editor | Monaco Editor 0.55 |
| Charts | ApexCharts 5, vue3-apexcharts |
| AI | Ollama / OpenAI / Anthropic (pluggable) |

## License

Private project — all rights reserved.