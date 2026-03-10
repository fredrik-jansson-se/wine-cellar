-- Add new columns
ALTER TABLE wines ADD COLUMN comment TEXT;
ALTER TABLE wines ADD COLUMN comment_updated_at DATETIME;

-- Migrate existing comments: concatenate oldest-first per wine
-- DATA IMPACT: wine_comments rows are irreversibly consolidated; the source table is dropped.
UPDATE wines
SET
    comment = (
        SELECT group_concat(comment, char(10) || char(10))
        FROM (
            SELECT comment
            FROM wine_comments
            WHERE wine_id = wines.wine_id
            ORDER BY dt ASC
        )
    ),
    comment_updated_at = (
        SELECT MAX(dt)
        FROM wine_comments
        WHERE wine_id = wines.wine_id
    )
WHERE wine_id IN (SELECT DISTINCT wine_id FROM wine_comments);

-- Drop old table
DROP TABLE wine_comments;
