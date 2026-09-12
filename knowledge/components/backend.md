---
type: Component
title: Backend Component
description: Technical details of the Rust backend, including the hexagonal architecture rules.
tags: [backend, rust, axum, sqlx]
status: stable
generated: { by: reference_agent/gemini-3.1-pro, at: 2026-09-12T16:45:00Z }
sources:
  - id: contributing
    resource: /CONTRIBUTING.md
    title: Indelible Contributing Guide
---

# Backend Overview

The backend is written in Rust and utilizes a Cargo workspace located in the `backend/` directory. It is divided into applications (`apps/`) and library crates (`crates/`) following a strict hexagonal architecture.

## Key Libraries

- **HTTP Server**: `axum`
- **Database**: `sqlx` (PostgreSQL)
- **Async Runtime**: `tokio`
- **Background Jobs**: `apalis`
- **OpenAPI**: `utoipa`
- **Object Storage**: `aws-sdk-s3`

## Workspace Structure

### Apps
- `ind-api`: The main HTTP server providing the REST API and serving the web frontend.
- `ind-worker`: The background job processor (handles content fetching, indexing, AI).
- `ind-renderer`: A microservice wrapper around headless Chromium for rendering difficult web pages.

### Crates (Hexagonal Architecture)
The domain logic is split across several crates to enforce architectural boundaries:
- `ind-domain`: Pure business logic and core types. No IO, no SQL, no HTTP types.
- `ind-application`: Application services and use-case orchestrators. Defines ports (traits) for external dependencies.
- `ind-persistence`: Implements repository traits using SQL (SQLx). This is the *only* place SQL queries are allowed.
- `ind-auth`: Security-critical crypto (HMAC, AEAD, key derivation).
- `ind-search`, `ind-ai`, `ind-html`, `ind-egress`, `ind-ingest`: Various adapters and utilities.
- `ind-http-api`: Contains HTTP handlers and routes.

## Architecture Rules (Enforced)

According to `CONTRIBUTING.md`, the following rules block merges if violated:

1. **Purity**: `ind-domain` and `ind-application` stay pure. No `std::env::var`, no SQL, no direct filesystem or network I/O.
2. **Persistence**: SQL lives only in `ind-persistence`. Modifications should be targeted column updates, not full-row `update()` calls.
3. **Crypto**: Security-critical cryptography lives only in `ind-auth`.
4. **Configuration**: Environment reads live only in `apps/*/src/config*`. Library crates must take a typed config struct.
5. **DTOs**: Data Transfer Objects live in `routes/<domain>/dto.rs`, never in `state.rs` or directly in a handler.
6. **Object Storage**: Presigned object-store URLs never leave `routes/asset_proxy.rs`. Response bodies always carry API-origin asset URLs to ensure they are reachable from the browser.
7. **File Size**: Non-generated, non-test source files must stay at or under 600 lines.
