use std::collections::HashMap;
use warp::http::StatusCode;

use crate::store::Store;
use crate::types::{
    answer::{Answer, AnswerId},
    question::QuestionId,
};

pub async fn add_answer(
    store: Store,
    params: HashMap<String, String>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let answer = Answer {
        // TODO: ランダムな数値を生成する
        id: AnswerId("1".to_string()),
        // TODO: エラーチェックの追加
        content: params.get("content").unwrap().to_string(),
        question_id: QuestionId(params.get("questionId").unwrap().to_string()),
    };

    // TODO: 質問が存在するかチェック

    store.answers.write().insert(answer.id.clone(), answer);

    Ok(warp::reply::with_status("Answer added", StatusCode::OK))
}
