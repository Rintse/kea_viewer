mod config;
mod lease;
mod viewer;
use crate::viewer::LeasesTemplate;
use askama::Template;
use rouille::Request;
use rouille::Response;

const FAVICON_PATH: &str = "assets/favicon.png";

/// All the fields that we keep track of during execution of the server
struct Context {
    settings: config::Settings,
}

impl Context {
    fn new(settings: config::Settings) -> Self {
        Self { settings, }
    }
}

fn leases_handler(ctx: &Context) -> Response {
    let leases = lease::parse_file(&ctx.settings.leases_file);

    match leases {
        Ok(leases) => {
            let template = LeasesTemplate { leases };
            // TODO: handle error
            let html = template.render().unwrap();
            Response::html(html)
        }
        Err(e) => Response::text(format!("Error: {e}")).with_status_code(500),
    }
}

fn favicon_handler() -> Response {
    let favicon_f = std::fs::File::open(FAVICON_PATH);
    match favicon_f {
        Ok(file) => Response::from_file("image/x-icon", file),
        Err(_) => Response::text("").with_status_code(500),
    }
}

fn handler(ctx: &Context, request: &Request) -> Response {
    eprintln!(
        "Got {} request at {} from {:?}",
        request.method(),
        request.url(),
        request.remote_addr()
    );

    match request.url().as_str() {
        "/" => leases_handler(ctx),
        "/favicon.ico" => favicon_handler(),
        _ => Response::empty_404(),
    }
}

fn main() {
    let cfg = config::Settings::new();
    let ctx = Context::new(cfg.clone());

    eprintln!("Hosting server on http://{}", cfg.bind_addr);
    rouille::start_server(cfg.bind_addr, move |r| handler(&ctx, r));
}
