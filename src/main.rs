use std::env;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    if let Err(error) = ibkr_flex_cli::run(env::args().skip(1).collect()).await {
        eprintln!("Erreur : {error}");
        std::process::exit(1);
    }
}
