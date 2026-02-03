DROP TABLE aa;
-- GAS_ORM(forward_backward_separator)
CREATE TABLE IF NOT EXISTS aa(
	id BIGSERIAL NOT NULL,
	first_name TEXT NOT NULL,
	foreign BIGINT REFERENCES bb(id) NOT NULL, 
	PRIMARY KEY (id)
);
