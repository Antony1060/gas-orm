use crate::error::GasError;
use crate::internals::PgParam;
use crate::GasResult;
use lazy_static::lazy_static;
use std::borrow::Cow;

lazy_static! {
    static ref PG_PARAMATER_REGEX: regex::Regex = regex::Regex::new(r"\$\d+").unwrap();
}

// eh
#[derive(Debug, Default)]
pub struct SqlQuery<'a> {
    query: Cow<'a, str>,
}

pub type SqlStatement<'a> = (SqlQuery<'a>, Box<[PgParam]>);

impl<'a> SqlQuery<'a> {
    pub fn new() -> Self {
        SqlQuery {
            query: Cow::from(""),
        }
    }

    pub fn append_query(&mut self, other: SqlQuery) {
        self.query.to_mut().push_str(&other.query);
    }

    pub fn append_str(&mut self, other: &str) {
        self.query.to_mut().push_str(other);
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

impl<'a> From<String> for SqlQuery<'a> {
    fn from(value: String) -> Self {
        Self {
            query: Cow::from(value),
        }
    }
}

impl<'a> From<&'a str> for SqlQuery<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            query: Cow::from(value),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::error::GasError;
    use crate::internals::SqlQuery;

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
