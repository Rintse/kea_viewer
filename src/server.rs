use crate::lease::Lease;
use crate::lease::{self, FileParseError};
use askama::Template;
use chrono::DateTime;
use log::{debug, info, warn};
use rouille::Request;
use rouille::Response;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};

#[derive(Template)]
#[template(path = "leases.html")]
pub struct LeasesTemplate {
    pub leases: Vec<Lease>,
}

/// All the fields that we keep track of during execution of the server
pub struct Context {
    settings: crate::config::Settings,
}

impl Context {
    pub fn new(settings: crate::config::Settings) -> Self {
        Self { settings }
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

fn cmp_from_str(
    field: &str,
) -> Option<
    impl for<'a, 'b> Fn(&'a lease::Lease, &'b lease::Lease) -> std::cmp::Ordering,
> {
    let cmp = match field {
        "ip" => |a: &Lease, b: &Lease| Ipv4Addr::cmp(&a.ip_addr, &b.ip_addr),
        "mac" => |a: &Lease, b: &Lease| String::cmp(&a.hw_addr, &b.hw_addr),
        "exp" => |a: &Lease, b: &Lease| DateTime::cmp(&a.expires, &b.expires),
        "name" => |a: &Lease, b: &Lease| Option::cmp(&a.hostname, &b.hostname),
        _ => return None,
    };

    Some(cmp)
}

fn preprocess_leases(leases: &mut Vec<Lease>, params: &RequestParams) {
    // Only list the most recent lease for each HW address
    leases.sort_by(|a, b| {
        String::cmp(&a.hw_addr, &b.hw_addr)
            .then(DateTime::cmp(&a.expires, &b.expires))
    });
    leases.dedup_by(|a, b| a.hw_addr == b.hw_addr);

    if let Some(field) = &params.sort_on {
        debug!("Sorting on {field}");
        match cmp_from_str(field) {
            Some(cmp) => {
                if params.sort_desc {
                    debug!("Sorting in descending order");
                    leases.sort_by(|a, b| cmp(b, a));
                } else {
                    debug!("Sorting in ascending order");
                    leases.sort_by(cmp);
                }
            }
            None => warn!("Invalid sort field: {field}"),
        }
    }
}

fn files_in_dir(dir: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let files = std::fs::read_dir(dir);
    files.map(|files| files.filter_map(|r| r.ok()).map(|r| r.path()).collect())
}

fn leases_handler(ctx: &Context, params: RequestParams) -> Response {
    // For a directory, look at all files
    let leases = if ctx.settings.leases_db.is_dir() {
        let files = files_in_dir(&ctx.settings.leases_db);
        match files {
            Ok(files) => {
                let results: Result<Vec<Vec<Lease>>, FileParseError> =
                    files.iter().map(|f| crate::lease::parse_file(f)).collect();
                results.map(|r| r.into_iter().flatten().collect())
            }
            Err(e) => {
                return Response::text(format!("Error: {e}"))
                    .with_status_code(500)
            }
        }
    }
    // For a file, read just the one
    else if ctx.settings.leases_db.is_file() {
        lease::parse_file(&ctx.settings.leases_db)
    } else {
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
    let favicon_bytes = include_bytes!("../assets/favicon.png");
    Response::from_data("image/x-icon", favicon_bytes)
}

pub fn handler(ctx: &Context, request: &Request) -> Response {
    info!(
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
