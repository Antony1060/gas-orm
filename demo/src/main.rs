use crate::models::{audit_logs, order, person, post, product, user};
use crate::tracing_util::setup_tracing;
use gas::connection::PgConnection;
use gas::eq::{PgEq, PgEqNone, PgEqTime};
use gas::error::GasError;
use gas::group::GroupSorting;
use gas::helpers::OptionHelperOps;
use gas::types::{Local, NaiveDate, NaiveTime, TimeDelta, Utc};
use gas::{GasResult, ModelOps, RelationOps};
use rust_decimal::Decimal;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

mod models;
mod tracing_util;

#[tokio::main]
async fn main() -> GasResult<()> {
    setup_tracing(env::var("TRACE_ORM").map(|_| true).unwrap_or(false));

    let conn =
        PgConnection::new_connection_pool("postgres://postgres:strong_password@localhost/postgres")
            .await?;

    person::Model::create_table(&conn, true).await?;
    audit_logs::Model::create_table(&conn, true).await?;
    user::Model::create_table(&conn, true).await?;
    post::Model::create_table(&conn, true).await?;
    product::Model::create_table(&conn, true).await?;
    order::Model::create_table(&conn, true).await?;

    normal_ops(&conn).await?;
    tracing::info!("----------------");
    tracing::info!("by key");
    tracing::info!("----------------");
    by_key_ops(&conn).await?;
    tracing::info!("----------------");
    tracing::info!("transaction");
    tracing::info!("----------------");
    transaction_ops(&conn).await?;
    tracing::info!("----------------");
    tracing::info!("(date)(time)");
    tracing::info!("----------------");
    datetime_ops(&conn).await?;
    tracing::info!("----------------");
    tracing::info!("sorting");
    tracing::info!("----------------");
    sort_limit_ops(&conn).await?;
    tracing::info!("----------------");
    tracing::info!("aggregate");
    tracing::info!("----------------");
    aggregate_ops(&conn).await?;
    tracing::info!("----------------");
    tracing::info!("foreign key");
    tracing::info!("----------------");
    foreign_key_ops(&conn).await?;

    let rand_name = rand::random_range(10..100);
    user::Def! {
        name: format!("John Doe {}", rand_name),
    }
    .update_by_key(&conn, 4)
    .await?;

    Ok(())
}

async fn foreign_key_ops(conn: &PgConnection) -> GasResult<()> {
    let mut some_post = post::Model::query()
        .sort(post::id.asc())
        .find_one(conn)
        .await?
        .unwrap();

    tracing_dbg!(some_post);
    tracing_dbg!("fk", some_post.user.get_foreign_key());

    tracing_dbg!("lazy loaded", some_post.user.load(conn).await?);

    tracing_dbg!(some_post);
    tracing_dbg!("fk", some_post.user.get_foreign_key());

    tracing_dbg!(
        "eager loaded",
        post::Model::query()
            .include(post::user)
            .sort(user::id.desc())
            .find_one(conn)
            .await?
    );

    tracing_dbg!(
        "aggregate on joined",
        post::Model::query()
            .include(post::user)
            .filter(|| user::id.gt(2))
            .sort(post::id.desc())
            .sum(conn, user::id)
            .await?
    );

    tracing_dbg!(
        "group fk",
        post::Model::query()
            .include(post::user)
            .group(post::user)
            .count(conn, post::id)
            .await?
    );

    tracing_dbg!(
        "multiple include",
        order::Model::query()
            .include(order::user)
            .include(order::product)
            .limit(4)
            .find_all(conn)
            .await?
    );

    // optional include test
    let mut order = order::Model::query()
        .include(order::product)
        // tests for filter + sort of related entities
        .filter(|| order::product.is_not_null())
        .sort(order::product.desc())
        .find_one(conn)
        .await?
        .res()?;

    tracing_dbg!(order);

    let order_product = order.product.model().res()?;

    tracing_dbg!(order_product);

    Ok(())
}

async fn aggregate_ops(conn: &PgConnection) -> GasResult<()> {
    tracing_dbg!(
        "simple count",
        person::Model::query().count(conn, person::id).await?
    );

    tracing_dbg!(
        "simple sum",
        person::Model::query()
            .sum(conn, person::bank_account_balance)
            .await?
    );

    tracing_dbg!(
        "complex count",
        person::Model::query()
            .filter(|| person::bank_account_balance.gte(200))
            // sort and limit should be ignored for aggregates
            .sort(person::id.asc())
            .limit(4)
            .count(conn, person::phone_number)
            .await?
    );

    tracing_dbg!(
        "complex sum",
        person::Model::query()
            .filter(|| person::bank_account_balance.gte(200))
            // sort and limit should be ignored for aggregates
            .sort(person::id.asc())
            .limit(4)
            .sum(conn, person::bank_account_balance)
            .await?
    );

    tracing_dbg!(
        "simple count (grouped)",
        person::Model::query()
            .group(person::bank_account_balance)
            .count(conn, person::id)
            .await?
    );

    tracing_dbg!(
        "simple sum (grouped)",
        person::Model::query()
            .group(person::last_name)
            .sum(conn, person::bank_account_balance)
            .await?
    );

    tracing_dbg!(
        "complex count (grouped)",
        person::Model::query()
            .filter(|| person::bank_account_balance.gte(2000))
            .group(person::bank_account_balance)
            .sort(GroupSorting::Aggregate.asc() >> GroupSorting::Key.desc())
            .limit(4)
            .count(conn, person::id)
            .await?
    );

    tracing_dbg!(
        "complex sum (grouped)",
        person::Model::query()
            .filter(|| person::bank_account_balance.gte(2000))
            .group(person::last_name)
            .sort(GroupSorting::Aggregate.desc() >> person::last_name.asc())
            .limit(4)
            .sum(conn, person::bank_account_balance)
            .await?
    );
    Ok(())
}

async fn sort_limit_ops(conn: &PgConnection) -> GasResult<()> {
    tracing_dbg!(
        "sort one",
        person::Model::query()
            .sort(person::id.asc())
            .limit(4)
            .find_all(conn)
            .await?
    );

    tracing_dbg!(
        "sort two",
        person::Model::query()
            .filter(|| person::bank_account_balance.lte(2000))
            .sort(person::bank_account_balance.desc() >> person::id.asc())
            .find_all(conn)
            .await?
    );

    tracing_dbg!(
        "sort find one",
        person::Model::query()
            .sort(person::id.desc())
            .limit(4)
            .find_one(conn)
            .await?
    );

    Ok(())
}

async fn datetime_ops(conn: &PgConnection) -> GasResult<()> {
    let mut first = audit_logs::Def! {
        updated_at: NaiveDate::from_ymd_opt(2025, 10, 22).unwrap().and_hms_opt(10, 22, 40).unwrap(),
        random_date: NaiveDate::from_ymd_opt(2025, 11, 22).unwrap(),
        random_time: NaiveTime::from_hms_opt(12, 6, 20).unwrap(),
    }
    .into_model();

    first.insert(conn).await?;

    let mut second = audit_logs::Def! {
        created_at: Utc::now() - TimeDelta::days(1),
        updated_at: NaiveDate::from_ymd_opt(2025, 10, 22).unwrap().and_hms_opt(10, 22, 40).unwrap() - TimeDelta::days(1),
        random_date: NaiveDate::from_ymd_opt(2025, 11, 22).unwrap() - TimeDelta::days(1),
        random_time: NaiveTime::from_hms_opt(12, 6, 20).unwrap() - TimeDelta::hours(6),
    }.into_model();
    second.insert(conn).await?;

    let mut third = audit_logs::Def! {
        created_at: Utc::now() + TimeDelta::days(1),
        updated_at: NaiveDate::from_ymd_opt(2025, 10, 22).unwrap().and_hms_opt(10, 22, 40).unwrap() + TimeDelta::days(1),
        random_date: NaiveDate::from_ymd_opt(2025, 11, 22).unwrap() + TimeDelta::days(1),
        random_time: NaiveTime::from_hms_opt(12, 6, 20).unwrap() + TimeDelta::hours(6),
    }.into_model();
    third.insert(conn).await?;

    tracing_dbg!(
        "before now",
        audit_logs::Model::query()
            .filter(|| audit_logs::created_at.is_now_or_before())
            .find_all(conn)
            .await?
    );

    tracing_dbg!(
        "after now",
        audit_logs::Model::query()
            .filter(|| audit_logs::updated_at.is_now_or_after())
            .find_all(conn)
            .await?
    );

    tracing_dbg!(
        "before now date",
        audit_logs::Model::query()
            .filter(|| audit_logs::random_date.is_before_now())
            .find_all(conn)
            .await?
    );

    tracing_dbg!(
        "after now date",
        audit_logs::Model::query()
            .filter(|| audit_logs::random_date.is_after_now())
            .find_all(conn)
            .await?
    );

    tracing_dbg!(
        "less than time",
        audit_logs::Model::query()
            .filter(|| audit_logs::random_time.lt(NaiveTime::from_hms_opt(12, 0, 0).unwrap()))
            .find_all(conn)
            .await?
    );

    tracing_dbg!(
        "after time",
        audit_logs::Model::query()
            .filter(|| audit_logs::random_time.gt(NaiveTime::from_hms_opt(12, 0, 0).unwrap()))
            .find_all(conn)
            .await?
    );

    tracing_dbg!(
        "datetime compare different timezone",
        audit_logs::Model::query()
            .filter(|| audit_logs::created_at.gte(Local::now() - TimeDelta::minutes(1)))
            .find_all(conn)
            .await?
    );

    first.delete(conn).await?;
    second.delete(conn).await?;
    third.delete(conn).await?;

    Ok(())
}

async fn transaction_ops(conn: &PgConnection) -> GasResult<()> {
    let mut tx = conn.transaction().await?;

    let since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let mut person = person::Def! {
        first_name: String::from("Some"),
        last_name: String::from("Person"),
        email: format!("{}@person.com", since_epoch.as_secs()),
    };

    person.phone_number = Some(String::from("192"));
    person.bank_account_balance = Decimal::from(2000);
    person.insert(&mut tx).await?;

    tracing_dbg!(person);

    match rand::random_bool(0.4) {
        true => {
            tracing::warn!("saving tx");
            tx.save().await
        }
        false => {
            tracing::warn!("discarding tx");
            tx.discard().await
        }
    }?;

    tracing_dbg!(person::Model::find_by_key(&mut tx, person.id).await?);

    Ok(())
}

async fn by_key_ops(conn: &PgConnection) -> GasResult<()> {
    person::Def! {
        phone_number: Some(format!("+385{}", rand::random_range(100000000u64..999999999u64))),
        bank_account_balance: Decimal::from(rand::random_range(100u64..100000u64)),
    }
    .update_by_key(conn, 20)
    .await?;

    // test update no real fields
    let res = person::Def! { id: 20, }.update_by_key(conn, 0).await;
    let Err(GasError::InvalidInput(_)) = res else {
        return res.map(|_| ());
    };

    dbg!(&res);

    // test update invalid key
    let res = person::Def! { first_name: String::from("gaser"), }
        .update_by_key(conn, 0)
        .await;
    let Err(GasError::QueryNoResponse(_)) = res else {
        return res.map(|_| ());
    };

    dbg!(&res);

    // person::Model::delete_by_key(&conn, 4).await?;

    let id_ten = person::Model::find_by_key(conn, 20).await?;
    tracing_dbg!(id_ten);

    Ok(())
}

async fn normal_ops(conn: &PgConnection) -> GasResult<()> {
    let mut new_person = person::Def! {
        first_name: String::from("Test"),
        last_name: String::from("Test"),
        email: String::from("nonce"),
    }
    .into_model();

    tracing_dbg!("before insert", new_person);
    tracing_dbg!(
        person::Model::query()
            .filter(|| person::email.eq("nonce"))
            .find_one(conn)
            .await?
    );

    new_person.insert(conn).await?;

    tracing_dbg!("after insert", new_person);
    tracing_dbg!(
        person::Model::query()
            .filter(|| person::email.eq("nonce") & person::id.eq(new_person.id))
            .find_one(conn)
            .await?
    );

    new_person.last_name = String::from("Doe");
    new_person.bank_account_balance = Decimal::from(2000);
    new_person.update(conn).await?;

    tracing_dbg!(
        "after update",
        person::Model::query()
            .filter(|| person::email.eq("nonce") & person::id.eq(new_person.id))
            .find_one(conn)
            .await?
    );

    let persons = person::Model::query()
        .filter(|| person::email.eq("nonce"))
        .find_all(conn)
        .await?;

    tracing_dbg!(persons);
    new_person.delete(conn).await?;

    Ok(())
}
