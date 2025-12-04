use gas::types::{DateTime, Decimal, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use gas::Relation;

#[gas::model(table_name = "persons")]
#[derive(Debug)]
pub struct Person {
    #[primary_key]
    #[serial]
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    #[unique]
    pub email: String,
    pub phone_number: Option<String>,
    #[default(fn = Decimal::from(100))]
    #[column(name = "bank_balance")]
    pub bank_account_balance: Decimal,
}

#[gas::model(table_name = "audit_logs", mod_name = "audit_logs")]
#[derive(Debug)]
pub struct AuditLog {
    #[primary_key]
    #[serial]
    pub id: i64,
    #[default(fn = Utc::now())]
    pub created_at: DateTime<Utc>,
    pub updated_at: NaiveDateTime,
    pub random_date: NaiveDate,
    pub random_time: NaiveTime,
}

#[gas::model(table_name = "users")]
#[derive(Debug)]
pub struct User {
    #[primary_key]
    #[serial]
    pub id: i64,
    pub name: String,
    #[relation(inverse = post::user)]
    pub posts: Vec<post::Model>,
}

#[gas::model(table_name = "posts")]
#[derive(Debug)]
pub struct Post {
    #[primary_key]
    #[serial]
    pub id: i64,
    pub title: String,
    // TODO: references to itself or multiple others
    #[column(name = "user_fk")]
    #[relation(field = user::id)]
    pub user: Relation<i64, user::Model>,
}

#[gas::model(table_name = "products")]
#[derive(Debug)]
pub struct Product {
    #[primary_key]
    #[serial]
    pub id: i64,
    pub name: String,
}

#[gas::model(table_name = "orders")]
#[derive(Debug)]
pub struct Order {
    #[primary_key]
    #[serial]
    pub id: i64,
    pub quantity: i32,
    #[column(name = "user_fk")]
    #[relation(field = user::id)]
    pub user: Relation<i64, user::Model>,
    #[column(name = "product_fk")]
    #[relation(field = product::id)]
    pub product: Option<Relation<i64, product::Model>>,
}
