CREATE TABLE IF NOT EXISTS miniflux_sync_map (user_id UUID NOT NULL, document_id UUID NOT NULL, miniflux_id INT NOT NULL, PRIMARY KEY (user_id, document_id));
