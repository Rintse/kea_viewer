use crate::lease::Lease;
use askama::Template;

#[derive(Template)]
#[template(path = "leases.html")]
pub struct LeasesTemplate {
    pub leases: Vec<Lease>,
}
