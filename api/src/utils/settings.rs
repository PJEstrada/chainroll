use dotenvy::dotenv;
use lazy_static::lazy_static;
use secrecy::SecretString;
use std::env as std_env;

lazy_static! {
    pub static ref DATABASE_URL: SecretString = set_db_url();
}

fn set_db_url() -> SecretString {
    dotenv().ok();
    let url = std_env::var(env::DATABASE_URL).expect("DATABASE_URL must be set");
    if url.is_empty() {
        panic!("DATABASE_URL is empty");
    }
    SecretString::new(url.into_boxed_str())
}

pub mod env {
    #[allow(dead_code)]
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const DATABASE_URL: &str = "DATABASE_URL";
}

pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:3000";
}

#[allow(dead_code)]
pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";
}
