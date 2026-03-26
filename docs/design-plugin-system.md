# Plugin System Design (v1.0)

**Status:** Draft
**Last updated:** 2026-03-26
**Target release:** v1.0

---

## Overview

Memory Palace's plugin system enables third-party extensions via WebAssembly (WASM) sandboxing. Plugins can add new import source adapters, analysis stages, UI panels, and export formats without modifying core code. This replaces the current approach of hardcoded `SourceAdapter` trait implementations compiled into the binary.

### Goals

- Allow community-contributed import adapters without rebuilding the app
- Maintain the privacy-first guarantee: plugins run sandboxed with explicit permissions
- Keep the core small: move niche platform adapters to optional plugins over time
- Provide a developer-friendly SDK for plugin authors

### Non-Goals

- Plugins cannot replace core pipeline stages (chunker, normalizer, dedup)
- No plugin marketplace or auto-update mechanism in v1.0
- No inter-plugin communication or dependency chains

---

## Architecture

```
+------------------------------------------------------------------+
|  Memory Palace App                                                |
|                                                                   |
|  +-----------+   +-----------+   +-----------+   +------------+  |
|  | Ingestion |   | Analysis  |   |  Export   |   |     UI     |  |
|  | Pipeline  |   | Pipeline  |   |  Engine   |   |  Renderer  |  |
|  +-----+-----+   +-----+-----+   +-----+-----+   +-----+------+  |
|        |               |               |               |          |
|  +-----v---------------v---------------v---------------v------+  |
|  |              Plugin Registry & Router                      |  |
|  +-----+---------------+---------------+---------------------+  |
|        |               |               |                         |
|  +-----v-----+   +-----v-----+   +-----v-----+                  |
|  | WASM      |   | WASM      |   | WASM      |                  |
|  | Sandbox 1 |   | Sandbox 2 |   | Sandbox 3 |                  |
|  | (Plugin A)|   | (Plugin B)|   | (Plugin C)|                  |
|  +-----------+   +-----------+   +-----------+                  |
+------------------------------------------------------------------+
```

### Plugin Manifest (`plugin.toml`)

Every plugin ships as a directory containing a `plugin.toml` manifest and a compiled `.wasm` module.

```toml
[plugin]
name = "kindle-highlights"
version = "0.1.0"
author = "Community"
description = "Import Kindle highlights and annotations"
license = "MIT"
min_app_version = "1.0.0"
wasm_module = "kindle.wasm"

[capabilities]
read_documents = true
write_memory = true
call_llm = false
ui_panel = false
file_system = { read = true, paths = ["~/Documents/Kindle"] }

[extension_point]
type = "source_adapter"
platform_id = "kindle"
display_name = "Kindle Highlights"
icon = "book"
accepted_extensions = [".txt", ".csv", ".html"]
handles_zip = false
```

### Extension Points

Memory Palace exposes four extension points that plugins can implement:

#### 1. Source Adapters

Plugins implement the `parse` function to add new import formats. The plugin receives a path to a file or extracted directory and returns structured documents.

```
Plugin exports:
  fn metadata() -> SourceAdapterMeta
  fn detect(file_listing: string[]) -> f32
  fn parse(path: string) -> Document[]
```

This mirrors the existing `SourceAdapter` trait (`src-tauri/src/pipeline/ingestion/source_adapters/mod.rs`) but runs inside WASM.

#### 2. Analysis Stages

Plugins can contribute additional analysis after the core pipeline runs. They receive chunks and context, and return insights.

```
Plugin exports:
  fn analyze(chunks: Chunk[], context: AnalysisContext) -> Insight[]
```

Analysis plugins run after the built-in stages (entity extraction, sentiment tracking, belief extraction) and their outputs are stored alongside native insights.

#### 3. UI Panels

Plugins can render custom HTML/JS in a sandboxed iframe within the app. The iframe communicates with the plugin WASM via a postMessage bridge.

```
Plugin exports:
  fn render_panel() -> HtmlContent
  fn handle_event(event: UiEvent) -> UiResponse
```

UI panels appear as additional tabs in the app's sidebar.

#### 4. Export Plugins

Plugins can add new export formats beyond the built-in options.

```
Plugin exports:
  fn export_formats() -> ExportFormat[]
  fn export(data: ExportData, format: string) -> bytes
```

### Host Functions (WASM Imports)

The host application provides these functions to plugin WASM modules:

| Function | Capability Required | Description |
|----------|-------------------|-------------|
| `read_document(id: string) -> Document` | `read_documents` | Fetch a document by ID |
| `search_documents(query: string, limit: u32) -> Document[]` | `read_documents` | Full-text search across documents |
| `list_documents(offset: u32, limit: u32) -> Document[]` | `read_documents` | Paginated document listing |
| `store_memory_fact(fact: MemoryFact)` | `write_memory` | Create a memory fact |
| `store_insight(insight: Insight)` | `write_memory` | Create an insight |
| `call_llm(prompt: string, params: LlmParams) -> string` | `call_llm` | Send prompt to the user's configured LLM |
| `read_file(path: string) -> bytes` | `file_system.read` | Read from allowed paths only |
| `log(level: string, message: string)` | *(always allowed)* | Write to plugin log (visible in Settings) |
| `get_config(key: string) -> string` | *(always allowed)* | Read plugin-specific config values |
| `set_config(key: string, value: string)` | *(always allowed)* | Store plugin-specific config values |

### Security Model

- **WASM sandbox:** Plugins have no raw filesystem, network, or system call access. All I/O goes through host functions.
- **Capability-based permissions:** Declared in `plugin.toml`, approved by user at install time. The app never grants capabilities not listed in the manifest.
- **Instance isolation:** Each plugin runs in its own `wasmtime` instance with a separate memory space. No shared memory between plugins.
- **Resource limits:**
  - Memory: 256 MB per plugin instance
  - CPU time: 30 seconds per invocation (configurable in settings)
  - Stack size: 1 MB
- **No inter-plugin communication:** Plugins cannot call or observe each other.
- **Filesystem allow-list:** `file_system.read` only permits access to paths explicitly listed in the manifest and approved by the user.
- **No network access:** Plugins cannot make HTTP requests. If they need external data, the user must provide it as a local file.

### Plugin Lifecycle

```
  Discovery ──> Validation ──> Registration ──> Activation ──> Execution
       │              │              │               │              │
  Scan plugin    Parse toml,    Register ext    User enables,   Called at
  directories    verify WASM    points, show    approve caps    pipeline
                 compatibility  in Settings                     stage
                                                                  │
                                                           Deactivation
                                                                  │
                                                           Free resources,
                                                           unregister
```

1. **Discovery** -- On app startup, scan `~/.memorypalace/plugins/` for subdirectories containing `plugin.toml`.
2. **Validation** -- Parse the manifest. Check `min_app_version` compatibility. Verify the `.wasm` module exists and is valid WASM. Reject malformed plugins with a clear error in the UI.
3. **Registration** -- Register the plugin's extension points with the appropriate pipeline stage. Source adapters join the `SourceAdapterRegistry`. Analysis stages are appended to the analysis pipeline. Show the plugin in Settings > Plugins.
4. **Activation** -- User explicitly enables the plugin in Settings. The app displays requested capabilities and the user approves. The WASM module is compiled and instantiated.
5. **Execution** -- The plugin is invoked at the appropriate pipeline stage. Source adapters are called during import when `detect()` matches. Analysis plugins run after core analysis. Export plugins appear in the export dialog.
6. **Deactivation** -- User disables the plugin. The WASM instance is dropped, resources freed, extension points unregistered.

### Implementation Crates

| Crate | Version | Purpose |
|-------|---------|---------|
| `wasmtime` | 20+ | WASM runtime with WASI preview2, sandboxing, fuel metering |
| `toml` | 0.8 | Manifest parsing |
| `wit-bindgen` | 0.25+ | WIT interface type generation for host-guest contract |
| `wit-component` | 0.25+ | Component model support |

### Key Files (Future)

```
src-tauri/src/plugins/
  mod.rs          -- Plugin loader and registry
  host.rs         -- Host function implementations (WASM imports)
  manifest.rs     -- plugin.toml parser and validator
  sandbox.rs      -- wasmtime Engine/Store configuration, resource limits
  bridge.rs       -- Type conversion between WASM guest types and domain models

src-tauri/src/domain/ports/
  plugin.rs       -- IPluginRegistry trait (port)

wit/
  mempalace.wit   -- WIT interface definition for the plugin contract
```

---

## Testing Plugins

### Unit Testing (Plugin Author)

Plugin authors test their WASM modules locally using the Memory Palace Plugin SDK (see below). The SDK includes a mock host that simulates all host functions:

```rust
// In plugin's test harness
use mempalace_sdk::testing::MockHost;

#[test]
fn test_kindle_parse() {
    let host = MockHost::new()
        .with_file("highlights.txt", include_bytes!("fixtures/highlights.txt"));
    let docs = kindle::parse(&host, "highlights.txt");
    assert_eq!(docs.len(), 42);
    assert!(docs[0].content.contains("Highlight"));
}
```

### Integration Testing (Core Team)

The core app includes integration tests that load real `.wasm` plugins into a wasmtime sandbox and exercise the full lifecycle:

```
tests/
  plugin_integration/
    test_lifecycle.rs      -- discovery through deactivation
    test_source_adapter.rs -- plugin-provided source adapter in import pipeline
    test_security.rs       -- verify capability enforcement, resource limits
    test_malformed.rs      -- graceful handling of bad manifests and panicking WASM
```

### Security Testing

- Fuzz the WASM module loading path
- Verify that a plugin requesting `file_system.read = ["/etc/passwd"]` is rejected on approval
- Verify that exceeding the 30-second CPU limit terminates the plugin cleanly
- Verify that exceeding the 256 MB memory limit terminates the plugin cleanly
- Verify that a plugin cannot access host functions for capabilities it was not granted

---

## Plugin Development SDK

The SDK is a standalone Rust crate (`mempalace-plugin-sdk`) published to crates.io. It provides:

1. **Type definitions** -- All types used in the host-guest interface (`Document`, `Chunk`, `Insight`, `MemoryFact`, etc.) as Rust structs with serde support.
2. **Procedural macros** -- `#[mempalace_source_adapter]`, `#[mempalace_analysis]`, `#[mempalace_export]` to generate the required WASM exports.
3. **Mock host** -- A testing harness that simulates all host functions without needing the full app.
4. **CLI tool** -- `mempalace-plugin` CLI for scaffolding, building, and validating plugins:

```bash
# Scaffold a new source adapter plugin
mempalace-plugin new --type source-adapter my-kindle-plugin

# Build to WASM
mempalace-plugin build

# Validate manifest and WASM exports
mempalace-plugin validate

# Package for distribution
mempalace-plugin package  # produces my-kindle-plugin-0.1.0.tar.gz
```

### SDK Project Structure (generated by scaffold)

```
my-kindle-plugin/
  plugin.toml          -- manifest
  Cargo.toml           -- Rust project, target = wasm32-wasip2
  src/
    lib.rs             -- plugin implementation
  tests/
    integration.rs     -- tests using MockHost
  fixtures/
    sample_input.txt   -- test data
```

---

## Example Plugin Walkthrough: Kindle Highlights Adapter

### Step 1: Scaffold

```bash
mempalace-plugin new --type source-adapter kindle-highlights
cd kindle-highlights
```

### Step 2: Implement

```rust
// src/lib.rs
use mempalace_sdk::prelude::*;

#[mempalace_source_adapter]
pub struct KindleAdapter;

impl SourceAdapterPlugin for KindleAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "kindle".into(),
            display_name: "Kindle Highlights".into(),
            icon: "book".into(),
            takeout_url: None,
            instructions: "Export your Kindle clippings from your device...".into(),
            accepted_extensions: vec![".txt".into()],
            handles_zip: false,
            platform_id: "kindle".into(),
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        if file_listing.iter().any(|f| f.contains("My Clippings.txt")) {
            0.95
        } else {
            0.0
        }
    }

    fn parse(&self, path: &str) -> Result<Vec<Document>, PluginError> {
        let content = host::read_file(path)?;
        let text = String::from_utf8(content)?;

        let mut documents = Vec::new();
        for entry in text.split("==========") {
            let entry = entry.trim();
            if entry.is_empty() { continue; }

            // Parse Kindle clipping format:
            // Book Title (Author)
            // - Your Highlight on page X | Location Y-Z | Added on Date
            //
            // Highlight text
            let lines: Vec<&str> = entry.lines().collect();
            if lines.len() < 4 { continue; }

            let title = lines[0].trim();
            let highlight = lines[3..].join("\n").trim().to_string();

            documents.push(Document {
                source_platform: "kindle",
                title: title.into(),
                content: highlight,
                // ... fill other fields
            });
        }

        host::log("info", &format!("Parsed {} Kindle highlights", documents.len()));
        Ok(documents)
    }
}
```

### Step 3: Build and Install

```bash
mempalace-plugin build
mempalace-plugin validate
# Copy to plugin directory
cp -r target/plugin/kindle-highlights ~/.memorypalace/plugins/
```

### Step 4: Activate

Open Memory Palace > Settings > Plugins. The Kindle Highlights plugin appears. Click Enable, review capabilities (file_system.read for ~/Documents/Kindle), approve. The adapter now appears in the import dialog.

---

## Migration Path from Hardcoded Adapters

The current codebase has ~30 hardcoded source adapters in `src-tauri/src/pipeline/ingestion/source_adapters/`. These all implement the `SourceAdapter` trait directly. The migration to plugins proceeds in phases:

### Phase 1: Plugin Infrastructure (v1.0)

- Build the plugin runtime (wasmtime integration, manifest parser, sandbox)
- Define the WIT interface matching the existing `SourceAdapter` trait
- The `SourceAdapterRegistry` gains a `register_plugin_adapter()` method
- All existing hardcoded adapters continue working unchanged
- New adapters can be added as plugins

### Phase 2: Dual Mode (v1.1)

- Port 2-3 niche adapters (e.g., Tumblr, Pinterest) to plugins as proof-of-concept
- Remove them from the compiled binary
- Distribute them as bundled plugins that ship with the app
- Validate performance parity (WASM overhead should be negligible for I/O-bound parsing)

### Phase 3: Community Plugins (v1.2+)

- Publish the SDK to crates.io
- Document the plugin development workflow
- Move remaining niche adapters to plugins (keep core 5-6 compiled: Obsidian, Markdown, DayOne, WhatsApp, Twitter, Facebook)
- Community contributes adapters for niche platforms

### Registry Compatibility Layer

The `SourceAdapterRegistry` (`src-tauri/src/pipeline/ingestion/source_adapters/registry.rs`) will be extended to hold both native and plugin-based adapters behind the same `SourceAdapter` trait interface:

```
SourceAdapterRegistry
  native_adapters: Vec<Box<dyn SourceAdapter>>     // compiled-in
  plugin_adapters: Vec<PluginSourceAdapter>         // WASM-backed
```

`PluginSourceAdapter` implements `SourceAdapter` by delegating to the WASM module, translating between Rust domain types and WIT types via `bridge.rs`.

---

## Implementation Estimates

| Task | Effort | Dependencies |
|------|--------|-------------|
| WIT interface definition | 1 week | None |
| Manifest parser + validator | 1 week | None |
| wasmtime sandbox setup + resource limits | 2 weeks | WIT |
| Host function implementations | 2 weeks | Sandbox |
| Plugin registry + lifecycle management | 1 week | Host functions |
| SDK crate (types, macros, mock host) | 2 weeks | WIT |
| CLI tool (scaffold, build, validate) | 1 week | SDK |
| Settings UI for plugin management | 1 week | Registry |
| Port 2 adapters to plugins (proof of concept) | 1 week | All above |
| Security audit + fuzz testing | 1 week | All above |
| **Total** | **~13 weeks** | |

---

## Open Questions

1. **Should plugins be able to register custom Tauri commands?** This would allow richer UI interaction but increases the attack surface.
2. **Component Model vs. Core WASM:** wasmtime supports the Component Model (WASI preview2). Should we require it, or support plain WASM modules too?
3. **Plugin distribution format:** A `.tar.gz` with manifest + WASM, or a custom `.mpp` (MemPalace Plugin) archive?
4. **Hot reload during development:** Should the app watch the plugin directory for changes and reload without restart?
