mod config;
mod lease;
mod viewer;
use std::cmp;
use std::net::Ipv4Addr;

use chrono::DateTime;
use lease::FileParseError;
use rouille::Request;
use rouille::Response;
use crate::lease::Lease;
use askama::Template;

const FAVICON_PATH: &str = "assets/favicon.png";

#[derive(Template)]
#[template(path = "leases.html")]
pub struct LeasesTemplate {
    pub leases: Vec<Lease>,
}
// TODO: logging

/// All the fields that we keep track of during execution of the server
struct Context {
    settings: config::Settings,
}

impl Context {
    fn new(settings: config::Settings) -> Self {
        Self { settings, }
    }
}

// The parameters that may be added to the GET request on the root
struct RequestParams {
    sort_on: Option<String>,
    sort_desc: bool,
}

impl RequestParams {
    fn from_request(req: &Request) -> Self {
        RequestParams {
            sort_on: req.get_param("sort"),
            sort_desc: req.get_param("order_desc").is_some(),
        }
    }
}

fn preprocess_leases(leases: &mut [Lease], params: &RequestParams) {
    if let Some(field) = &params.sort_on {
        eprintln!("Sorting on {field}");
        let cmp = match field.as_str() {
            "ip" => |a: &Lease, b: &Lease| Ipv4Addr::cmp(&a.ip_addr, &b.ip_addr),
            "mac" => |a: &Lease, b: &Lease| String::cmp(&a.hw_addr, &b.hw_addr),
            "exp" => |a: &Lease, b: &Lease| DateTime::cmp(&a.expires, &b.expires),
            "name" => |a: &Lease, b: &Lease| Option::cmp(&a.hostname, &b.hostname),
            _ => todo!(),
        };

        if params.sort_desc {
            eprintln!("In descending order");
            leases.sort_by(|a, b| cmp(b, a));
        }
        else {
            eprintln!("In ascending order");
            leases.sort_by(cmp);
        }
    }
}

fn leases_handler(ctx: &Context, params: RequestParams) -> Response {
    // For a directory, look at all files
    let leases = if ctx.settings.leases_file.is_dir() {
        let files = std::fs::read_dir(&ctx.settings.leases_file);
        match files {
            Ok(files) => {
                let results: Result<Vec<Vec<Lease>>, FileParseError> = files
                    .filter_map(|r| r.ok())
                    .map(|r| lease::parse_file(&r.path()))
                    .collect();
                results.map(|r| r.into_iter().flatten().collect())
            },
            Err(e) => return Response::text(format!("Error: {e}")).with_status_code(500),
        }
    } 
    // For a file, read just the one
    else if ctx.settings.leases_file.is_file() {
        lease::parse_file(&ctx.settings.leases_file)
    } 
    else {
        unreachable!("right?");
    };

    match leases {
        Ok(mut leases) => {
            preprocess_leases(&mut leases, &params);
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

    let params = RequestParams::from_request(request);

    match request.url().as_str() {
        "/" => leases_handler(ctx, params),
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
