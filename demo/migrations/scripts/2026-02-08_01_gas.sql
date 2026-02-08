ALTER TABLE ddcc ADD UNIQUE(var1);
ALTER TABLE ddcc DROP CONSTRAINT ddcc_var4_key;
-- GAS_ORM(forward_backward_separator)
ALTER TABLE ddcc ADD UNIQUE(var4);
ALTER TABLE ddcc DROP CONSTRAINT ddcc_var1_key;
