use crate::models::{audit_logs, person};
use crate::tracing_util::setup_tracing;
use gas::connection::PgConnection;
use gas::eq::{PgEq, PgEqTime};
use gas::error::GasError;
use gas::types::{Local, NaiveDate, NaiveTime, TimeDelta, Utc};
use gas::{GasResult, ModelOps};
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
