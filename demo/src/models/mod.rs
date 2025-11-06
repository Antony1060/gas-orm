#[pg_orm::model]
pub struct Student {
    pub id: u64,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

#[pg_orm::model]
pub struct User {
    pub username: String,
    pub email: String,
    pub password: String,
}
