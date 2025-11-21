use gas::types::{DateTime, Decimal, NaiveDate, NaiveDateTime, NaiveTime, Utc};

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
    pub logs: gas::Relation<i64, audit_logs::Model, { audit_logs::id.index }>,
}

#[gas::model(table_name = "audit_logs", mod_name = "audit_logs")]
#[derive(Debug)]
pub struct AuditLog {
    #[primary_key]
    #[serial]
    pub id: i64,
    pub message: String,
    #[default(fn = Utc::now())]
    pub created_at: DateTime<Utc>,
    pub updated_at: NaiveDateTime,
    pub random_date: NaiveDate,
    pub random_time: NaiveTime,
}
