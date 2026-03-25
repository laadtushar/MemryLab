# Contributing to Memory Palace

Thank you for your interest in contributing to Memory Palace! This guide will help you get set up and make meaningful contributions.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Architecture Overview](#architecture-overview)
- [Adding a New Source Adapter](#adding-a-new-source-adapter)
- [Frontend Development](#frontend-development)
- [Testing](#testing)
- [Pull Request Guidelines](#pull-request-guidelines)
- [Code Style](#code-style)
- [Issue Labels](#issue-labels)

---

## Getting Started

### Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| **Rust** | 1.75+ | Backend (Tauri core) |
| **Node.js** | 20+ | Frontend tooling |
| **pnpm** | 9+ | Package manager |
| **Tauri CLI** | 2.x | Build system |
| **Ollama** | 0.5+ | Local LLM (optional, for AI features) |

### System Dependencies (Windows)

Tauri requires WebView2 (pre-installed on Windows 10/11) and Visual Studio Build Tools with the C++ workload.

### System Dependencies (macOS)

```bash
xcode-select --install
```

### System Dependencies (Linux)

```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

## Development Setup

```bash
# Clone the repository
git clone https://github.com/your-username/MemPalace.git
cd MemPalace

# Install frontend dependencies
pnpm install

# Install Tauri CLI
cargo install tauri-cli@^2

# Run in development mode (hot-reload frontend + Rust backend)
cargo tauri dev

# Build for production
cargo tauri build
```

### Optional: Set up Ollama for AI features

```bash
# Install Ollama from https://ollama.com
ollama pull nomic-embed-text    # Embeddings (384-dim)
ollama pull llama3.1:8b         # LLM for analysis + RAG
```

## Architecture Overview

```
src-tauri/src/
├── adapters/           # Interface implementations
│   ├── llm/            #   Ollama + Claude providers
│   └── storage/        #   SQLite stores
├── app_state.rs        # Shared application state
├── commands/           # Tauri command handlers (frontend <-> backend bridge)
├── domain/
│   ├── models/         #   Document, Chunk, Memory, Entity, etc.
│   └── ports/          #   Trait definitions (interfaces)
├── error.rs            # Unified error type
├── pipeline/
│   ├── analysis/       #   Theme, sentiment, belief, entity extraction
│   └── ingestion/      #   Source adapters, chunker, dedup, orchestrator
├── prompts/            # LLM prompt templates
└── query/              # RAG pipeline

src/                    # React + TypeScript frontend
├── components/
│   ├── ask/            #   RAG chat interface
│   ├── import/         #   Import wizard with 30+ sources
│   ├── layout/         #   AppShell, Sidebar
│   ├── search/         #   Keyword + semantic search
│   ├── settings/       #   LLM config, stats
│   └── timeline/       #   D3.js timeline visualization
├── lib/
│   └── tauri.ts        #   Type-safe Tauri command bindings
└── stores/             # Zustand state management
```

### Key Design Principles

1. **Hexagonal Architecture** — Domain logic depends on traits (ports), not implementations (adapters)
2. **Offline-first** — All data stays local. SQLite + SQLCipher for storage, Ollama for AI
3. **Adapter pattern** — Each data source is a self-contained adapter implementing `SourceAdapter`
4. **Auto-detection** — ZIP/folder imports auto-detect the source format via confidence scoring

## Adding a New Source Adapter

This is the most common contribution. Each adapter is a single Rust file.

### Step 1: Create the adapter file

Create `src-tauri/src/pipeline/ingestion/source_adapters/yourplatform.rs`:

```rust
use std::path::Path;
use chrono::Utc;
use walkdir::WalkDir;
use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;
use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct YourPlatformAdapter;

impl SourceAdapter for YourPlatformAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "your_platform".into(),
            display_name: "Your Platform".into(),
            icon: "your_platform".into(),
            takeout_url: Some("https://example.com/download-data".into()),
            instructions: "Go to Settings > Privacy > Download Your Data. \
                          Choose JSON format. Upload the ZIP file here.".into(),
            accepted_extensions: vec!["zip".into(), "json".into()],
            handles_zip: true,
            platform: SourcePlatform::Custom, // or add a new variant
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        // Return 0.0-1.0 confidence that this adapter handles the given files.
        // Higher = more confident. Look for platform-specific filenames.
        let has_signature = file_listing.iter().any(|f| {
            f.contains("your_platform_specific_file")
        });
        if has_signature { 0.9 } else { 0.0 }
    }

    fn name(&self) -> &str {
        "your_platform"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        {
            let content = std::fs::read_to_string(entry.path())
                .map_err(|e| AppError::Io(e.to_string()))?;
            let json: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| AppError::Parse(e.to_string()))?;

            // Extract text content from the JSON structure
            let text = parse_utils::flatten_json_to_text(&json);
            if text.is_empty() { continue; }

            let doc = parse_utils::build_document(
                SourcePlatform::Custom,
                text,
                Utc::now(), // Replace with actual timestamp parsing
                vec![],
                serde_json::json!({
                    "source_file": entry.path().to_string_lossy(),
                }),
            );
            documents.push(doc);
        }

        Ok(documents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect() {
        let adapter = YourPlatformAdapter;
        let files = vec!["your_platform_specific_file.json"];
        assert!(adapter.detect(&files) > 0.5);
    }

    #[test]
    fn test_metadata() {
        let adapter = YourPlatformAdapter;
        let meta = adapter.metadata();
        assert_eq!(meta.id, "your_platform");
        assert!(meta.takeout_url.is_some());
    }
}
```

### Step 2: Register the adapter

1. Add `pub mod yourplatform;` to `source_adapters/mod.rs`
2. Add `use super::yourplatform::YourPlatformAdapter;` to `source_adapters/registry.rs`
3. Add `Box::new(YourPlatformAdapter),` to the `all_adapters()` vector in `registry.rs`

### Step 3: Add the icon (optional)

Add an SVG icon case in `src/components/import/SourceIcon.tsx`. If you skip this, the fallback renders the first letter in a circle.

### Step 4: Add to categories

Add your adapter's `id` to the appropriate category in `src/components/import/ImportWizard.tsx` (the `CATEGORIES` constant).

### Parse Utilities

`parse_utils.rs` provides these helpers to reduce boilerplate:

| Function | Purpose |
|----------|---------|
| `build_document(platform, text, timestamp, participants, metadata)` | Factory with SHA-256 content hashing |
| `html_to_text(html)` | Strip HTML tags, decode entities |
| `parse_csv_file(path)` | Read CSV file into `Vec<Vec<String>>` |
| `parse_csv_string(content)` | Parse CSV from string |
| `flatten_json_to_text(value)` | Recursively extract all strings from JSON |
| `unwrap_twitter_js(content)` | Strip `window.YTD.x = ` prefix from Twitter JS files |
| `fix_facebook_encoding(text)` | Fix Facebook's broken UTF-8 encoding |

### Detection Scoring Guidelines

| Confidence | When to use |
|-----------|-------------|
| 0.95 | Unique signature file (e.g., `data/tweets.js` for Twitter) |
| 0.85-0.9 | Strong indicators (e.g., `messages/inbox/` for Facebook) |
| 0.5-0.7 | Moderate indicators (e.g., CSV files with expected column names) |
| 0.1-0.3 | Weak heuristic (e.g., generic file patterns) |
| 0.0 | No match |

## Frontend Development

### Tech Stack

- **React 18** with TypeScript
- **Vite** for bundling
- **Tailwind CSS** for styling
- **Zustand** for state management
- **Lucide** for icons
- **D3.js** for timeline visualization

### Adding a new view

1. Create component in `src/components/yourview/YourView.tsx`
2. Add route in `src/stores/app-store.ts` (add to `View` type)
3. Add sidebar entry in `src/components/layout/Sidebar.tsx`
4. Add case in `src/components/layout/AppShell.tsx`

### Tauri Command Bindings

All Tauri commands are typed in `src/lib/tauri.ts`. When adding a new Rust command:

1. Add the `#[tauri::command]` function in the appropriate `commands/*.rs` file
2. Register it in `lib.rs` → `generate_handler![]`
3. Add the TypeScript binding in `src/lib/tauri.ts`

## Testing

### Rust tests

```bash
cd src-tauri
cargo test                    # Run all tests
cargo test source_adapters    # Run adapter tests only
cargo test -- --nocapture     # Show println output
```

### Frontend tests

```bash
pnpm test        # Vitest
pnpm lint        # ESLint
pnpm type-check  # TypeScript compiler check
```

### Testing a source adapter

1. Obtain a real data export from the platform (or create a minimal mock)
2. Place test fixtures in `src-tauri/tests/fixtures/yourplatform/`
3. Write a test that calls `adapter.parse(fixture_path)` and asserts on document count, content, timestamps

## Pull Request Guidelines

### Before submitting

- [ ] `cargo check` passes with no errors
- [ ] `cargo test` passes
- [ ] `pnpm build` succeeds (frontend)
- [ ] New adapter? Added to `mod.rs`, `registry.rs`, and `ImportWizard.tsx`
- [ ] No hardcoded paths or secrets

### PR format

```
## Summary
- Brief description of changes

## Test plan
- How you tested the changes
- For adapters: sample data used

## Screenshots (if UI changes)
```

### Commit messages

We use conventional commits:

```
feat: add Mastodon source adapter
fix: handle empty CSV rows in Reddit adapter
docs: add contributing guide for source adapters
refactor: extract date parsing into parse_utils
```

## Code Style

### Rust

- Follow standard `rustfmt` formatting
- Use `AppError` for all error types (not `unwrap()` or `panic!()`)
- Prefer `walkdir` over manual recursion
- Use `parse_utils` helpers instead of reimplementing CSV/HTML/JSON parsing

### TypeScript

- Strict mode enabled
- Use `interface` over `type` for object shapes
- Components in PascalCase, hooks in camelCase
- Tailwind for all styling (no CSS modules)

## Issue Labels

| Label | Description |
|-------|-------------|
| `good-first-issue` | Great for newcomers — usually adding a simple adapter |
| `adapter` | Related to source adapter implementation |
| `frontend` | React/UI changes |
| `backend` | Rust/Tauri changes |
| `ai-pipeline` | LLM, embeddings, RAG, analysis |
| `bug` | Something isn't working |
| `enhancement` | New feature or improvement |
| `docs` | Documentation only |

## Need Help?

- Open an issue for questions or to discuss your approach before coding
- Check existing adapters for patterns to follow
- The `generic.rs` adapter is the simplest reference implementation

Thank you for contributing!
