use sqlx::{postgres::PgPoolOptions, types::time::Date};

#[derive(Debug)]
struct SimpleTable {
    id: i32,
}

pub async fn hello_db() {
    let db_url =
        std::env::var("DATABASE_URL").expect("Env var DATABASE_URL is required for this example.");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();

    // Query all rows from the table
    let rows: Vec<SimpleTable> = sqlx::query_as!(SimpleTable, r#"SELECT id FROM simple"#)
        .fetch_all(&pool)
        .await
        .unwrap();

    tracing::debug!("{:?}", rows);
}
