<img alt="An archived article open in the Indelible reader, with Mila's summary, extracted metadata and reading progress in the detail panel beside it" src="https://assets.useindelible.com/readme/reader.webp" width="100%" />

# Indelible

> [!NOTE]
> This is a trimmed fork of the original Indelible repository. The marketing website, browser extension, and Obsidian plugin components have been removed to focus entirely on the core application, mobile apps, and Docker deployment logic.

Open-source, self-hosted read-it-later and knowledge archiver. Articles,
newsletters, PDFs, and EPUBs are captured in full and stored in your own
library, permanently. Links rot; your library does not.

## What it does

- **Full-content archiving.** Pages are fetched, extracted, and stored at save
  time, with a headless Chromium renderer for the hard ones. The original can
  disappear; your copy stays readable.
- **A focused reader.** Clean typography, highlights, notes, and reading
  progress that syncs across web and mobile.
- **Search that understands you.** Full-text and semantic search across
  everything you have saved.
- **Mila, an optional AI assistant.** Ask questions across your library and
  summarize long reads, using your own provider key.
- **Content in, from anywhere.** Personal email-in
  addresses, RSS feeds, and file uploads.
- **Your tools, connected.** Sync to Obsidian.

<img alt="Four Indelible screens: the Home dashboard, full-text and semantic search, Collections, and the RSS feed" src="https://assets.useindelible.com/readme/wall.webp" width="100%" />

Mila is optional and runs on your own provider key, against any
OpenAI-compatible endpoint. It summarizes, tags, and answers questions across
the library.

<img alt="Indelible's Mila settings, showing the provider toggle, library indexing progress, and the editable summary and tag prompt presets" src="https://assets.useindelible.com/readme/mila.webp" width="100%" />

## Quick start

You need Docker and a machine with a few GB of RAM.

```bash
mkdir indelible && cd indelible && curl -fsSLO https://github.com/useindelible/indelible/releases/latest/download/install.sh && sh install.sh
```

The installer checks the downloads against the release checksums, generates
the required secrets, and brings up PostgreSQL, Silo, the renderer, the worker,
and the API, which serves the web interface on the same port. Open
http://localhost:38473 and create the first account. Compose pulls the release
images from GHCR; it does not build Indelible on your machine.

This quickstart runs on localhost without TLS. For a deployment reachable by
other people, follow the
[installation guide](https://useindelible.com/docs/self-hosting/install/) and
[security checklist](https://useindelible.com/docs/self-hosting/security/).

## Development

Contributors build the images from the checked-out source using the root
Compose file:

```bash
git clone https://github.com/useindelible/indelible.git
cd indelible
docker compose up -d --build
```

## Clients

| Client | Where |
| --- | --- |
| Web | Served by `ind-api`, no separate deployment |
| Mobile | `mobile/` (Android and iOS, Kotlin Multiplatform) |

Android and iOS are one Kotlin Multiplatform codebase, sharing the library and
reading position with the web.

<img alt="Three Indelible phone screens: the daily home view, an article being highlighted with the native selection toolbar, and saving a URL" src="https://assets.useindelible.com/readme/phones.webp" width="100%" />

## Documentation

Full documentation, including every configuration variable, lives at
[useindelible.com](https://useindelible.com).

## Contributing

Start with [CONTRIBUTING.md](CONTRIBUTING.md), which covers the development
setup, the checks CI runs, and the architecture boundaries backend changes are
reviewed against.

## Licence

[AGPL-3.0](LICENSE).
