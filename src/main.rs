#![warn(clippy::all)]

use handle_errors::return_error;
use warp::hyper::Method;
use warp::Filter;

mod routes;
mod store;
mod types;

#[tokio::main]
async fn main() {
    // INFO: Store構造体をDBモックとする
    let store = store::Store::new();
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
        .and(warp::query()) // ハンドラの第1引数にクエリパラメータ
        .and(store_filter.clone()) // 第2引数にStore
        .and_then(routes::question::get_questions);

    // GET /questions/:question_id
    let get_question = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
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
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::body::json())
        .and(store_filter.clone())
        .and_then(routes::question::update_question);

    // DELETE /questions/:question_id
    let delete_question = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
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
        .recover(return_error);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
