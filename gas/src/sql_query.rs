use crate::error::GasError;
use crate::pg_param::PgParam;
use crate::GasResult;
use lazy_static::lazy_static;

lazy_static! {
    static ref PG_PARAMATER_REGEX: regex::Regex = regex::Regex::new(r"\$\d+").unwrap();
}

// eh
#[derive(Debug)]
pub struct SqlQuery {
    query: String,
}

pub type SqlStatement<'a> = (SqlQuery, &'a [PgParam]);

impl SqlQuery {
    pub fn new<T: AsRef<str>>(query: T) -> Self {
        SqlQuery {
            query: query.as_ref().to_string(),
        }
    }

    pub fn append_query(&mut self, other: SqlQuery) {
        self.query.push_str(&other.query);
    }

    pub fn append_str(&mut self, other: &str) {
        self.query.push_str(other);
    }

    pub(crate) fn finish(self) -> GasResult<String> {
        let mut updated = 0;
        let out = self.query.split("?").fold(String::new(), |acc, curr| {
            // special case for the first entry, which will always be the empty string
            if acc.is_empty() {
                return curr.to_string();
            }

            updated += 1;
            format!("{acc}${updated}{curr}")
        });

        let param_count = PG_PARAMATER_REGEX.find_iter(&out).count();
        if param_count != updated {
            return Err(GasError::QueryFormatError);
        }

        if !out.ends_with(';') {
            return Ok(out + ";");
        }

        Ok(out)
    }
}

impl<T: AsRef<str>> From<T> for SqlQuery {
    fn from(value: T) -> Self {
        SqlQuery::new(value)
    }
}

#[cfg(test)]
mod test {
    use crate::error::GasError;
    use crate::sql_query::SqlQuery;

    #[test]
    pub fn test_parameterize() {
        let query = SqlQuery::from("WHERE id=? AND name IN (?, ?, ?)").finish();

        assert!(matches!(query, Ok(out) if out == "WHERE id=$1 AND name IN ($2, $3, $4)"))
    }

    #[test]
    pub fn test_fail() {
        let query = SqlQuery::from("WHERE id IN (?, $2, ?)").finish();

        assert!(matches!(query, Err(GasError::QueryFormatError)));
    }
}
