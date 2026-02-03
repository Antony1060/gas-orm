DROP TABLE aaa;
-- GAS_ORM(forward_backward_separator)
CREATE TABLE IF NOT EXISTS aaa(
	id BIGSERIAL NOT NULL,
	first_name TEXT NOT NULL, 
	PRIMARY KEY (id)
);
