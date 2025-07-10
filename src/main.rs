fn main() {
    if let Err(err) = dotenvy::dotenv() {
        if !err.not_found() {
            panic!("Error while loading .env file: {err}")
        }
    }

    let logger_env = env_logger::Env::default()
        .filter_or("LOG_LEVEL", "debug")
        .write_style_or("LOG_STYLE", "always");
    env_logger::init_from_env(logger_env);

    log::info!("Hello, world!");
}
