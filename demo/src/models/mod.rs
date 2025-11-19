use gas::types::Decimal;

#[gas::model(table_name = "persons")]
#[derive(Debug, Clone)]
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
// #[gas::model(table_name = "audit_logs", mod_name = "audit_logs")]
// #[derive(Debug, Clone)]
// pub struct AuditLog {
//     #[primary_key]
//     #[serial]
//     pub id: i64,
// }

// #[gas::model(table_name = "audit_logs", mod_name = "audit_logs")]
// #[derive(Debug, Clone)]
// pub struct AuditLog {
//     #[primary_key]
//     #[serial]
//     pub id: i64,
//     pub created_at: DateTime<Utc>,
//     pub updated_at: NaiveDateTime,
//     pub random_date: NaiveDate,
//     pub random_time: NaiveTime,
// }
