ALTER TABLE ddcc ADD FOREIGN KEY(var5) REFERENCES bb(id);
-- GAS_ORM(forward_backward_separator)
ALTER TABLE ddcc DROP CONSTRAINT ddcc_var5_fkey;
