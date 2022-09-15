use question_and_answer::{config, run, setup_store};

#[tokio::main]
async fn main() -> Result<(), handle_errors::Error> {
    // .envファイル読み込み
    dotenv::dotenv().ok();

    let config = config::Config::new().expect("Config can't be set");
    let store = setup_store(&config).await?;

    tracing::info!(
        "Q&A service build ID {}",
        env!("QUESTION_AND_ANSWER_VERSION")
    );

    run(config, store).await;

    Ok(())
}
