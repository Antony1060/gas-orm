pub mod user {
    use crate::sql::{Field, ModelOps, PgType};

    pub const username: Field<String> = Field::new("username", PgType::TEXT);
    pub const email: Field<String> = Field::new("email", PgType::TEXT);
    pub const password: Field<String> = Field::new("password", PgType::TEXT);

    pub struct Model {
        pub username: String,
        pub email: String,
        pub password: String,
    }

    impl ModelOps for Model {
        fn table_name() -> &'static str {
            "users"
        }
    }
}
