use ind_domain::ItemType;

/// Infer the library content type for a URL-bearing save.
pub fn infer_item_type_for_url(_url: &str) -> ItemType {
    ItemType::Article
}
