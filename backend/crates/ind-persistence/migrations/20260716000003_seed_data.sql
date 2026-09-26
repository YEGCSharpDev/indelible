-- YouTube resolves through YouTube's own Atom feeds; RSSHub is only a
-- fallback. Public RSSHub instances rot, so none ship enabled: the official
-- instance is seeded disabled for operators who want to opt in (it
-- rate-limits anonymous use), and self-hosted instances can be added as rows
-- here or at runtime.
INSERT INTO public.feed_provider_instances (id, provider_type, base_url, priority, enabled, last_success_at, last_failure_at, consecutive_failures, created_at, updated_at) VALUES ('fd374d85-2cb4-4d63-9563-0b337a67a790', 'nitter', 'https://nitter.net', 10, true, NULL, NULL, 0, now(), now());
INSERT INTO public.feed_provider_instances (id, provider_type, base_url, priority, enabled, last_success_at, last_failure_at, consecutive_failures, created_at, updated_at) VALUES ('7c8a803e-8b0e-4c3b-a2a3-a820b15ac74e', 'rsshub', 'https://rsshub.app', 200, false, NULL, NULL, 0, now(), now());

INSERT INTO public.search_index_state (singleton, current_version, updated_at, target_version, cursor_created_at, cursor_document_id) VALUES (true, 0, now(), NULL, NULL, NULL);
