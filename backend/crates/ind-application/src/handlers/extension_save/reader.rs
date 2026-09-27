use bytes::Bytes;

use crate::content_hash::compute_content_hash;
use crate::error::AppError;
use crate::handlers::provided_content::stage_provided_content;
use crate::repos::document_lifecycle::{
    MaterializeIdentity, MaterializeSideEffects, SaveSideEffectsFn, SaveToLibraryRequest,
};
use ind_domain::{ArchiveAssetKind, ContentSource, UserId};

use super::utils::resolved_canonical_url;
use super::{ExtensionSaveService, ReaderSaveInput, SaveResult};

impl ExtensionSaveService {
    /// Provided-content save: the browser already extracted readable HTML. The save does NOT
    /// enable content-gated AI (no redundant server render); instead the readable HTML is attached
    /// as a document-keyed asset and the embed + search reindex are enqueued once it exists.
    pub async fn reader_save(
        &self,
        user_id: UserId,
        input: ReaderSaveInput,
    ) -> Result<SaveResult, AppError> {
        super::utils::validate_lead_image_url(&input.lead_image_url)?;
        let canonical_url = resolved_canonical_url(&input.url, input.canonical_url.as_deref());
        let content_hash = Some(compute_content_hash(&input.reader_html));
        let document = Self::build_url_document(
            user_id,
            &input.url,
            canonical_url,
            input.title,
            input.author,
            input.excerpt,
            input.language,
            input.lead_image_url,
            None,
            content_hash,
            input.item_type,
        );

        let staged_readable = stage_provided_content(
            &self.object_storage,
            user_id,
            ArchiveAssetKind::ReadableHtml,
            "text/html",
            Bytes::from(input.reader_html),
        )
        .await?;

        let staged = staged_readable.clone();
        let side_effects: Option<SaveSideEffectsFn> =
            Some(Box::new(move |ctx| MaterializeSideEffects {
                events: Vec::new(),
                outbox: vec![staged.outbox(ctx.document.id, user_id)],
            }));

        let outcome = self
            .lifecycle
            .save_to_library(SaveToLibraryRequest {
                identity: MaterializeIdentity::Url {
                    document,
                    origin: None,
                },
                source: ContentSource::Extension,
                source_delivery_id: None,
                hide_deliveries: true,
                enqueue_engaged_ai: false,
                restore_policy: Default::default(),
                side_effects,
            })
            .await?;

        self.attach_staged_document_asset(outcome.document.id, &staged_readable)
            .await?;

        Ok(Self::save_result(&outcome))
    }
}
