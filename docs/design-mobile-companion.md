# Mobile Companion App Design (v1.0)

**Status:** Draft
**Last updated:** 2026-03-26
**Target release:** v1.0
**Depends on:** [Multi-Device Encrypted Sync](design-multi-device-sync.md)

---

## Overview

A lightweight mobile app for iOS and Android that provides read-only access to Memory Palace data plus cloud-powered Ask/RAG chat. The mobile app does not perform imports or analysis -- those are desktop-only operations. The phone becomes a window into your personal memory, always in your pocket.

### Goals

- Read-only browse and search across all memory data
- Ask questions via RAG (using cloud LLM with user's API key)
- Sync data from desktop via the Multi-Device Encrypted Sync system
- Native feel on both iOS and Android
- Same privacy guarantees: data stays on-device, LLM calls use user's own key

### Non-Goals

- Import or ingest new data sources on mobile
- Run local LLM inference (Ollama) on mobile
- Run analysis pipeline on mobile (entity extraction, sentiment, etc.)
- Offline AI chat (requires network for cloud LLM)

---

## Technology Decision

### Option A: Tauri Mobile (Recommended)

| Aspect | Detail |
|--------|--------|
| Framework | Tauri 2.0 with mobile targets |
| Backend | Same Rust core compiled for ARM (iOS/Android) |
| Frontend | Same React/TypeScript UI with responsive layout |
| Database | SQLite (native on mobile, same schema) |
| Code sharing | ~80% shared with desktop |
| Binary size | ~15-25 MB (Rust + WebView) |

**Advantages:**
- Single codebase. Bug fixes and features benefit both platforms.
- The hexagonal architecture ports (`IDocumentStore`, `IVectorStore`, etc.) work unchanged on mobile.
- SQLite runs natively; no database migration needed.
- Tauri 2.0's mobile support is production-ready.

### Option B: React Native

| Aspect | Detail |
|--------|--------|
| Framework | React Native with Expo |
| Backend | Rewrite Rust logic in TypeScript or use FFI bridge |
| Database | SQLite via expo-sqlite or WatermelonDB |
| Code sharing | ~40% (UI components only) |
| Binary size | ~30-50 MB |

**Disadvantages:**
- Separate codebase to maintain.
- Rust FFI to React Native is fragile and poorly documented.
- Would need to reimplement domain logic or maintain a translation layer.
- Larger team required for two codebases.

### Option C: Kotlin Multiplatform + SwiftUI

| Aspect | Detail |
|--------|--------|
| Framework | KMP for shared logic, native UI per platform |
| Code sharing | 0% with existing Rust/React codebase |
| Quality | Best native feel |

**Disadvantages:**
- Complete rewrite of backend logic.
- Two UI codebases (SwiftUI + Compose).
- Not viable for a small team.

### Decision: Tauri Mobile (Option A)

The desktop codebase already compiles to mobile targets with Tauri 2.0. The shared Rust backend and React frontend minimize maintenance burden. The read-only mobile scope means we only need a subset of the desktop features, making the adaptation straightforward.

---

## App Architecture

```
+-------------------------------------------------------+
|  Mobile App (Tauri 2.0)                               |
|                                                       |
|  +-------------------+   +-------------------------+  |
|  | React Frontend    |   | Rust Backend (subset)   |  |
|  | (responsive)      |   |                         |  |
|  | - Timeline        |   | - SQLite read queries   |  |
|  | - Search          |   | - Vector search         |  |
|  | - Ask (RAG)       |   | - RAG pipeline          |  |
|  | - Memory Browser  |   | - Cloud LLM client      |  |
|  | - Settings        |   | - Sync engine (read)    |  |
|  +-------------------+   +-------------------------+  |
|           |                         |                  |
|  +--------v-------------------------v--------------+   |
|  |              SQLite Database (synced)            |   |
|  +--------------------------------------------------+  |
+-------------------------------------------------------+
```

### Backend Scope (Mobile Subset)

The mobile Rust backend includes only read-path modules:

```
Included:
  domain/models/*           -- all domain types
  domain/ports/*            -- all port traits
  adapters/sqlite/*         -- SQLite read queries (document_store, memory_store, etc.)
  query/rag_pipeline.rs     -- RAG pipeline for Ask feature
  adapters/llm/claude.rs    -- Cloud LLM client (Claude, OpenAI)
  adapters/llm/openai_compat.rs
  sync/ (read path only)    -- Consume sync events from desktop

Excluded:
  pipeline/ingestion/*      -- no importing on mobile
  pipeline/analysis/*       -- no analysis on mobile
  adapters/llm/ollama.rs    -- no local LLM on mobile
  commands/import.rs        -- no import commands
  commands/embeddings.rs    -- no embedding generation
```

This exclusion is achieved via Cargo feature flags:

```toml
[features]
default = ["desktop"]
desktop = ["ingestion", "analysis", "ollama"]
mobile = ["cloud-llm", "sync-read"]
```

---

## Mobile-Specific Views

### 1. Timeline (Home)

The primary view. A vertically scrolling timeline of the user's personal evolution.

```
+----------------------------------+
|  Memory Palace          [search] |
|----------------------------------|
|  < March 2026 >                  |
|                                  |
|  +----- Mar 26 ---------------+  |
|  | Journal: Morning thoughts  |  |
|  | "Been thinking about..."   |  |
|  +----------------------------+  |
|                                  |
|  +----- Mar 25 ---------------+  |
|  | WhatsApp: 3 conversations  |  |
|  | Twitter: 5 tweets          |  |
|  +----------------------------+  |
|                                  |
|  +----- Mar 24 ---------------+  |
|  | Insight: Your interest in  |  |
|  | philosophy has grown 40%   |  |
|  | over the past quarter.     |  |
|  +----------------------------+  |
|                                  |
|  [Timeline] [Search] [Ask] [Me] |
+----------------------------------+
```

- Swipe left/right to navigate months
- Pull down to refresh (re-read from synced database)
- Tap a card to expand full content
- Life event boundaries shown as colored dividers

### 2. Search

Full-text and semantic search across all documents, facts, and insights.

```
+----------------------------------+
|  Search                          |
|  [________________________] [mic]|
|                                  |
|  Recent: philosophy, childhood   |
|                                  |
|  Results for "career change":    |
|                                  |
|  +----- Document -------------+  |
|  | Journal, Jan 2026          |  |
|  | "...considering a shift..." |  |
|  | Relevance: 0.94            |  |
|  +----------------------------+  |
|                                  |
|  +----- Memory Fact ----------+  |
|  | "Wants to transition to    |  |
|  |  product management"       |  |
|  | Extracted: Feb 2026        |  |
|  +----------------------------+  |
|                                  |
|  [Timeline] [Search] [Ask] [Me] |
+----------------------------------+
```

- Voice input via platform speech recognition API
- Toggle between full-text and semantic (vector) search
- Filter by source platform, date range, or data type

### 3. Ask (RAG Chat)

Conversational interface for querying personal memory using RAG.

```
+----------------------------------+
|  Ask Memory Palace               |
|                                  |
|  +----------------------------+  |
|  | How has my view on remote  |  |
|  | work changed over the past |  |
|  | year?                      |  |
|  +----------------------------+  |
|                                  |
|  +----------------------------+  |
|  | Based on your journals and |  |
|  | messages, you initially    |  |
|  | loved remote work (Jan     |  |
|  | 2025) but started missing  |  |
|  | in-person collaboration by |  |
|  | mid-2025. By Dec 2025 you  |  |
|  | were advocating for hybrid.|  |
|  |                            |  |
|  | Sources: [Journal Jan 25]  |  |
|  | [Slack thread Jun 25]      |  |
|  | [Journal Dec 25]           |  |
|  +----------------------------+  |
|                                  |
|  [________________________] [>]  |
|  [Timeline] [Search] [Ask] [Me] |
+----------------------------------+
```

- Uses the same `rag_pipeline.rs` as desktop
- Cloud LLM only (Claude or OpenAI via user's API key)
- Sources are tappable, opening the full document
- Conversation history stored locally

### 4. Memory Browser

Browse extracted memory facts, entities, and themes.

```
+----------------------------------+
|  Memory Browser                  |
|  [Facts] [Entities] [Themes]     |
|                                  |
|  Latest Facts:                   |
|                                  |
|  * Values creative autonomy      |
|    (confidence: 0.91)            |
|    First seen: Jan 2025          |
|                                  |
|  * Practices meditation daily    |
|    (confidence: 0.87)            |
|    First seen: Mar 2025          |
|                                  |
|  * Skeptical of AI hype          |
|    (confidence: 0.73)            |
|    First seen: Nov 2025          |
|    Contradicts: "Excited about   |
|    AI potential" (Jun 2024)      |
|                                  |
|  [Timeline] [Search] [Ask] [Me] |
+----------------------------------+
```

- Three tabs: Facts, Entities, Themes
- Facts show evolution markers (when beliefs changed)
- Entities show relationship counts
- Themes show sentiment trends

### 5. Settings

```
+----------------------------------+
|  Settings                        |
|                                  |
|  Sync Status                     |
|  [*] Connected to Desktop        |
|  Last sync: 2 minutes ago        |
|  Documents: 1,247                |
|                                  |
|  LLM Provider                    |
|  [Claude (Anthropic)        v]   |
|  API Key: ****...7f2a            |
|  [Test Connection]               |
|                                  |
|  Appearance                      |
|  [Dark mode          toggle]     |
|                                  |
|  Storage                         |
|  Database: 142 MB                |
|  [Clear local cache]             |
|                                  |
|  About                           |
|  Version 1.0.0                   |
|  [Privacy Policy] [Licenses]     |
|                                  |
|  [Timeline] [Search] [Ask] [Me] |
+----------------------------------+
```

---

## Data Access

### Sync-Based (Primary)

The mobile app reads from a local SQLite database that is kept in sync with the desktop via the [Multi-Device Encrypted Sync](design-multi-device-sync.md) system:

1. Desktop produces encrypted sync events as documents are imported and analyzed.
2. Events propagate to the phone via Syncthing (background sync) or S3.
3. Mobile sync engine decrypts and applies events to the local SQLite database.
4. All reads are from the local database (fast, offline-capable).

### Read-Only Guarantees

The mobile app never writes to synced tables (documents, chunks, memory_facts, entities, insights). It only writes to:

- `mobile_config` -- local settings (LLM key, appearance)
- `ask_history` -- RAG conversation history (local only, not synced)
- `sync_state` -- vector clock and sync bookkeeping

This prevents the mobile app from creating conflicts in the sync system.

### Fallback: Read-Only Mode

If sync is not configured or the sync directory is unreachable:

- The app shows the last-known state of the database
- A banner indicates "Sync unavailable -- showing cached data"
- Ask/RAG still works if the user has an API key and network (reads from local DB, calls cloud LLM)
- No data loss; the app simply shows stale data until sync resumes

---

## Responsive Design Considerations

The React frontend is shared between desktop and mobile. Key adaptations:

### Layout Strategy

```
Desktop (>1024px)          Tablet (768-1024px)       Phone (<768px)
+-------+----------+      +-------+---------+       +-----------+
|       |          |      |       |         |       |           |
| Side  |  Main    |      | Side  |  Main   |       |   Main    |
| bar   |  Content |      | bar   |  Content|       |   Content |
|       |          |      | (col) |         |       |           |
|       |          |      |       |         |       |           |
+-------+----------+      +-------+---------+       +-----------+
                                                    | Tab Bar   |
                                                    +-----------+
```

### Breakpoint System

| Breakpoint | Width | Layout |
|-----------|-------|--------|
| `mobile` | < 640px | Single column, bottom tab bar, full-width cards |
| `tablet` | 640-1024px | Collapsible sidebar, two-column where useful |
| `desktop` | > 1024px | Full sidebar, multi-panel layouts |

### Touch Adaptations

- Minimum tap target: 44x44 points (Apple HIG) / 48x48 dp (Material)
- Swipe gestures for timeline navigation (left/right = month, up/down = scroll)
- Pull-to-refresh on all data views
- Long-press for context menus (instead of right-click)
- No hover states; use tap for all interactions

### Typography

- Base font size: 16px on mobile (prevents iOS zoom on input focus)
- Line height: 1.5 for readability on small screens
- Document content uses a serif font for comfortable reading
- Truncate long content with "Read more" expansion

### Performance

- Virtual scrolling for timeline and search results (only render visible items)
- Image thumbnails lazy-loaded
- SQLite queries use the same indexes as desktop; mobile adds no extra indexes
- Target: first meaningful paint < 500ms, search results < 200ms

---

## Offline Mode

The mobile app is fully functional offline for read operations:

| Feature | Online | Offline |
|---------|--------|---------|
| Timeline browsing | Yes | Yes (cached data) |
| Full-text search | Yes | Yes (local SQLite FTS) |
| Semantic search | Yes | Yes (local vector store) |
| Ask (RAG) | Yes | No (needs cloud LLM) |
| Memory Browser | Yes | Yes (local data) |
| Sync | Yes | Queued (resumes on connect) |

### Offline-First Architecture

```
User action --> Local SQLite (always) --> UI update (immediate)
                     |
                     +--> Sync engine (when online) --> Desktop
```

All data is local. The network is only needed for:
1. Cloud LLM calls (Ask feature)
2. Sync transport (Syncthing or S3)

### Caching Strategy

- The full SQLite database is on-device (no partial sync in v1.0)
- Ask conversation history is stored locally
- No CDN or remote asset dependencies
- App is fully functional after first sync, even if network is permanently lost

---

## Push Notifications for New Insights

When the desktop completes an analysis run and syncs new insights to the phone, the mobile app can surface them as notifications.

### Notification Types

| Type | Priority | Example |
|------|----------|---------|
| New insight | Normal | "New insight: Your interest in philosophy grew 40% this quarter" |
| Belief change detected | High | "Evolution detected: Your view on remote work has shifted" |
| Milestone | Normal | "You've imported 1,000 documents into Memory Palace" |
| Sync complete | Low (silent) | Badge update only |

### Implementation

```
Sync engine receives new insight events
  |
  v
Check notification preferences (user can disable per type)
  |
  v
iOS: UNUserNotificationCenter.add(request)
Android: NotificationManager.notify(id, notification)
  |
  v
Tap notification --> deep link to relevant insight/document
```

### Deep Linking

Notifications include a deep link to the relevant content:

```
mempalace://insight/uuid-here
mempalace://document/uuid-here
mempalace://timeline/2026-03
```

The app's router handles these URIs and navigates to the appropriate view.

### Notification Preferences (Settings)

- Toggle notifications on/off globally
- Per-type toggles (insights, belief changes, milestones)
- Quiet hours setting
- Badge count shows unread insights since last app open

---

## App Store Distribution

### iOS (App Store)

| Aspect | Detail |
|--------|--------|
| Minimum iOS | 16.0 (Tauri 2.0 requirement) |
| Review category | Productivity / Lifestyle |
| Privacy label | Data stored on-device only. Optional cloud LLM calls use user's own API key. No analytics, no tracking, no advertising. |
| App size | ~20 MB (estimated) |
| In-app purchases | None |
| Signing | Standard Apple Developer account ($99/year) |

**App Review considerations:**
- The app stores data locally and makes no undisclosed network calls.
- Cloud LLM calls are user-initiated and use the user's own API key (no proxy server).
- No account creation required. No server infrastructure.
- The sync feature uses Syncthing (user-installed) or user-provided S3. Apple reviewers may test without sync (read-only mode with sample data).

### Android (Google Play)

| Aspect | Detail |
|--------|--------|
| Minimum API | 26 (Android 8.0, Tauri 2.0 requirement) |
| Category | Productivity |
| Data safety | All data on-device. No data shared with third parties. Optional cloud API calls initiated by user. |
| App size | ~25 MB (estimated, includes native libraries) |
| In-app purchases | None |
| Signing | Google Play App Signing |

### Alternative Distribution

- **F-Droid:** Full open-source distribution. No Google Play Services dependency.
- **Direct APK:** Available from GitHub releases for sideloading.
- **TestFlight (iOS):** Beta distribution for early testers.

### CI/CD Pipeline

```
GitHub Actions
  |
  +---> Build iOS (macOS runner)
  |       |
  |       +--> cargo build --target aarch64-apple-ios --features mobile
  |       +--> tauri ios build
  |       +--> Upload to App Store Connect (Fastlane)
  |
  +---> Build Android (Ubuntu runner)
          |
          +--> cargo build --target aarch64-linux-android --features mobile
          +--> tauri android build
          +--> Upload to Google Play (Fastlane)
```

---

## Key Files (Future)

```
src-tauri/
  tauri.conf.json           -- add mobile configuration
  Cargo.toml                -- add mobile feature flag
  src/
    mobile.rs               -- mobile-specific Tauri commands (if any)

src/
  components/
    mobile/
      BottomTabBar.tsx      -- mobile navigation
      TimelineCard.tsx      -- compact timeline entry
      AskInput.tsx          -- chat input with voice button
  hooks/
    useResponsive.ts        -- breakpoint detection
    useSwipeNavigation.ts   -- swipe gesture handling
  layouts/
    MobileLayout.tsx        -- mobile shell with tab bar
    DesktopLayout.tsx       -- desktop shell with sidebar
    ResponsiveLayout.tsx    -- switches between mobile/desktop
```

---

## Implementation Estimates

| Task | Effort | Dependencies |
|------|--------|-------------|
| Cargo feature flags (mobile vs. desktop) | 0.5 weeks | None |
| Tauri mobile configuration + build setup | 1 week | None |
| Responsive layout system (breakpoints, tab bar) | 2 weeks | None |
| Timeline mobile view | 1.5 weeks | Layout system |
| Search mobile view (+ voice input) | 1.5 weeks | Layout system |
| Ask/RAG mobile view | 1 week | Layout system |
| Memory Browser mobile view | 1 week | Layout system |
| Mobile Settings view (sync status, LLM config) | 1 week | Layout system |
| Touch gesture handling (swipe, pull-to-refresh) | 1 week | Views |
| Push notifications (iOS + Android) | 1.5 weeks | Sync engine |
| Deep linking | 0.5 weeks | Views, Notifications |
| iOS build + App Store submission | 1 week | All above |
| Android build + Play Store submission | 1 week | All above |
| Beta testing (TestFlight + internal track) | 2 weeks | Builds |
| **Total** | **~16 weeks** | |

Note: This estimate assumes the Multi-Device Sync feature is already implemented. The mobile app depends on sync for data access.

---

## Open Questions

1. **Selective sync for mobile:** Should the mobile app sync the full database, or only recent data (e.g., last 2 years)? Full sync is simpler but could use significant storage on older phones.
2. **Local embeddings on mobile:** Modern phones can run small embedding models. Should we support local semantic search without vector sync, generating embeddings on-device? This would reduce sync payload but use CPU/battery.
3. **Widget support:** iOS Widgets and Android Widgets for "On this day" memories or daily insights. Worth the effort for v1.0, or defer to v1.1?
4. **Tablet-specific layouts:** Should iPad/Android tablet get a dedicated two-pane layout, or use the responsive breakpoint system?
5. **Apple Watch / Wear OS:** A "memory of the day" complication. Extremely limited scope but high engagement value. Defer to v1.2+.
