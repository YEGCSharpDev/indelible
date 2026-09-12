---
type: Component
title: Mobile & Extension Components
description: Technical details of the Mobile apps (Kotlin Multiplatform) and Browser Extension (WXT).
tags: [mobile, extension, kmp, wxt]
status: stable
generated: { by: reference_agent/gemini-3.1-pro, at: 2026-09-12T16:45:00Z }
---

# Mobile Application (`mobile/`)

The mobile client targets both Android and iOS from a single shared codebase using Kotlin Multiplatform (KMP).

## Tech Stack

- **Core**: Kotlin Multiplatform
- **UI Framework**: Compose Multiplatform (allows sharing UI code across Android and iOS)
- **Build System**: Gradle (`build.gradle.kts`)

The mobile app shares the library state and reading positions with the web app by consuming the same central API.

# Browser Extension (`extension/`)

The browser extension allows users to save the page they are currently on and archives its full text. It injects a toolbar over the article for highlighting and taking notes.

## Tech Stack

- **Framework**: WXT (Web Extension Tooling)
- **Language**: TypeScript
- **Styling / UI**: Standard web tooling configured via `wxt.config.ts`.

## Capabilities

- **Inline Highlighting**: Text marked on the live page shows up in the reader, and highlights made in the reader are re-anchored onto the original page when reopened.
- **In-place Archiving**: Archives the full text of the page at save time without leaving the tab.
