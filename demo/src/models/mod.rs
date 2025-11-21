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
}

#[gas::model(table_name = "posts")]
#[derive(Debug)]
pub struct Post {
    #[primary_key]
    #[serial]
    pub id: i64,
    pub title: String,
    #[column(name = "user_fk")]
    pub user: Relation<i64, user::Model, { user::id.index }>,
}
