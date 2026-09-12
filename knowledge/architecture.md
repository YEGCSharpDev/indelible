---
type: Architecture
title: Indelible Architecture Overview
description: High-level overview of the Indelible system architecture, deployment model, and tech stack.
tags: [architecture, system-design]
status: stable
generated: { by: reference_agent/gemini-3.1-pro, at: 2026-09-12T16:45:00Z }
sources:
  - id: readme
    resource: /README.md
    title: Indelible README
  - id: contributing
    resource: /CONTRIBUTING.md
    title: Indelible Contributing Guide
---

# System Overview

Indelible is an open-source, self-hosted read-it-later and knowledge archiver. It captures articles, newsletters, PDFs, and EPUBs in full, storing them permanently.

The system is designed for self-hosting via Docker and is composed of several moving parts that communicate with a central backend API.

## Core Services

When deployed, the system consists of the following components (as seen in `docker-compose.yml`):
- **PostgreSQL**: Relational database for storing user accounts, metadata, collections, and highlights.
- **Silo**: Object storage (S3-compatible) for storing the actual archived content (HTML, PDFs, EPUBs, images).
- **ind-api**: The main backend REST API that serves both the frontend web app and provides endpoints for clients (mobile, extension, Obsidian).
- **ind-worker**: A background worker process that handles asynchronous jobs (like fetching content, full-text indexing, calling the optional AI assistant).
- **ind-renderer**: A headless Chromium instance used by the worker to render dynamic pages that require JavaScript to load.

## Clients

Indelible has multiple clients that interact with the `ind-api`:
1. **Web App**: Served directly by `ind-api`.
2. **Browser Extension**: Injected into pages to capture content and sync highlights.
3. **Mobile Apps**: Android and iOS apps sharing a Kotlin Multiplatform codebase.
4. **Obsidian Plugin**: Syncs highlights and notes to a user's local Obsidian vault.

## Tech Stack Summary

| Component | Technology |
| --- | --- |
| Backend | Rust, Axum, SQLx, Tokio |
| Frontend | SvelteKit, Vite, Tailwind CSS |
| Mobile | Kotlin Multiplatform (KMP), Compose Multiplatform |
| Extension | WXT (Web Extension Tooling) |
| Database | PostgreSQL |
| Storage | S3 API |
