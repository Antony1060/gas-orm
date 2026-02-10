use gas::Relation;

#[gas::model(table_name = "authors")]
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[schema(as = Author)]
pub struct Author {
    #[primary_key]
    #[serial]
    pub id: i64,
    pub name: String,
    #[default(fn = String::new(), sql = r#"''"#)]
    pub email: String,
    pub bio: String,
}

#[gas::model(table_name = "categories")]
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[schema(as = Category)]
pub struct Category {
    #[primary_key]
    #[serial]
    pub id: i64,
    #[unique]
    pub name: String,
    pub description: String,
}

// utoipa seems to buchet my types, so deriving ToSchema breaks compilation
#[gas::model(table_name = "books")]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Book {
    #[primary_key]
    #[serial]
    pub id: i64,
    pub title: String,
    #[unique]
    pub isbn: String,
    pub published_year: i32,
    #[default(fn = 0, sql = r#"0"#)]
    pub page_count: i32,
    #[column(name = "author_fk")]
    #[relation(field = author::id)]
    pub author: Relation<i64, author::Model>,
}

#[gas::model(table_name = "reviews")]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Review {
    #[primary_key]
    #[serial]
    pub id: i64,
    #[default(fn = String::new(), sql = r#"''"#)]
    pub reviewer_name: String,
    pub rating: i32,
    pub content: String,
    #[column(name = "book_fk")]
    #[relation(field = book::id)]
    #[serde(skip)]
    pub book: Relation<i64, book::Model>,
}
