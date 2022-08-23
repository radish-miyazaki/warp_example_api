use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use warp::body::BodyDeserializeError;
use warp::cors::CorsForbidden;
use warp::hyper::{Method, StatusCode};
use warp::{reject::Reject, Filter, Rejection, Reply};

#[derive(Clone)]
struct Store {
    questions: Arc<RwLock<HashMap<QuestionId, Question>>>,
}

impl Store {
    fn new() -> Self {
        Store {
            questions: Arc::new(RwLock::new(Self::init())),
        }
    }

    fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!("../questions.json");
        serde_json::from_str(file).expect("can't read questions.json")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Eq, Clone, PartialEq, Hash, Deserialize)]
struct QuestionId(String);

#[derive(Debug)]
struct InvalidId;
impl Reject for InvalidId {}

#[derive(Debug)]
struct Pagination {
    start: usize,
    end: usize,
}

#[derive(Debug)]
enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameters,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Error::ParseError(ref err) => write!(f, "Cannot parse parameter: {}", err),
            Error::MissingParameters => write!(f, "Missing parameter"),
        }
    }
}

impl Reject for Error {}

fn extract_pagination(params: HashMap<String, String>) -> Result<Pagination, Error> {
    if params.contains_key("start") && params.contains_key("end") {
        return Ok(Pagination {
            start: params
                .get("start")
                .unwrap()
                .parse::<usize>()
                .map_err(Error::ParseError)?,
            end: params
                .get("end")
                .unwrap()
                .parse::<usize>()
                .map_err(Error::ParseError)?,
        });
    }

    Err(Error::MissingParameters)
}

async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    if !params.is_empty() {
        let pagination = extract_pagination(params)?;
        let res: Vec<Question> = store.questions.read().values().cloned().collect();
        let res = &res.get(pagination.start..pagination.end);

        Ok(warp::reply::json(&res.unwrap_or(&[])))
    } else {
        let res: Vec<Question> = store.questions.read().values().cloned().collect();
        Ok(warp::reply::json(&res))
    }
}

async fn add_question(
    question: Question,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    store
        .questions
        .write()
        .insert(question.id.clone(), question);

    Ok(warp::reply::with_status("Question added", StatusCode::OK))
}

async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(error) = r.find::<Error>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::RANGE_NOT_SATISFIABLE,
        ))
    } else if let Some(error) = r.find::<CorsForbidden>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else if let Some(error) = r.find::<BodyDeserializeError>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}

#[tokio::main]
async fn main() {
    // TODO: Store構造体をDBモックとする
    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());

    // CORS
    // TODO: CORSによるエラーはここで落としてくれるので、`return_error`で別途自前で用意する必要はなさそう
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
        .and_then(get_questions);

    // POST /questions
    let add_question = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::body::json()) // 第1引数にハンドラの引数にリクエストボディ
        .and(store_filter.clone()) // 第2引数にStore
        .and_then(add_question);

    let routes = get_questions
        .or(add_question)
        .with(cors)
        .recover(return_error);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
