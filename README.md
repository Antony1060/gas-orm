# gas

A macro-driven PostgreSQL ORM for Rust, using [sqlx](https://github.com/launchbadge/sqlx) as a backing driver. I'm using
it in production in some projects but still working towards a v1.

## Getting started

### Installing

```toml
[dependencies]
# not on crates.io yet, tags will come soon
gas = { git = "https://github.com/antony1060/gas-orm" }
# tokio is a hard dependency for now
tokio = { version = "1", features = ["full"] }
```

### Basic usage

Define a model, and `gas` generates a module for interacting with the database:

```rust
use gas::connection::PgConnection;
use gas::GasResult;

#[gas::model(table_name = "todos")]
#[derive(Debug, Clone)]
pub struct Todo {
    #[primary_key]
    #[serial]
    pub id: i64,
    pub title: String,
    pub done: bool,
}

#[tokio::main]
async fn main() -> GasResult<()> {
    let db = PgConnection::new_connection_pool("postgres://localhost/myapp").await?;

    todo::Model::create_table(&db, true).await?;

    // insert
    let mut new_todo = todo::Model {
        title: "Write README".into(),
        done: false,
        ..Default::default()
    };
    // will mutate `new_todo` with the `id` from the database
    new_todo.insert(&db).await?;
    // `new_todo.inserted(&db)` can be used to return the database value instead of mutating it

    // query
    let pending = todo::Model::query()
        .filter(|| todo::done.eq(false))
        .sort(todo::id.desc())
        .limit(10)
        .find_all(&db)
        .await?;

    // update
    let mut first = todo::Model::find_by_key(&db, 1).await?.unwrap();
    first.done = true;
    first.update(&db).await?;

    // delete
    first.delete(&db).await?;

    Ok(())
}
```

The `#[gas::model]` macro expands your struct into a module (here `todo`) containing a `Model` type and typed field
statics like `todo::title`, `todo::done`, etc.

> [!NOTE]
> It's recommended to add additional attribute and derive macros below this one.

## Querying

Filters are type-safe and composable with `&` (AND) and `|` (OR):

```rust
let results = todo::Model::query()
.filter(| | {
(todo::title.eq("Important") & todo::done.eq(false))
| todo::id.one_of( & [1, 2, 3])
})
.sort(todo::title.asc() > > todo::id.desc())
.limit(25)
.find_all( & db)
.await?;

// or just grab one
let single = todo::Model::query()
.filter( | | todo::title.eq("Write README"))
.find_one( & db)
.await?; // Option<todo::Model>
```

Fields expose comparison methods depending on their type: `eq`, `neq`, `lt`, `lte`, `gt`, `gte`, `one_of` for values;
`is_null`, `is_not_null` for optionals; and `is_before_now`, `is_after_now`, etc. for date/time fields.

### Aggregates

```rust
let total = order::Model::query()
.filter( | | order::status.eq("completed"))
.sum( & db, order::amount)
.await?;

let count = todo::Model::query()
.count( & db, todo::id)
.await?;

// group by
let per_status = order::Model::query()
.group(order::status)
.count( & db, order::id)
.await?; // Vec<Counted<String>>
```

## Relations

Define foreign keys with `#[relation(field = ...)]` and `gas::Relation`:

```rust
use gas::Relation;

#[gas::model(table_name = "authors")]
#[derive(Debug, Clone)]
pub struct Author {
    #[primary_key]
    #[serial]
    pub id: i64,
    pub name: String,
}

#[gas::model(table_name = "books")]
#[derive(Debug, Clone)]
pub struct Book {
    #[primary_key]
    #[serial]
    pub id: i64,
    pub title: String,
    #[column(name = "author_fk")]
    #[relation(field = author::id)]
    pub author: gas::Relation<i64, author::Model>,
}
```

Eager load with `include` (LEFT JOIN):

```rust
let books = book::Model::query()
.include(book::author)
.find_all( & db)
.await?;
```

Or lazy load on demand:

```rust
let mut b = book::Model::find_by_key( & db, 1).await?.unwrap();
let author = b.author.load( & db).await?;
```

### Inverse relations

You can also go the other direction - from a parent to its children - with `#[relation(inverse = ...)]`. Use
`Vec<Model>` for N:1 or `Option<Model>` for 1:1:

```rust
#[gas::model(table_name = "authors")]
#[derive(Debug, Clone)]
pub struct Author {
    #[primary_key]
    #[serial]
    pub id: i64,
    pub name: String,
    #[relation(inverse = book::author)]
    pub books: Vec<book::Model>,
}
```

Inverse relations are eagerly loaded when the parent is queried. The field implements a `Deref` to the inner type (`Vec`
or `Option`), so you can iterate directly:

```rust
let author = author::Model::find_by_key( & db, 1).await?.unwrap();
for book in author.books.iter() {
println ! ("{}", book.title);
}
```

> [!WARNING]
> Eager inverse relations run N + 1 queries for N returned rows. Keep this in mind for large result sets.

## Migrations

Each migration is a `.sql` file with forward and backward sections split by a marker:

```sql
CREATE TABLE todos
(... );
-- GAS_ORM(forward_backward_separator)
DROP TABLE todos;
```

Migrations live in a `migrations/` directory by default (configurable via `-m` on the CLI).

### CLI

```bash
gas-cli migrations init                       # scaffold the migrations directory
gas-cli migrations sync                       # diff your models and generate a migration
gas-cli migrations migrate                    # run all pending migrations
gas-cli migrations migrate --back --count 1   # roll back one step
```

### Runtime

You can also run migrations from your app on startup:

```rust
use gas::{load_migrations, migrations::{MigrateDirection, MigrateCount}};

let migrator = load_migrations!("./migrations")?;
migrator.run_migrations( & db, MigrateDirection::Forward, MigrateCount::All).await?;
```

> [!TIP]
> Add a `build.rs` so Cargo recompiles when migration files change:
>
> ```rust
> fn main() {
>     println!("cargo:rerun-if-changed=migrations/");
> }
> ```

## Axum integration

Enable the `axum` feature:

```toml
gas = { git = "https://github.com/antony1060/gas-orm", features = ["axum"] }
```

This gives you a Tower middleware layer that wraps each request in a transaction (auto-commits on 2xx, rolls back
otherwise), plus extractors for your handlers:

```rust
use gas::extra::axum::{Connection, Transaction};

let app = Router::new()
.route("/todos", get(list_todos))
.layer(gas::extra::tower::layer( & db));

async fn list_todos(Connection(db): Connection) -> impl IntoResponse {
    let todos = todo::Model::query().find_all(&db).await.unwrap();
    Json(todos)
}

async fn create_todo(
    Transaction(tx): Transaction,
    Json(req): Json<todo::Model>,
) -> impl IntoResponse {
    req.inserted(&tx).await.unwrap();

    axum::http::StatusCode::CREATED
}
```

## Model attributes

| Attribute                             | Level  | Description                                  |
|---------------------------------------|--------|----------------------------------------------|
| `#[gas::model(table_name = "...")]`   | Struct | Postgres table name                          |
| `#[gas::model(mod_name = "...")]`     | Struct | Override generated module name               |
| `#[primary_key]`                      | Field  | Primary key                                  |
| `#[serial]`                           | Field  | Auto-increment (`BIGSERIAL`)                 |
| `#[unique]`                           | Field  | `UNIQUE` constraint                          |
| `#[column(name = "...")]`             | Field  | Custom column name                           |
| `#[default(fn = expr, sql = "...")]`  | Field  | Default value in Rust (`fn`) and DDL (`sql`) |
| `#[relation(field = model::field)]`   | Field  | Forward foreign key                          |
| `#[relation(inverse = model::field)]` | Field  | Inverse (has-many) relation                  |

## Supported types

| Rust                              | PostgreSQL                        |
|-----------------------------------|-----------------------------------|
| `String`                          | `TEXT`                            |
| `bool`                            | `BOOLEAN`                         |
| `i16` / `i32` / `i64`             | `SMALLINT` / `INTEGER` / `BIGINT` |
| `f32` / `f64`                     | `REAL` / `DOUBLE PRECISION`       |
| `Decimal`                         | `DECIMAL`                         |
| `NaiveDateTime`                   | `TIMESTAMP`                       |
| `DateTime<Utc/Local/FixedOffset>` | `TIMESTAMPTZ`                     |
| `NaiveDate` / `NaiveTime`         | `DATE` / `TIME`                   |
| `serde_json::Value`               | `JSONB`                           |
| `Option<T>`                       | nullable variant                  |
| `Relation<Fk, Model>`             | `FOREIGN KEY REFERENCES`          |

## Workspace crates

| Crate        | Role                                          |
|--------------|-----------------------------------------------|
| `gas`        | Core ORM library                              |
| `gas-macros` | Proc macros (`#[model]`, `load_migrations!`)  |
| `gas-shared` | Shared types used by both the library and CLI |
| `gas-cli`    | Migration CLI                                 |
| `demo`       | Example bookstore API (Axum + Swagger UI)     |

## Demo

The `demo/` crate is a bookstore REST API with Swagger UI:

```bash
export DATABASE_URL="postgres://postgres:password@localhost/bookstore"
cargo run -p demo
# http://localhost:3000/swagger-ui
```

Set `TRACE_ORM=1` to log generated SQL.