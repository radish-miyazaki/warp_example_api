use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct APIResponse {
    message: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct BadWord {
    deviations: i64,
    info: i64,
    original: String,
    #[serde(rename = "replacedLen")]
    replaced_len: i64,
    word: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct BadWordsResponse {
    content: String,
    bad_words_total: i64,
    bad_words_list: Vec<BadWord>,
    censored_content: String,
}

/// 渡された文字列に不適切な単語が含まれていないかチェックし、含まれている場合は単語をフィルタリングして返す
pub async fn check_profanity(content: String) -> Result<String, handle_errors::Error> {
    // リトライを3回する設定を追加
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    // 上記の設定を含めたMiddlewareをHTTP Clientに適用
    let client = ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let res = client
        .post(env::var("BAD_WORDS_API_URL").expect("BAD_WORDS_API_URL must be set in .env"))
        .header(
            "apikey",
            env::var("BAD_WORDS_API_KEY").expect("BAD_WORDS_API_KEY must be set in .env"),
        )
        .body(content)
        .send()
        .await
        .map_err(handle_errors::Error::MiddlewareReqwestAPIError)?;

    // API失敗時の処理
    if !res.status().is_success() {
        let status = res.status().as_u16();
        let message = res.json::<APIResponse>().await.unwrap().message;

        let err = handle_errors::APILayerError { status, message };

        if status < 500 {
            return Err(handle_errors::Error::ClientError(err));
        } else {
            return Err(handle_errors::Error::ServerError(err));
        }
    }

    match res.json::<BadWordsResponse>().await {
        Ok(res) => Ok(res.censored_content),
        Err(e) => Err(handle_errors::Error::RequestAPIError(e)),
    }
}
