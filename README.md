# gas-orm

A very simple Postgres ORM in Rust, made as a University project. Not production ready, still work in progress.

### Basic usage for now

```rust
#[gas::model(table_name = "persons")]
#[derive(Debug, Clone)]
pub struct Person {
    #[primary_key]
    #[serial]
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone_number: Option<String>,
    #[column(name = "bank_balance")]
    pub bank_account_balance: Decimal,
}

#[tokio::main]
async fn main() -> GasResult<()> {
    let conn =
        PgConnection::new_connection_pool("postgres://user:password@localhost/db")
            .await?;

    person::Model::create_table(&conn, true).await?;

    let persons = person::Model::query()
        .filter(|| {
            (person::bank_account_balance.gte(6000) & person::phone_number.is_not_null())
                | (person::id.gte(18) & person::phone_number.is_null())
        })
        .find_all(&conn)
        .await?;

    dbg!(&persons);

    Ok(())
}
```

### Very short term todo

- [x] Migrations diff ordering
- [x] Implement all diffs for migrations
- [ ] Mirations cli `migrate` command
- [ ] Migrations on library side
