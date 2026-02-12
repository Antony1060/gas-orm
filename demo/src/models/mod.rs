use gas::Relation;

#[gas::model(table_name = "addresses")]
#[derive(Debug, serde::Serialize)]
pub struct Address {
    #[primary_key]
    #[serial]
    pub id: i64,
    pub address: String,
}

#[gas::model(table_name = "authors")]
#[derive(Debug, serde::Serialize)]
pub struct Author {
    #[primary_key]
    #[serial]
    pub id: i64,
    pub name: String,
    #[default(fn = String::new(), sql = r#"''"#)]
    pub email: String,
    pub bio: String,

    #[column(name = "address_fk")]
    #[relation(field = address::id)]
    pub address: Relation<i64, address::Model>,
    // // since this is always eager (and also always O(n) queries for n returned entries),
    // //  consider not including in most structs; here just for demo purposes
    // #[serde(skip_deserializing)]
    // #[relation(inverse = book::author)]
    // pub books: Vec<book::Model>,
}

#[gas::model(table_name = "categories")]
#[derive(Debug, serde::Serialize)]
pub struct Category {
    #[primary_key]
    #[serial]
    pub id: i64,
    #[unique]
    pub name: String,
    #[unique]
    pub description: String,
}

// utoipa seems to bucher my types, so deriving ToSchema breaks compilation
#[gas::model(table_name = "books")]
#[derive(Debug, serde::Serialize)]
pub struct Book {
    #[primary_key]
    #[serial]
    pub id: i64,
    pub title: String,
    pub isbn: String,
    pub published_year: i32,
    #[default(fn = 0, sql = r#"0"#)]
    pub page_count: i32,
    #[serde(skip_deserializing)]
    #[column(name = "author_fk")]
    #[relation(field = author::id)]
    pub author: Relation<i64, author::Model>,
}

#[gas::model(table_name = "reviews")]
#[derive(Debug, serde::Serialize)]
pub struct Review {
    #[primary_key]
    #[serial]
    pub id: i64,
    #[default(fn = String::new(), sql = r#"''"#)]
    pub reviewer_name: String,
    pub rating: i32,
    pub content: String,
    #[serde(skip_deserializing)]
    #[column(name = "book_fk")]
    #[relation(field = book::id)]
    pub book: Relation<i64, book::Model>,
}
