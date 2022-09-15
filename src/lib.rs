#![warn(clippy::all)]
pub use handle_errors;

use tokio::sync::{oneshot, oneshot::Sender};
use tracing_subscriber::fmt::format::FmtSpan;
use warp::hyper::Method;
use warp::Filter;

pub mod config;
mod profanity;
mod routes;
mod store;
mod types;

async fn build_routes(store: store::Store) -> impl Filter<Extract = impl warp::Reply> + Clone {
    // INFO: storeをmapのコールバック内に所有権を移動しているので、各storeの操作が終わった後にfilter化
    let store_filter = warp::any().map(move || store.clone());

    // CORS
    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[Method::PUT, Method::DELETE, Method::GET, Method::POST]);

    // GET /questions
    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter.clone())
        .and_then(routes::question::get_questions)
        .with(warp::trace(|info| {
            tracing::info_span!(
                "get_questions request",
                method = %info.method(),
                path = %info.path(),
                id = %uuid::Uuid::new_v4()
            )
        }));

    // GET /questions/:question_id
    let get_question = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::question::get_question);

    // POST /questions
    let add_question = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(store_filter.clone())
        .and(routes::authentication::auth())
        .and_then(routes::question::add_question);

    // PUT /questions/:question_id
    let update_question = warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end())
        .and(warp::body::json())
        .and(store_filter.clone())
        .and(routes::authentication::auth())
        .and_then(routes::question::update_question);

    // DELETE /questions/:question_id
    let delete_question = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(routes::authentication::auth())
        .and_then(routes::question::delete_question);

    // POST /answers (x-www-form-urlencoded)
    // INFO: /questions/:question_id/answers にルートを変更
    let add_answer = warp::post()
        .and(warp::path("answers"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::form())
        .and(routes::authentication::auth())
        .and_then(routes::answer::add_answer);

    // POST /registration
    let registration = warp::post()
        .and(warp::path("registration"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::authentication::register);

    // POST /login
    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::path::end())
        .and(store_filter)
        .and(warp::body::json())
        .and_then(routes::authentication::login);

    get_questions
        .or(get_question)
        .or(add_question)
        .or(update_question)
        .or(delete_question)
        .or(add_answer)
        .or(registration)
        .or(login)
        .with(cors)
        .with(warp::trace::request())
        .recover(handle_errors::return_error)
}

pub async fn setup_store(config: &config::Config) -> Result<store::Store, handle_errors::Error> {
    // Database
    let store = store::Store::new(&format!(
        "postgres://{}:{}@{}:{}/{}",
        config.database_user,
        config.database_password,
        config.database_host,
        config.database_port,
        config.database_name
    ))
    .await;

    // Migration
    // INFO: ディレクトリを指定しないと、ALTER TABLEが効かなかったので追加
    sqlx::migrate!("./migrations")
        .run(&store.clone().conn)
        .await
        .expect("Cannnot run migration");

    // Logging & Tracing
    // INFO: ログレベルを各モジュールごとにセット
    // 当アプリケーション(question_and_answer) / warp内部 / 自作モジュール(handler_errors)内部
    let log_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| {
        format!(
            "handle_errors={},question_and_answer={},warp={}",
            config.log_level, config.log_level, config.log_level
        )
    });

    // INFO: ログやトレースをどう扱うを決めるサブスクライバーを定義
    tracing_subscriber::fmt()
        .with_env_filter(log_filter)
        .with_span_events(FmtSpan::CLOSE)
        .init(); // tracing-subscriberのセット

    Ok(store)
}

pub async fn run(config: config::Config, store: store::Store) {
    let routes = build_routes(store).await;
    warp::serve(routes).run(([0, 0, 0, 0], config.port)).await;
}

pub struct OneshotHandler {
    pub sender: Sender<i32>,
}

/// 統合テスト用に瞬間的に本番と同じ環境のサーバを立ち上げる関数
pub async fn oneshot(store: store::Store) -> OneshotHandler {
    let routes = build_routes(store).await;
    let (tx, rx) = oneshot::channel::<i32>();

    let socket: std::net::SocketAddr = "127.0.0.1:3030"
        .to_string()
        .parse()
        .expect("Not a valid address");

    let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(socket, async {
        rx.await.ok();
    });

    tokio::task::spawn(server);

    OneshotHandler { sender: tx }
}
