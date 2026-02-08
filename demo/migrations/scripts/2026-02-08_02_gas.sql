CREATE SEQUENCE ddcc_id_seq;
ALTER TABLE ddcc ALTER COLUMN id SET DEFAULT (nextval('ddcc_id_seq'::regclass));
ALTER SEQUENCE ddcc_id_seq OWNED BY ddcc.id;
ALTER TABLE ddcc DROP CONSTRAINT ddcc_var3_fkey;
-- GAS_ORM(forward_backward_separator)
ALTER TABLE ddcc ADD FOREIGN KEY(var3) REFERENCES ccdd(id);
ALTER TABLE ddcc ALTER COLUMN id DROP DEFAULT;
DROP SEQUENCE ddcc_id_seq;
