use sqlx::{
    postgres::{PgPool, PgPoolOptions, PgRow},
    Row,
};

use crate::types::question::{NewQuestion, Question, QuestionId};
use handle_errors::Error;

#[derive(Clone, Debug)]
pub struct Store {
    pub conn: PgPool,
}

impl Store {
    pub async fn new(db_url: &str) -> Self {
        let db_pool = match PgPoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await
        {
            Ok(pool) => pool,
            Err(e) => panic!("Couldn't establish DB connection!"),
        };

        Store { conn: db_pool }
    }

    pub async fn get_questions(
        &self,
        limit: Option<u32>,
        offset: u32,
    ) -> Result<Vec<Question>, Error> {
        match sqlx::query("SELECT * FROM questions LIMIT $1 OFFSET $2")
            .bind(limit.unwrap_or_default() as i32)
            .bind(offset as i32)
            .map(|row: PgRow| Question {
                id: QuestionId(row.get("id")),
                title: row.get("title"),
                content: row.get("content"),
                tags: row.get("tags"),
            })
            .fetch_all(&self.conn)
            .await
        {
            Ok(questions) => Ok(questions),
            Err(e) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", e);
                Err(Error::DatabaseQueryError)
            }
        }
    }

    pub async fn get_question(&self, id: i32) -> Result<Question, Error> {
        match sqlx::query("SELECT * FROM questions WHERE id = $1")
            .bind(id)
            .map(|row: PgRow| Question {
                id: QuestionId(row.get("id")),
                title: row.get("title"),
                content: row.get("content"),
                tags: row.get("rows"),
            })
            .fetch_one(&self.conn)
            .await
        {
            Ok(question) => Ok(question),
            Err(e) => Err(Error::DatabaseQueryError),
        }
    }

    pub async fn add_question(self, new_question: NewQuestion) -> Result<Question, Error> {
        match sqlx::query("INSERT INTO questions (title, content, tags) VALUES ($1, $2, $3) RETURNING id, title, content, tags")
            .bind(new_question.title)
            .bind(new_question.content)
            .bind(new_question.tags)
            .map(|row: PgRow| Question {
                id: QuestionId(row.get("id")),
                title: row.get("title"),
                content: row.get("content"),
                tags: row.get("rows")
            })
            .fetch_one(&self.conn)
            .await {
                Ok(question) => Ok(question),
                Err(e) => Err(Error::DatabaseQueryError)
            }
    }
}
