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
//#[derive(Debug, Clone, Serialize)]
// #[serde(rename_all = "PascalCase")]
pub struct User {
    #[serial]
    #[primary_key]
    pub id: i64,
    pub username: String,
    pub email: String,
    // #[serde(rename = "pw_hash")]
    pub password: String,
    pub bank_account_balance: Decimal,
}

#[gas::model(table_name = "things")]
pub struct Thing {
    #[serial]
    #[primary_key]
    pub id: i32,
    pub txt: String,
    pub smallint: i16,
    pub int: i32,
    pub bigint: i64,
    pub real: f32,
    pub double: f64,
    pub dec: Decimal,
    pub txt_opt: Option<String>,
    pub smallint_opt: Option<i16>,
    pub int_opt: Option<i32>,
    pub bigint_opt: Option<i64>,
    pub real_opt: Option<f32>,
    pub double_opt: Option<f64>,
    pub dec_opt: Option<Decimal>,
}
