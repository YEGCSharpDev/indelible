use ind_domain::FeedProviderInstance;
use url::Url;
use uuid::Uuid;

/// One attempt URL for fetching a feed via a known provider instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderCandidate {
    pub url: String,
    pub provider_type: String,
    /// `None` when the URL was supplied by the user and does not map to any
    /// known instance — we still try it, but skip success/failure recording.
    pub instance_id: Option<Uuid>,
}

/// Build candidate YouTube RSSHub URLs for `rsshub_path`
/// (e.g. `/youtube/user/@JFlaMusic`). YouTube only flows through RSSHub today,
/// so non-rsshub instances are skipped silently.
pub fn youtube_candidates(
    rsshub_path: &str,
    instances: &[FeedProviderInstance],
) -> Vec<ProviderCandidate> {
    instances
        .iter()
        .filter(|inst| inst.provider_type == "rsshub")
        .map(|inst| ProviderCandidate {
            url: format!("{}{rsshub_path}", inst.base_url.trim_end_matches('/')),
            provider_type: inst.provider_type.clone(),
            instance_id: Some(inst.id),
        })
        .collect()
}

/// Find which provider instance a URL belongs to by origin + path-boundary match.
/// Among matching instances the longest base-url prefix wins so a more specific
/// `https://host/rss2` registration takes precedence over `https://host/rss`.
pub fn instance_for_url<'a>(
    instances: &'a [FeedProviderInstance],
    url: &str,
) -> Option<&'a FeedProviderInstance> {
    let parsed = Url::parse(url).ok()?;
    instances
        .iter()
        .filter_map(|inst| {
            let base = Url::parse(inst.base_url.trim_end_matches('/')).ok()?;
            if parsed.scheme() != base.scheme()
                || parsed.host_str() != base.host_str()
                || parsed.port_or_known_default() != base.port_or_known_default()
            {
                return None;
            }

            let base_path = base.path().trim_end_matches('/');
            let url_path = parsed.path();
            let path_matches = base_path.is_empty()
                || base_path == "/"
                || url_path == base_path
                || url_path
                    .strip_prefix(base_path)
                    .is_some_and(|rest| rest.starts_with('/'));
            path_matches.then_some((base.as_str().len(), inst))
        })
        .max_by_key(|(base_len, _)| *base_len)
        .map(|(_, inst)| inst)
}

/// Recover the RSSHub path from a canonical YouTube URL like
/// `https://www.youtube.com/@JFlaMusic` or `https://www.youtube.com/channel/UC123`.
pub fn youtube_rsshub_path_from_canonical(youtube_url: &str) -> Option<String> {
    let url = youtube_url.trim_end_matches('/');
    let path = url
        .find("youtube.com")
        .map(|i| &url[i + "youtube.com".len()..])?;
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    match segments.as_slice() {
        [handle, ..] if handle.starts_with('@') => Some(format!("/youtube/user/{handle}")),
        ["channel", id, ..] => Some(format!("/youtube/channel/{id}")),
        ["user", name, ..] => Some(format!("/youtube/user/{name}")),
        ["c", name, ..] => Some(format!("/youtube/c/{name}")),
        _ => None,
    }
}
