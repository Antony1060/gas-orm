use gas::types::Decimal;

#[gas::model(table_name = "students")]
// #[derive(Debug, Clone)]
pub struct Student {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

#[gas::model(table_name = "users")]
// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(rename_all = "PascalCase")]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    // #[serde(rename = "pw_hash")]
    pub password: String,
    pub bank_account_balance: Decimal,
}
