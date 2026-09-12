---
type: Component
title: Web Frontend Component
description: Technical details of the web application built with SvelteKit.
tags: [frontend, web, sveltekit, vite]
status: stable
generated: { by: reference_agent/gemini-3.1-pro, at: 2026-09-12T16:45:00Z }
sources:
  - id: web-pkg
    resource: /web/package.json
    title: Web Package JSON
---

# Web Frontend Overview

The web client for Indelible is a single-page application (SPA) built using modern web technologies. It is located in the `web/` directory.

## Tech Stack

- **Framework**: SvelteKit (`@sveltejs/kit` v2) / Svelte v5
- **Bundler**: Vite
- **Styling**: Tailwind CSS (v4)
- **Language**: TypeScript
- **Testing**: Vitest (Unit) and Playwright (E2E)

## Architecture & Tooling

- **Static Generation**: It uses `@sveltejs/adapter-static` for building the production assets, which are then served statically by the Rust backend (`ind-api`).
- **API Client**: The application uses `@hey-api/openapi-ts` to automatically generate a strongly-typed TypeScript API client based on the OpenAPI spec exported by the backend. This generation can be triggered via `pnpm run api:generate`.
- **Formatting & Linting**: Enforced via `eslint`, `svelte-check`, and `prettier`.

## Core Libraries

- `pdfjs-dist`: For rendering and reading PDFs natively in the browser.
- `marked`: For Markdown parsing.
- `dompurify`: For sanitizing rendered HTML to prevent XSS.
- `svelte-i18n`: For internationalization and localization.
