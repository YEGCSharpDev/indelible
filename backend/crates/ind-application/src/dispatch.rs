use ind_domain::ItemType;

/// Infer the library content type for a URL-bearing save.
pub fn infer_item_type_for_url(url: &str) -> ItemType {
    inferred_url_item_type(url).unwrap_or(ItemType::Article)
}

fn inferred_url_item_type(url: &str) -> Option<ItemType> {
    if is_youtube_url(url) {
        Some(ItemType::Video)
    } else {
        None
    }
}

/// Returns true when `url` identifies a supported YouTube video.
///
/// Recognised forms:
/// - `https://www.youtube.com/watch?v=...`
/// - `https://m.youtube.com/watch?v=...`
/// - `https://music.youtube.com/watch?v=...`
/// - `https://youtube.com/watch?v=...`
/// - `https://youtube.com/shorts/...`
/// - `https://youtu.be/...`
pub fn is_youtube_url(url: &str) -> bool {
    ind_domain::youtube_video_id(url).is_some()
}

/// Extract the stable video ID from a supported YouTube URL.
pub fn extract_youtube_video_id(url: &str) -> Option<String> {
    ind_domain::youtube_video_id(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn youtube_shorts_use_the_video_dispatch_path() {
        let url = "https://www.youtube.com/shorts/abc123?feature=share";

        assert!(is_youtube_url(url));
        assert_eq!(extract_youtube_video_id(url).as_deref(), Some("abc123"));
        assert_eq!(infer_item_type_for_url(url), ItemType::Video);
    }
}
