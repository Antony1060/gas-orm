ALTER TABLE ddcc DROP CONSTRAINT var1_unq;
ALTER TABLE ddcc ADD UNIQUE(var4);
-- GAS_ORM(forward_backward_separator)
ALTER TABLE ddcc DROP CONSTRAINT var4_unq;
ALTER TABLE ddcc ADD UNIQUE(var1);
