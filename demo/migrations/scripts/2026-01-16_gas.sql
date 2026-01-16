ALTER TABLE foo ADD id BIGINT;
-- GAS_ORM(forward_backward_separator)
ALTER TABLE foo DROP COLUMN id;
