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
    #[column(name = "bank_balance")]
    pub bank_account_balance: Decimal,
}
