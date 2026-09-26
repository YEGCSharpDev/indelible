use std::collections::HashMap;

use ind_domain::DocumentId;

use crate::AppError;

#[async_trait::async_trait]
pub trait ExportSummaryProvider: Send + Sync {
    async fn summary_for_document(
        &self,
        document_id: DocumentId,
        excerpt: Option<&str>,
    ) -> Result<Option<String>, AppError>;

    async fn summaries_for_documents(
        &self,
        sources: &[DocumentSummarySource],
    ) -> Result<HashMap<DocumentId, Option<String>>, AppError> {
        let mut summaries = HashMap::with_capacity(sources.len());
        for source in sources {
            let summary = self
                .summary_for_document(source.document_id, source.excerpt.as_deref())
                .await?;
            summaries.insert(source.document_id, summary);
        }
        Ok(summaries)
    }
}

#[derive(Debug, Clone)]
pub struct DocumentSummarySource {
    pub document_id: DocumentId,
    pub excerpt: Option<String>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StoredExportSummaryProvider {}

impl StoredExportSummaryProvider {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl ExportSummaryProvider for StoredExportSummaryProvider {
    async fn summary_for_document(
        &self,
        _document_id: DocumentId,
        excerpt: Option<&str>,
    ) -> Result<Option<String>, AppError> {
        Ok(normalized_summary(excerpt))
    }

    async fn summaries_for_documents(
        &self,
        sources: &[DocumentSummarySource],
    ) -> Result<HashMap<DocumentId, Option<String>>, AppError> {
        let mut summaries = HashMap::with_capacity(sources.len());
        for source in sources {
            let summary = normalized_summary(source.excerpt.as_deref());
            summaries.insert(source.document_id, summary);
        }
        Ok(summaries)
    }
}

fn normalized_summary(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}
