use handle_errors::Error;
use std::collections::HashMap;
use tracing::{event, instrument, Level};
use warp::http::StatusCode;

use crate::store::Store;
use crate::types::pagination::{extract_pagination, Pagination};
use crate::types::question::{NewQuestion, Question, QuestionId};

#[instrument]
pub async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    event!(target: "question_and_answer", Level::INFO, "querying questions");
    let mut pagination = Pagination::default();

    if !params.is_empty() {
        event!(Level::INFO, pagination = true);
        pagination = extract_pagination(params)?;
    }

    let res: Vec<Question> = match store
        .get_questions(pagination.limit, pagination.offset)
        .await
    {
        Ok(res) => res,
        Err(e) => return Err(warp::reject::custom(Error::DatabaseQueryError)),
    };

    Ok(warp::reply::json(&res))
}

#[instrument]
pub async fn get_question(id: i32, store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    let res: Question = match store.get_question(id).await {
        Ok(res) => res,
        Err(e) => return Err(warp::reject::custom(Error::DatabaseQueryError)),
    };

    Ok(warp::reply::json(&res))
}

#[instrument]
pub async fn add_question(
    new_question: NewQuestion,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    if let Err(e) = store.add_question(new_question).await {
        return Err(warp::reject::custom(Error::DatabaseQueryError));
    }

    Ok(warp::reply::with_status("Question added", StatusCode::OK))
}

#[instrument]
pub async fn update_question(
    id: i32,
    question: Question,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    match store.questions.write().get_mut(&QuestionId(id)) {
        Some(q) => *q = question,
        None => return Err(warp::reject::custom(Error::QuestionNotFound)),
    }

    Ok(warp::reply::with_status("Question updated", StatusCode::OK))
}

#[instrument]
pub async fn delete_question(id: i32, store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    match store.questions.write().remove(&QuestionId(id)) {
        Some(_) => Ok(warp::reply::with_status("Question deleted", StatusCode::OK)),
        None => Err(warp::reject::custom(Error::QuestionNotFound)),
    }
}
