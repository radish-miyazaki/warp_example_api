#![warn(clippy::all)]
use handle_errors::return_error;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::hyper::Method;
use warp::Filter;

mod routes;
mod store;
mod types;

#[tokio::main]
async fn main() {
    // TODO: Store構造体をDBモックとする
    let store = store::Store::new();
    let store_filter = warp::any().map(move || store.clone());

    // CORS
    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[Method::PUT, Method::DELETE, Method::GET, Method::POST]);

    // Logging & Tracing
    let log_filter = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "question_and_answer=info,warp=error".to_owned()); // ログレベルの追加
    tracing_subscriber::fmt()
        .with_env_filter(log_filter)
        .with_span_events(FmtSpan::CLOSE)
        .init(); // tracing-subscriberのセット

    // GET /questions
    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query()) // ハンドラの第1引数にクエリパラメータ
        .and(store_filter.clone()) // 第2引数にStore 第3引数にエラー用のUUID
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
        .and(warp::body::json()) // 第1引数にハンドラの引数にリクエストボディ
        .and(store_filter.clone()) // 第2引数にStore
        .and_then(routes::question::add_question);

    // PUT /questions/:question_id
    let update_question = warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end())
        .and(warp::body::json())
        .and(store_filter.clone())
        .and_then(routes::question::update_question);

    // DELETE /questions/:question_id
    let delete_question = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::question::delete_question);

    // POST /answers (x-www-form-urlencoded)
    // INFO: /questions/:question_id/answers にルートを変更
    let add_answer = warp::post()
        .and(warp::path("answers"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::form())
        .and_then(routes::answer::add_answer);

    let routes = get_questions
        .or(get_question)
        .or(add_question)
        .or(update_question)
        .or(delete_question)
        .or(add_answer)
        .with(cors)
        .with(warp::trace::request())
        .recover(return_error);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
