---
type: Component
title: Mobile Components
description: Technical details of the Mobile apps (Kotlin Multiplatform).
tags: [mobile, kmp]
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
