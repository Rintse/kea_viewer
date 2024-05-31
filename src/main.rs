mod config;
mod lease;
mod server;
use crate::server::handler;
use log::error;
use log::info;

fn main() {
    env_logger::init();

    let ctx = match config::Settings::new() {
        Ok(cfg) => server::Context::new(cfg.clone()),
        Err(e) => {
            error!("{e}");
            return;
        }
    };

    info!("Hosting server on http://{}", ctx.settings.bind_addr);
    rouille::start_server(ctx.settings.bind_addr, move |r| handler(&ctx, r));
}
