# Multi-Device Encrypted Sync Design (v1.0)

**Status:** Draft
**Last updated:** 2026-03-26
**Target release:** v1.0

---

## Overview

Enable secure synchronization between Memory Palace instances across devices (desktop-to-desktop, desktop-to-mobile) without any cloud service dependency. All sync data is end-to-end encrypted. The user's memory data never leaves their control in plaintext.

### Goals

- Zero-knowledge sync: no server ever sees plaintext data
- Works peer-to-peer (Syncthing) or via user-provided S3-compatible storage
- Eventual consistency with deterministic conflict resolution
- Works offline; syncs when connectivity resumes

### Non-Goals

- Real-time collaborative editing (this is a personal tool)
- Hosted sync service operated by Memory Palace team
- Sync of application settings beyond user preferences (window size, etc.)

---

## Architecture

```
+-------------------+                      +-------------------+
|   Device A        |                      |   Device B        |
|   (Desktop)       |                      |   (Desktop/Mobile)|
|                   |                      |                   |
| +---------------+ |    Encrypted Sync    | +---------------+ |
| | SQLite DB     | |    Directory (shared)| | SQLite DB     | |
| +-------+-------+ |                      | +-------+-------+ |
|         |         |                      |         |         |
| +-------v-------+ |  +----------------+ | +-------v-------+ |
| | Sync Engine   +---->| sync/          +<--+ Sync Engine   | |
| | (write events)| |  | deviceA/*.enc  | | | (read events) | |
| +---------------+ |  | deviceB/*.enc  | | +---------------+ |
|                   |  +----------------+ |                   |
+-------------------+  (Syncthing / S3)   +-------------------+
```

### Sync Protocol

- **Transport:** Syncthing (peer-to-peer, no central server) or any S3-compatible object storage (MinIO, Backblaze B2, AWS S3). The sync layer is transport-agnostic; it only reads and writes files in a directory.
- **Format:** Append-only encrypted change log files in a shared sync directory.
- **Encryption:** NaCl box (X25519 + XSalsa20-Poly1305) per sync file.
- **Key derivation:** Argon2id from user's sync passphrase produces a 256-bit group key. Each device derives a sub-key for signing.

### Change Log Format

Each database write operation produces a sync event. Events are serialized as JSON, then encrypted.

```json
{
  "event_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2026-03-26T14:30:00Z",
  "device_id": "a1b2c3d4-...",
  "sequence": 1042,
  "operation": "insert",
  "table": "documents",
  "row_id": "doc-uuid-here",
  "data": {
    "title": "Journal Entry",
    "content": "...",
    "content_hash": "sha256:abc123...",
    "source_platform": "obsidian",
    "created_at": "2026-03-26T14:00:00Z"
  },
  "checksum": "sha256:def456..."
}
```

Events are batched by time window and written as encrypted files:

```
sync/
  manifest.enc                          -- encrypted device registry
  a1b2c3d4/                             -- Device A's outbox
    2026-03-26T14-00-00_00001042.enc    -- batch file (timestamp + sequence)
    2026-03-26T14-30-00_00001085.enc
  b5c6d7e8/                             -- Device B's outbox
    2026-03-26T15-00-00_00000001.enc    -- Device B's first sync
```

### Encryption Scheme

```
User passphrase
      |
      v
  Argon2id (salt = device_group_id, t=3, m=256MB, p=4)
      |
      v
  group_key (256-bit)
      |
      +---> file_key = HKDF(group_key, file_path)
      |         |
      |         v
      |     NaCl secretbox(file_key, nonce, plaintext) --> .enc file
      |
      +---> signing_key = HKDF(group_key, device_id)
                |
                v
            Ed25519 sign each event batch (integrity + device attribution)
```

**Key properties:**
- Each file is encrypted with a unique derived key (prevents cross-file analysis)
- Each device signs its events (detect tampering if sync directory is compromised)
- The group key never leaves the device; only derived file keys and signatures are used
- Passphrase can be changed by re-encrypting the manifest and re-keying future events (old events remain readable with old key stored in manifest)

---

## Conflict Resolution

| Data Type | Strategy | Rationale |
|-----------|----------|-----------|
| Documents | Deduplicate by `content_hash` | Same content = same document regardless of source device |
| Chunks | Follow parent document | Chunks are derived from documents; re-chunk if parent changes |
| Memory Facts | Union + timestamp ordering | Facts accumulate over time; contradictions tracked as evolution |
| Entities | Merge by canonical name | Same entity name across devices maps to one node in the graph |
| Entity Relationships | Union | Relationships are additive; duplicates collapsed |
| Insights | Union (no dedup) | Each device may generate unique analysis insights |
| Config/Settings | Last-writer-wins by timestamp | User preferences; latest change takes precedence |
| Boundaries | Merge by name + date | Same life event boundary is one boundary |

### Merge Algorithm

When Device B processes events from Device A:

```
for each event in batch:
    match event.operation:
        Insert:
            if row_id exists locally:
                if content_hash matches: skip (already have it)
                else: apply conflict strategy for table type
            else:
                insert row
        Update:
            if local row.updated_at > event.timestamp: skip (local is newer)
            else: apply update
        Delete:
            if row exists: mark as deleted (soft delete with tombstone)
            else: record tombstone for future
```

### Vector Clock

Each device maintains a vector clock tracking the last-seen sequence number from every other device:

```json
{
  "device_a": 1085,
  "device_b": 523
}
```

On sync, a device only reads event files with sequence numbers greater than its last-seen value. This enables efficient incremental sync.

---

## Device Pairing Flow

```
Device A                                          Device B
   |                                                  |
   |  1. Generate group_key from passphrase           |
   |  2. Encode as 6-word BIP39 mnemonic              |
   |  3. Display pairing code                         |
   |                                                  |
   |  ---- User reads code aloud or types it ---->    |
   |                                                  |
   |                    4. User enters pairing code    |
   |                    5. Derive same group_key       |
   |                    6. Generate device_id          |
   |                                                  |
   |  7. Configure sync directory                     |
   |     (Syncthing folder ID or S3 bucket)           |
   |                                                  |
   |  8. Device A writes full export to sync/         |
   |                                                  |
   |                    9. Device B reads full export  |
   |                   10. Imports all data            |
   |                   11. Begins bidirectional sync   |
   |                                                  |
   v                                                  v
```

### Pairing Code Format

The 6-word BIP39 mnemonic encodes 66 bits: 32-bit group ID + 34-bit key material seed. Combined with the user's passphrase via Argon2id, this produces the full group key. Example:

```
abandon bicycle clock dolphin envelope frost
```

The code is one-time-use for pairing. After both devices have the group key, the mnemonic is no longer needed.

---

## Incremental Sync

### Write Path (Producing Events)

1. Application performs a database write (insert/update/delete).
2. The `SyncEngine` intercepts the write via a SQLite trigger or Rust-level hook.
3. The event is added to an in-memory batch buffer.
4. Every 60 seconds (configurable), or when the buffer exceeds 1000 events, the batch is:
   a. Serialized to JSON
   b. Signed with the device's signing key
   c. Encrypted with the file-specific key
   d. Written to `sync/<device_id>/<timestamp>_<sequence>.enc`
5. Syncthing or S3 sync picks up the new file and propagates it.

### Read Path (Consuming Events)

1. The `SyncEngine` polls the sync directory every 30 seconds for new `.enc` files from other devices.
2. For each new file (sequence > last-seen for that device):
   a. Decrypt with derived file key
   b. Verify signature
   c. Deserialize event batch
   d. Apply each event using the merge algorithm
   e. Update the vector clock
3. Processed files are recorded in a local `sync_state` table to avoid reprocessing.

### Bandwidth Optimization

- **Batching:** Events are grouped into files (not one file per event). Typical batch: 1-100 KB.
- **Compression:** Event batches are zstd-compressed before encryption. Journal text compresses 3-5x.
- **Deferred large fields:** Document `content` is stored separately from metadata events. Devices can sync metadata first, then fetch content on demand.
- **Pruning:** After all devices have acknowledged a sequence (via their vector clocks in the manifest), old event files can be archived or deleted.
- **Initial sync optimization:** The first full export uses a compact binary format (SQLite dump, encrypted) rather than replaying all individual events.

---

## Conflict UI

When automatic resolution is insufficient (rare), the user sees a conflict notification:

```
+----------------------------------------------------------+
|  Sync Conflict                                      [x]  |
|                                                          |
|  Document "2026-03 Reflections" was modified on both     |
|  devices since last sync.                                |
|                                                          |
|  Device A (Desktop):        Device B (Laptop):           |
|  Modified: Mar 26, 14:30    Modified: Mar 26, 15:00      |
|  +320 chars added           Title changed                |
|                                                          |
|  [ Keep Desktop ] [ Keep Laptop ] [ Keep Both ] [ Diff ] |
+----------------------------------------------------------+
```

Conflict resolution rules:

- **Automatic (no UI):** Content-hash dedup, union merges, last-writer-wins for config. These cover ~99% of cases.
- **Manual (UI prompt):** Only shown when the same document is edited on two devices between syncs and content hashes differ. The user can keep either version, keep both as separate documents, or view a diff.
- **Conflict log:** All resolutions (automatic and manual) are logged to `sync_conflicts` table for auditability.

---

## Offline Handling

Memory Palace is offline-first by design. Sync is purely additive:

1. **No connectivity:** The app works normally. Events accumulate in the local outbox (`sync/<device_id>/`).
2. **Reconnect:** When the sync directory becomes available again (Syncthing reconnects, S3 becomes reachable), all pending events are exchanged.
3. **Extended offline:** If a device is offline for weeks, it may have many events to process. The sync engine processes them in chronological order with progress indication.
4. **Permanent disconnect:** A device can be removed from the sync group via Settings. Its outbox is ignored; its device ID is removed from the manifest.

### Data Integrity During Offline Periods

- Each event includes a SHA-256 checksum of its data payload.
- Event batches are signed by the producing device.
- The vector clock ensures no events are missed or double-applied.
- Tombstones for deleted records are retained for 90 days to ensure propagation.

---

## Security Threat Model

### Assets Protected

- User's personal memory data (documents, facts, insights, entities)
- Sync passphrase / group key

### Threat Scenarios

| Threat | Mitigation |
|--------|-----------|
| **Sync directory compromised** (attacker reads S3 bucket) | All files encrypted with NaCl. Attacker sees opaque blobs. No plaintext metadata in filenames (timestamps are device-local sequence numbers). |
| **Sync directory tampered** (attacker modifies files) | Each event batch is Ed25519-signed by the producing device. Tampered files fail signature verification and are rejected. |
| **Weak passphrase** | Argon2id with high memory cost (256 MB) makes brute-force expensive. UI warns if passphrase entropy is low. Minimum 6 words or 20 characters enforced. |
| **Device stolen** | SQLite database is encrypted at rest (via SQLCipher or application-level encryption). Sync directory on device is also encrypted. Device can be remotely removed from the sync group. |
| **Key compromise** | Passphrase can be rotated. New events use new key. Old events are re-encrypted in a background migration. Other devices receive the new key via a key-rotation event encrypted with the old key. |
| **Replay attack** (old events replayed) | Vector clock + sequence numbers detect replayed events. Each event_id is unique and recorded. |
| **Man-in-the-middle** (during pairing) | Pairing code is read aloud or entered physically. No network channel for pairing. Syncthing uses TLS for transport. |

### What We Explicitly Do Not Protect Against

- A compromised device with the app running (attacker has full access to decrypted data in memory)
- User sharing their passphrase
- Keyloggers capturing the passphrase during entry

---

## Key Files (Future)

```
src-tauri/src/sync/
  mod.rs              -- SyncEngine: orchestrates read/write paths
  crypto.rs           -- Encryption, key derivation, signing
  events.rs           -- Event serialization, batching
  merge.rs            -- Conflict resolution and merge algorithm
  clock.rs            -- Vector clock implementation
  transport.rs        -- Transport abstraction (filesystem-based)
  pairing.rs          -- Device pairing flow, BIP39 encoding

src-tauri/src/domain/ports/
  sync_engine.rs      -- ISyncEngine trait (port)
```

---

## Implementation Estimates

| Task | Effort | Dependencies |
|------|--------|-------------|
| Encryption primitives (NaCl, Argon2id, HKDF) | 1 week | None |
| Event model + serialization + batching | 1 week | None |
| Vector clock implementation | 0.5 weeks | None |
| Sync write path (intercept writes, produce events) | 2 weeks | Events, Crypto |
| Sync read path (consume events, merge) | 2 weeks | Events, Crypto, Vector clock |
| Conflict resolution engine | 1.5 weeks | Merge algorithm |
| Device pairing flow + BIP39 | 1 week | Crypto |
| Sync Settings UI (pair, unpair, status, conflicts) | 1.5 weeks | Pairing |
| Conflict resolution UI | 1 week | Conflict engine |
| S3-compatible transport adapter | 1 week | Write/read paths |
| Syncthing integration guide + testing | 1 week | Write/read paths |
| Initial full-export sync optimization | 1 week | All above |
| Security audit + penetration testing | 1.5 weeks | All above |
| **Total** | **~16 weeks** | |

---

## Open Questions

1. **SQLCipher vs. application-level DB encryption:** SQLCipher encrypts the entire database transparently but adds a native dependency. Application-level encryption is more portable but requires careful implementation. Decision needed before v1.0.
2. **Syncthing bundling:** Should the app bundle Syncthing, or require the user to install it separately? Bundling simplifies UX but increases app size and update complexity.
3. **Maximum sync group size:** How many devices should be supported? 2-5 devices is the expected use case, but the vector clock scales linearly with device count.
4. **Tombstone retention:** 90 days is proposed. Should this be configurable? Longer retention increases sync directory size but prevents data resurrection.
5. **Selective sync:** Should users be able to sync only certain date ranges or source platforms? This adds complexity but could be important for mobile devices with limited storage.
