use anyhow::Context;
use async_sqlx_session::SqliteSessionStore;
use dotenv;
use sqlx::sqlite::SqlitePool;
use std::{env, time::Duration};
use tide::{
    http::{ensure, format_err},
    sessions::SessionMiddleware,
    Redirect,
};

pub mod records;
mod templates;

mod routes;
mod utils;

#[derive(Clone)]
pub struct State {
    db: SqlitePool,
}

pub type Request = tide::Request<State>;

async fn db_connection() -> tide::Result<SqlitePool> {
    // let database_url = env::var("DATABASE_URL").expect("No DATABASE_URL set");
    let database_url = env::var("DATABASE_URL").context("get database_url")?;
    Ok(SqlitePool::new(&database_url).await?)
}

async fn build_session_middleware(
    db: SqlitePool,
) -> tide::Result<SessionMiddleware<SqliteSessionStore>> {
    let session_store = SqliteSessionStore::from_client(db);
    session_store.migrate().await?;
    session_store.spawn_cleanup_task(Duration::from_secs(60 * 15));
    let session_secret = env::var("TIDE_SECRET").unwrap();
    Ok(SessionMiddleware::new(
        session_store,
        session_secret.as_bytes(),
    ))
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    tide::log::with_level(tide::log::LevelFilter::Info);
    dotenv::dotenv().ok();

    let port = env::var("TIDE_PORT").context("get TIDE_PORT")?;
    let database_url = env::var("DATABASE_URL")?;
    let tide_secret = env::var("TIDE_SECRET").context("get TIDE_SECRET")?;

    println!("Running with environment variables:");
    println!("DATABASE_URL={}", database_url);
    println!("TIDE_PORT={}", port);
    println!("TIDE_SECRET={}", tide_secret);
    ensure!(
        env::var("DATABASE_URL").is_ok(),
        "DATABASE_URL NOT DEFINED"
    );

    let db = db_connection().await?;
    let mut app = tide::with_state(State { db: db.clone() });

    app.with(build_session_middleware(db).await?);

    app.at("/").get(Redirect::new("/welcome"));

    app.at("/welcome").get(routes::welcome);

    let mut articles = app.at("/articles");

    articles
        .post(routes::articles::create)
        .get(routes::articles::index);

    articles.at("/new").get(routes::articles::new);

    articles
        .at("/:article_id")
        .get(routes::articles::show)
        .delete(routes::articles::delete)
        .put(routes::articles::update)
        .post(routes::articles::update);

    app.listen(format!("http://0.0.0.0:{}", env::var("TIDE_PORT")?))
        .await?;
    Ok(())
}
