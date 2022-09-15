use clap::Parser;
use std::env;

#[derive(Debug, Parser, PartialEq)]
#[clap(author, version, about, long_about = None)]
pub struct Config {
    #[clap(short, long, default_value = "warn")]
    pub log_level: String,
    #[clap(long, default_value = "localhost")]
    pub database_host: String,
    #[clap(long, default_value = "5432")]
    pub database_port: u16,
    #[clap(long, default_value = "rustwebdev")]
    pub database_name: String,
    #[clap(long, default_value = "user")]
    pub database_user: String,
    #[clap(short, long, default_value = "8080")]
    pub port: u16,
    #[clap(long, default_value = "password")]
    pub database_password: String,
}

impl Config {
    pub fn new() -> Result<Config, handle_errors::Error> {
        let config = Config::parse();

        if env::var("BAD_WORDS_API_URL").is_err() {
            panic!("BAD_WORDS_API_URL must be set in .env")
        }

        if env::var("BAD_WORDS_API_KEY").is_err() {
            panic!("BAD_WORDS_API_KEY must be set in .env")
        }

        if env::var("TOKEN_SECRET_KEY").is_err() {
            panic!("TOKEN_SECRET_KEY must be set in .env")
        }

        let port = std::env::var("PORT")
            .ok()
            .map(|val| val.parse::<u16>())
            .unwrap_or(Ok(config.port))
            .map_err(handle_errors::Error::ParseError)?;

        let database_user =
            env::var("POSTGRES_USER").unwrap_or_else(|_| config.database_user.to_owned());
        let database_password = env::var("POSTGRES_PASSWORD").unwrap();
        let database_host =
            env::var("POSTGRES_HOST").unwrap_or_else(|_| config.database_host.to_owned());
        let database_port =
            env::var("POSTGRES_PORT").unwrap_or_else(|_| config.database_port.to_string());
        let database_name =
            env::var("POSTGRES_DB").unwrap_or_else(|_| config.database_name.to_owned());

        Ok(Config {
            log_level: config.log_level,
            port,
            database_user,
            database_password,
            database_host,
            database_port: database_port
                .parse::<u16>()
                .map_err(handle_errors::Error::ParseError)?,
            database_name,
        })
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;

    fn set_env() {
        env::set_var("BAD_WORDS_API_URL", "http://localhost:3030");
        env::set_var("BAD_WORDS_API_KEY", "yes");
        env::set_var("TOKEN_SECRET_KEY", "7ZcbZPVuSTL4UasiGi3iwrZzWhKZadBY");
        env::set_var("POSTGRES_USER", "user");
        env::set_var("POSTGRES_PASSWORD", "pass");
        env::set_var("POSTGRES_HOST", "localhost");
        env::set_var("POSTGRES_PORT", "5432");
        env::set_var("POSTGRES_DB", "rustwebdev");
    }

    #[test]
    fn unset_and_set_api_key() {
        let result = std::panic::catch_unwind(Config::new);
        assert!(result.is_err());

        set_env();

        let expected = Config {
            log_level: "warn".to_string(),
            port: 8080,
            database_user: "user".to_string(),
            database_password: "pass".to_string(),
            database_host: "localhost".to_string(),
            database_port: 5432,
            database_name: "rustwebdev".to_string(),
        };

        let config = Config::new().unwrap();

        assert_eq!(expected, config);
    }
}
