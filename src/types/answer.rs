use serde::{Deserialize, Serialize};

use crate::types::question::QuestionId;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct AnswerId(pub i32);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Answer {
    pub id: AnswerId,
    pub content: String,
    pub question_id: QuestionId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewAnswer {
    pub content: String,
    pub question_id: QuestionId,
}
