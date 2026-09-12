---
type: Playbook
title: Development & Contribution Workflow
description: Guide for setting up the local development environment and contributing to the repository.
tags: [development, setup, contributing]
status: stable
generated: { by: reference_agent/gemini-3.1-pro, at: 2026-09-12T16:45:00Z }
sources:
  - id: contributing
    resource: /CONTRIBUTING.md
    title: Indelible Contributing Guide
---

# Development Setup

Indelible uses a mix of Docker for dependencies and native tooling for development.

## 1. Start Dependencies

Use Docker Compose to bring up the database, object storage (Silo), and initialization scripts.

```bash
docker compose up -d postgres silo silo-init
```

## 2. Configure Environment

The backend expects a `DATABASE_URL` and other environment variables.
Copy the example environment file:
```bash
cp .env.example .env
```
*(Adjust the variables in `.env` as needed. Migrations will run automatically at startup).*

## 3. Run the Backend API

In a new terminal, start the main Rust API server:
```bash
cd backend
cargo run -p ind-api
```
This runs the API on port `:38473`.

## 4. Run the Web Frontend

In another terminal, install dependencies and start the Vite development server:
```bash
cd web
pnpm install
pnpm dev
```
The web app will be available on port `:5173`.

# CI Checks & Testing

Before opening a pull request, ensure the following checks pass. These match what the CI pipeline runs:

| Component | Commands |
| --- | --- |
| **Backend** | `cargo fmt --check`<br>`cargo clippy --workspace --all-targets --all-features -- -D warnings`<br>`cargo test --workspace --all-features` |
| **Web** | `pnpm check`<br>`pnpm lint`<br>`pnpm format:check`<br>`pnpm test` |
| **Extension** | `npm run lint`<br>`npm run check`<br>`npm test` |
| **Mobile** | `./gradlew :composeApp:compileCommonMainKotlinMetadata`<br>`./gradlew :composeApp:jvmTest` |

*Note: Backend integration tests use real containers via the `ind-test-support` crate, so Docker must be running.*

# Pull Request Guidelines

- Branch from `main` using the format `<type>/<short-description>`.
- Use [Conventional Commits](https://www.conventionalcommits.org/) for PR titles (e.g., `feat:`, `fix:`, `docs:`).
- Keep one logical change per PR.
- Document what you ran to verify the change in the PR description.
