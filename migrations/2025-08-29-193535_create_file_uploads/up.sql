CREATE TABLE file_uploads (
    id UUID NOT NULL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    mime_type VARCHAR(128) NOT NULL,
    size BIGINT NOT NULL,
    uploaded_at TIMESTAMP NOT NULL
);
