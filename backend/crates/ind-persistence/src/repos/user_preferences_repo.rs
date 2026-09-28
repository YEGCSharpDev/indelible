use ind_application::AppError;
use ind_application::repos::user_preferences::UserPreferencesRepository;
use ind_domain::{
    AccentColor, AiPreferenceSettings, AppearanceSettings, ArchivalProcessingSettings,
    ArchivalSettings, ArchiveFormatPreferences, DefaultView, DuplicateAction,
    DuplicateDetectionSettings, DuplicateSensitivity, LayoutSettings, ListDensity,
    PreferencesSettings, ProxySettings, ReaderFontFamily, ReaderFontSize, ReaderLineHeight,
    ReaderSettings, SidePanelMode, SidebarMode, TriageMode, UserId, WorkflowSettings,
};
use sqlx::{FromRow, PgPool};

pub struct PgUserPreferencesRepository {
    pool: PgPool,
}

impl PgUserPreferencesRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct UserPreferencesRow {
    accent_color: String,
    sidebar_mode: String,
    default_view: String,
    list_density: String,
    side_panel: String,
    triage_mode: String,
    auto_advance: bool,
    reader_font_family: String,
    reader_font_size: String,
    reader_line_height: String,
    ai_mila_enabled: bool,
    ai_custom_prompt: Option<String>,
    archival_monolith: bool,
    archival_pdf: bool,
    archival_screenshot: bool,
    archival_warc: bool,
    duplicate_detection_enabled: bool,
    duplicate_sensitivity: String,
    duplicate_action: String,
    browser_timeout_secs: i32,
    max_concurrent_archives: i32,
    ai_auto_processing: bool,
    proxy_url: Option<String>,
    proxy_all_requests: bool,
}

fn map_sqlx_error(err: sqlx::Error) -> AppError {
    super::map_sqlx_error("user_preferences", "user preferences already exist", err)
}

fn parse_sidebar_mode(value: &str) -> SidebarMode {
    match value {
        "collapsed" => SidebarMode::Collapsed,
        "auto" => SidebarMode::Auto,
        _ => SidebarMode::Expanded,
    }
}

fn parse_default_view(value: &str) -> DefaultView {
    match value {
        "feed" => DefaultView::Feed,
        "search" => DefaultView::Search,
        _ => DefaultView::Library,
    }
}

fn parse_list_density(value: &str) -> ListDensity {
    match value {
        "compact" => ListDensity::Compact,
        _ => ListDensity::Comfortable,
    }
}

fn parse_side_panel_mode(value: &str) -> SidePanelMode {
    match value {
        "open" => SidePanelMode::Open,
        "closed" => SidePanelMode::Closed,
        _ => SidePanelMode::Auto,
    }
}

fn parse_triage_mode(value: &str) -> TriageMode {
    match value {
        "manual" => TriageMode::Manual,
        _ => TriageMode::Focus,
    }
}

fn parse_reader_font_family(value: &str) -> ReaderFontFamily {
    match value {
        "sans" => ReaderFontFamily::Sans,
        "mono" => ReaderFontFamily::Mono,
        "atkinson" => ReaderFontFamily::Atkinson,
        _ => ReaderFontFamily::Serif,
    }
}

fn parse_reader_font_size(value: &str) -> ReaderFontSize {
    match value {
        "small" => ReaderFontSize::Small,
        "large" => ReaderFontSize::Large,
        _ => ReaderFontSize::Medium,
    }
}

fn parse_reader_line_height(value: &str) -> ReaderLineHeight {
    match value {
        "compact" => ReaderLineHeight::Compact,
        _ => ReaderLineHeight::Relaxed,
    }
}

fn parse_accent_color(value: &str) -> AccentColor {
    match value {
        "green" => AccentColor::Green,
        "orange" => AccentColor::Orange,
        "rose" => AccentColor::Rose,
        _ => AccentColor::Blue,
    }
}

fn parse_duplicate_sensitivity(value: &str) -> DuplicateSensitivity {
    match value {
        "low" => DuplicateSensitivity::Low,
        "high" => DuplicateSensitivity::High,
        _ => DuplicateSensitivity::Medium,
    }
}

fn parse_duplicate_action(value: &str) -> DuplicateAction {
    match value {
        "skip_silently" => DuplicateAction::SkipSilently,
        "merge_with_existing" => DuplicateAction::MergeWithExisting,
        _ => DuplicateAction::NotifyMe,
    }
}

fn row_to_preferences(row: &UserPreferencesRow) -> PreferencesSettings {
    PreferencesSettings {
        appearance: AppearanceSettings {
            accent_color: parse_accent_color(&row.accent_color),
        },
        layout: LayoutSettings {
            sidebar_mode: parse_sidebar_mode(&row.sidebar_mode),
            default_view: parse_default_view(&row.default_view),
            list_density: parse_list_density(&row.list_density),
            side_panel: parse_side_panel_mode(&row.side_panel),
        },
        workflow: WorkflowSettings {
            triage_mode: parse_triage_mode(&row.triage_mode),
            auto_advance: row.auto_advance,
        },
        reader: ReaderSettings {
            font_family: parse_reader_font_family(&row.reader_font_family),
            font_size: parse_reader_font_size(&row.reader_font_size),
            line_height: parse_reader_line_height(&row.reader_line_height),
        },
        ai: AiPreferenceSettings {
            mila_enabled: row.ai_mila_enabled,
            custom_prompt: row.ai_custom_prompt.clone(),
        },
    }
}

fn row_to_archival(row: &UserPreferencesRow) -> ArchivalSettings {
    ArchivalSettings {
        archive_formats: ArchiveFormatPreferences {
            readable_html: true,
            monolith: row.archival_monolith,
            pdf: row.archival_pdf,
            screenshot: row.archival_screenshot,
            warc: row.archival_warc,
        },
        duplicate_detection: DuplicateDetectionSettings {
            enabled: row.duplicate_detection_enabled,
            sensitivity: parse_duplicate_sensitivity(&row.duplicate_sensitivity),
            on_duplicate: parse_duplicate_action(&row.duplicate_action),
        },
        processing: ArchivalProcessingSettings {
            browser_timeout_secs: row.browser_timeout_secs.max(0) as u32,
            max_concurrent_archives: row.max_concurrent_archives.max(0) as u32,
            ai_auto_processing: row.ai_auto_processing,
        },
        proxy: ProxySettings {
            url: row.proxy_url.clone(),
            all_requests: row.proxy_all_requests,
        },
    }
}

async fn fetch_row(pool: &PgPool, user_id: UserId) -> Result<Option<UserPreferencesRow>, AppError> {
    sqlx::query_as::<_, UserPreferencesRow>(
        "SELECT accent_color, sidebar_mode, default_view, list_density, side_panel, \
         triage_mode, auto_advance, reader_font_family, reader_font_size, reader_line_height, \
         ai_mila_enabled, ai_custom_prompt, archival_monolith, \
         archival_pdf, archival_screenshot, archival_warc, duplicate_detection_enabled, \
         duplicate_sensitivity, duplicate_action, browser_timeout_secs, max_concurrent_archives, \
         ai_auto_processing, proxy_url, proxy_all_requests \
         FROM user_preferences WHERE user_id = $1",
    )
    .bind(user_id.into_uuid())
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)
}

#[async_trait::async_trait]
impl UserPreferencesRepository for PgUserPreferencesRepository {
    async fn get_preferences(
        &self,
        user_id: UserId,
    ) -> Result<Option<PreferencesSettings>, AppError> {
        Ok(fetch_row(&self.pool, user_id)
            .await?
            .map(|row| row_to_preferences(&row)))
    }

    async fn upsert_preferences(
        &self,
        user_id: UserId,
        settings: &PreferencesSettings,
    ) -> Result<PreferencesSettings, AppError> {
        let accent_color = format!("{:?}", settings.appearance.accent_color).to_lowercase();
        let sidebar_mode = match settings.layout.sidebar_mode {
            SidebarMode::Expanded => "expanded",
            SidebarMode::Collapsed => "collapsed",
            SidebarMode::Auto => "auto",
        };
        let default_view = match settings.layout.default_view {
            DefaultView::Library => "library",
            DefaultView::Feed => "feed",
            DefaultView::Search => "search",
        };
        let list_density = match settings.layout.list_density {
            ListDensity::Comfortable => "comfortable",
            ListDensity::Compact => "compact",
        };
        let side_panel = match settings.layout.side_panel {
            SidePanelMode::Auto => "auto",
            SidePanelMode::Open => "open",
            SidePanelMode::Closed => "closed",
        };
        let triage_mode = match settings.workflow.triage_mode {
            TriageMode::Manual => "manual",
            TriageMode::Focus => "focus",
        };
        let reader_font_family = match settings.reader.font_family {
            ReaderFontFamily::Serif => "serif",
            ReaderFontFamily::Sans => "sans",
            ReaderFontFamily::Mono => "mono",
            ReaderFontFamily::Atkinson => "atkinson",
        };
        let reader_font_size = match settings.reader.font_size {
            ReaderFontSize::Small => "small",
            ReaderFontSize::Medium => "medium",
            ReaderFontSize::Large => "large",
        };
        let reader_line_height = match settings.reader.line_height {
            ReaderLineHeight::Compact => "compact",
            ReaderLineHeight::Relaxed => "relaxed",
        };

        let row = sqlx::query_as::<_, UserPreferencesRow>(
            "INSERT INTO user_preferences \
             (user_id, accent_color, sidebar_mode, default_view, list_density, side_panel, \
              triage_mode, auto_advance, reader_font_family, reader_font_size, reader_line_height, \
              ai_mila_enabled, ai_custom_prompt) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) \
             ON CONFLICT (user_id) DO UPDATE SET \
               accent_color = EXCLUDED.accent_color, \
               sidebar_mode = EXCLUDED.sidebar_mode, \
               default_view = EXCLUDED.default_view, \
               list_density = EXCLUDED.list_density, \
               side_panel = EXCLUDED.side_panel, \
               triage_mode = EXCLUDED.triage_mode, \
               auto_advance = EXCLUDED.auto_advance, \
               reader_font_family = EXCLUDED.reader_font_family, \
               reader_font_size = EXCLUDED.reader_font_size, \
               reader_line_height = EXCLUDED.reader_line_height, \
               ai_mila_enabled = EXCLUDED.ai_mila_enabled, \
               ai_custom_prompt = EXCLUDED.ai_custom_prompt, \
               updated_at = now() \
             RETURNING accent_color, sidebar_mode, default_view, list_density, side_panel, \
             triage_mode, auto_advance, reader_font_family, reader_font_size, reader_line_height, \
             ai_mila_enabled, ai_custom_prompt, archival_monolith, \
             archival_pdf, archival_screenshot, archival_warc, duplicate_detection_enabled, \
             duplicate_sensitivity, duplicate_action, browser_timeout_secs, max_concurrent_archives, \
             ai_auto_processing, proxy_url, proxy_all_requests",
        )
        .bind(user_id.into_uuid())
        .bind(accent_color)
        .bind(sidebar_mode)
        .bind(default_view)
        .bind(list_density)
        .bind(side_panel)
        .bind(triage_mode)
        .bind(settings.workflow.auto_advance)
        .bind(reader_font_family)
        .bind(reader_font_size)
        .bind(reader_line_height)
        .bind(settings.ai.mila_enabled)
        .bind(settings.ai.custom_prompt.as_deref())
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row_to_preferences(&row))
    }

    async fn get_archival(&self, user_id: UserId) -> Result<Option<ArchivalSettings>, AppError> {
        Ok(fetch_row(&self.pool, user_id)
            .await?
            .map(|row| row_to_archival(&row)))
    }

    async fn upsert_archival(
        &self,
        user_id: UserId,
        settings: &ArchivalSettings,
    ) -> Result<ArchivalSettings, AppError> {
        let duplicate_sensitivity = match settings.duplicate_detection.sensitivity {
            DuplicateSensitivity::Low => "low",
            DuplicateSensitivity::Medium => "medium",
            DuplicateSensitivity::High => "high",
        };
        let duplicate_action = match settings.duplicate_detection.on_duplicate {
            DuplicateAction::NotifyMe => "notify_me",
            DuplicateAction::SkipSilently => "skip_silently",
            DuplicateAction::MergeWithExisting => "merge_with_existing",
        };

        let row = sqlx::query_as::<_, UserPreferencesRow>(
            "INSERT INTO user_preferences \
             (user_id, archival_monolith, archival_pdf, archival_screenshot, archival_warc, \
              duplicate_detection_enabled, duplicate_sensitivity, duplicate_action, \
              browser_timeout_secs, max_concurrent_archives, ai_auto_processing, proxy_url, \
              proxy_all_requests) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) \
             ON CONFLICT (user_id) DO UPDATE SET \
               archival_monolith = EXCLUDED.archival_monolith, \
               archival_pdf = EXCLUDED.archival_pdf, \
               archival_screenshot = EXCLUDED.archival_screenshot, \
               archival_warc = EXCLUDED.archival_warc, \
               duplicate_detection_enabled = EXCLUDED.duplicate_detection_enabled, \
               duplicate_sensitivity = EXCLUDED.duplicate_sensitivity, \
               duplicate_action = EXCLUDED.duplicate_action, \
               browser_timeout_secs = EXCLUDED.browser_timeout_secs, \
               max_concurrent_archives = EXCLUDED.max_concurrent_archives, \
               ai_auto_processing = EXCLUDED.ai_auto_processing, \
               proxy_url = EXCLUDED.proxy_url, \
               proxy_all_requests = EXCLUDED.proxy_all_requests, \
               updated_at = now() \
             RETURNING accent_color, sidebar_mode, default_view, list_density, side_panel, \
             triage_mode, auto_advance, reader_font_family, reader_font_size, reader_line_height, \
             ai_mila_enabled, ai_custom_prompt, archival_monolith, \
             archival_pdf, archival_screenshot, archival_warc, duplicate_detection_enabled, \
             duplicate_sensitivity, duplicate_action, browser_timeout_secs, max_concurrent_archives, \
             ai_auto_processing, proxy_url, proxy_all_requests",
        )
        .bind(user_id.into_uuid())
        .bind(settings.archive_formats.monolith)
        .bind(settings.archive_formats.pdf)
        .bind(settings.archive_formats.screenshot)
        .bind(settings.archive_formats.warc)
        .bind(settings.duplicate_detection.enabled)
        .bind(duplicate_sensitivity)
        .bind(duplicate_action)
        .bind(settings.processing.browser_timeout_secs as i32)
        .bind(settings.processing.max_concurrent_archives as i32)
        .bind(settings.processing.ai_auto_processing)
        .bind(settings.proxy.url.as_deref())
        .bind(settings.proxy.all_requests)
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row_to_archival(&row))
    }
}

#[cfg(test)]
mod tests {
    use super::parse_triage_mode;
    use ind_domain::TriageMode;

    #[test]
    fn parses_supported_triage_modes_and_defaults_unknown_values_to_focus() {
        assert_eq!(parse_triage_mode("manual"), TriageMode::Manual);
        assert_eq!(parse_triage_mode("focus"), TriageMode::Focus);
        assert_eq!(parse_triage_mode("unknown"), TriageMode::Focus);
    }
}
