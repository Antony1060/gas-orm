ALTER TABLE authors ADD COLUMN email TEXT NOT NULL DEFAULT ('');
ALTER TABLE books ADD COLUMN page_count INTEGER NOT NULL DEFAULT (0);
ALTER TABLE reviews ADD COLUMN reviewer_name TEXT NOT NULL DEFAULT ('');
-- GAS_ORM(forward_backward_separator)
ALTER TABLE reviews DROP COLUMN reviewer_name;
ALTER TABLE books DROP COLUMN page_count;
ALTER TABLE authors DROP COLUMN email;
