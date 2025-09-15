mod data;
mod solver;
mod server;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();


    server::run_server().await;


}
