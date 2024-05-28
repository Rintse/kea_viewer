mod config;
mod lease;
mod server;
use crate::server::handler;
use log::info;

fn main() {
    let cfg = config::Settings::new();
    let ctx = server::Context::new(cfg.clone());

    env_logger::init();
    info!("Hosting server on http://{}", cfg.bind_addr);
    rouille::start_server(cfg.bind_addr, move |r| handler(&ctx, r));
}
