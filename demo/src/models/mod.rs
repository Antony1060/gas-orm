#[gas::model(table_name = "students")]
pub struct Student {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

#[gas::model(table_name = "users")]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub bank_account_balance: gas::types::Decimal,
}
