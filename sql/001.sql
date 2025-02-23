DROP TABLE IF EXISTS templates CASCADE;
DROP TABLE IF EXISTS category CASCADE;
DROP TABLE IF EXISTS posts CASCADE;
DROP TABLE IF EXISTS pages CASCADE;
DROP TABLE IF EXISTS config CASCADE;

CREATE TABLE templates
(
    key      TEXT PRIMARY KEY,
    template TEXT NOT NULL
);

CREATE TABLE category
(
    id       UUID PRIMARY KEY,
    name     TEXT NOT NULL,
    template TEXT
);

CREATE TABLE posts
(
    title       TEXT,
    slug        TEXT PRIMARY KEY,
    description TEXT   NOT NULL DEFAULT '',
    category    UUID   NOT NULL REFERENCES category (id),
    published   TIMESTAMP,
    content     TEXT   NOT NULL DEFAULT '',
    raw         TEXT   NOT NULL DEFAULT '',
    props       HSTORE NOT NULL
);

CREATE INDEX idx_posts_timestamp ON posts (published DESC);

CREATE TABLE pages
(
    title       TEXT,
    slug        TEXT PRIMARY KEY,
    description TEXT NOT NULL DEFAULT '',
    content     TEXT NOT NULL DEFAULT '',
    raw         TEXT NOT NULL DEFAULT ''
);

CREATE TABLE config
(
    key         TEXT PRIMARY KEY,
    value       TEXT,
    last_update TIMESTAMP
);

CREATE OR REPLACE PROCEDURE set_config(
    p_key TEXT,
    p_value TEXT
)
AS
$$
BEGIN
    INSERT INTO config (key, value, last_update)
    VALUES (p_key, p_value, NOW())
    ON CONFLICT ON CONSTRAINT config_pkey
        DO UPDATE SET key         = p_key,
                      value       = p_value,
                      last_update = NOW();
END;
$$ LANGUAGE plpgsql;

DROP FUNCTION IF EXISTS list_posts(integer, integer, uuid);
CREATE FUNCTION list_posts(
    p_page_size INT,
    p_page_number INT,
    p_category UUID
)
    RETURNS TABLE
            (
                title       TEXT,
                slug        TEXT,
                description TEXT,
                category    TEXT,
                category_id UUID
            )
AS
$$
DECLARE
    l_page_offset INT;
BEGIN
    l_page_offset := GREATEST(p_page_number - 1, 0) * p_page_size;

    RETURN QUERY
        SELECT posts.title, posts.slug, posts.description, cat.name, cat.id
        FROM posts
                 INNER JOIN category cat on posts.category = cat.id
        WHERE posts.published IS NOT NULL
          AND (p_category IS NULL OR posts.category = p_category)
        ORDER BY posts.published DESC
        LIMIT p_page_size OFFSET l_page_offset;
END;
$$ LANGUAGE plpgsql;

-- Preset Category For Dev
INSERT INTO category (id, name, template)
VALUES ('76a305cc-b0d4-40b3-a15b-84ec616ea79f', 'Programming', NULL)
ON CONFLICT ON CONSTRAINT category_pkey DO NOTHING;