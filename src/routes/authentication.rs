use argon2::Config;
use chrono::prelude::*;
use rand::Rng;
use std::env;
use warp::http::StatusCode;
use warp::Filter;

use crate::store::Store;
use crate::types::account::{Account, AccountId, Session};

pub async fn register(store: Store, account: Account) -> Result<impl warp::Reply, warp::Rejection> {
    let hashed_password = hash(account.password.as_bytes());

    let account = Account {
        id: account.id,
        email: account.email,
        password: hashed_password,
    };

    match store.add_account(account).await {
        Ok(_) => Ok(warp::reply::with_status("Account added", StatusCode::OK)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

pub async fn login(store: Store, login: Account) -> Result<impl warp::Reply, warp::Rejection> {
    // データベースにユーザが存在するかチェック
    match store.get_account(login.email).await {
        // パスワードが正しいかチェック
        Ok(account) => match verify_password(&account.password, login.password.as_bytes()) {
            Ok(verified) => {
                if verified {
                    Ok(warp::reply::json(&issue_token(
                        account.id.expect("id not found"),
                    )))
                } else {
                    Err(warp::reject::custom(handle_errors::Error::WrongPassword))
                }
            }
            Err(e) => Err(warp::reject::custom(
                handle_errors::Error::ArgonLibraryError(e),
            )),
        },
        Err(e) => Err(warp::reject::custom(e)),
    }
}

pub fn verify_token(token: String) -> Result<Session, handle_errors::Error> {
    let secret_key = env::var("TOKEN_SECRET_KEY").unwrap();
    let token = paseto::tokens::validate_local_token(
        &token,
        None,
        &Vec::from(secret_key),
        &paseto::tokens::TimeBackend::Chrono,
    )
    .map_err(|_| handle_errors::Error::CannotDecryptToken)?;

    serde_json::from_value::<Session>(token).map_err(|_| handle_errors::Error::CannotDecryptToken)
}

/// 戻り値の型はwarp::Filter::andメソッドに併せてセット
pub fn auth() -> impl Filter<Extract = (Session,), Error = warp::Rejection> + Clone {
    // warp::Filter::and_thenメソッドにセットする関数は非同期である必要があるのでasyncを付与
    // @ref https://docs.rs/warp/0.3.1/warp/trait.Filter.html#method.and_then
    warp::header::<String>("Authorization").and_then(|token: String| async move {
        let token = match verify_token(token) {
            Ok(t) => t,
            Err(_) => return Err(warp::reject::reject()),
        };

        Ok(token)
    })
}

pub fn hash(password: &[u8]) -> String {
    let salt = rand::thread_rng().gen::<[u8; 32]>();
    let config = Config::default();
    argon2::hash_encoded(password, &salt, &config).unwrap()
}

fn verify_password(hash: &str, password: &[u8]) -> Result<bool, argon2::Error> {
    argon2::verify_encoded(hash, password)
}

fn issue_token(account_id: AccountId) -> String {
    // 有効期限を1日にセット
    let current_date_time = Utc::now();
    let dt = current_date_time + chrono::Duration::days(1);

    // トークン生成時の秘密鍵を.envから取得
    let secret_key = env::var("TOKEN_SECRET_KEY").unwrap();

    paseto::tokens::PasetoBuilder::new()
        .set_encryption_key(&Vec::from(secret_key))
        .set_expiration(&dt)
        .set_not_before(&Utc::now())
        .set_claim("account_id", serde_json::json!(account_id))
        .build()
        .expect("Failed to construct paseto token w/ builder!")
}

#[cfg(test)]
mod authentication_tests {
    use super::{auth, env, issue_token, AccountId};

    #[tokio::test]
    async fn post_questions_auth() {
        env::set_var("TOKEN_SECRET_KEY", "7ZcbZPVuSTL4UasiGi3iwrZzWhKZadBY");
        let token = issue_token(AccountId(3));

        let filter = auth();

        let res = warp::test::request()
            .header("Authorization", token)
            .filter(&filter);

        assert_eq!(res.await.unwrap().account_id, AccountId(3));
    }
}
