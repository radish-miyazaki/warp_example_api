use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Eq, Clone, PartialEq, Hash, Deserialize)]
pub struct QuestionId(pub i32);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Question {
    pub id: QuestionId,
    pub title: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
}
