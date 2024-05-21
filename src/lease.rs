use chrono::{DateTime, Utc};
use std::time::Duration;
use std::{
    fs, io,
    net::{AddrParseError, Ipv4Addr},
    path::Path,
};

#[derive(Debug)]
pub struct Lease {
    pub ip_addr: Ipv4Addr,
    pub hw_addr: String,
    pub client_id: Option<String>,
    pub lifetime: Duration,
    pub expires: DateTime<Utc>,
    pub subnet_id: u32,
    pub fqdn_fwd: u32,
    pub fqdn_rev: u32,
    pub hostname: Option<String>,
    pub state: u32,
    pub user_context: Option<String>,
    pub pool_id: u32,
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("Not enough fields in CSV line. Expected 12. Got {0}.")]
    NotEnoughFields(usize),

    #[error("Error parsing IP address:\n{source}")]
    IpAddr {
        #[from]
        source: AddrParseError,
    },
    
    #[error("Invalid lease duration: {0}")]
    Duration(String),
    
    #[error("Invalid lease expiration time: {0}")]
    Time(String),
    
    #[error("Invalid subnet id: {0}")]
    SubnetId(String),
    
    #[error("Invalid forward FQDN: {0}")]
    FqdnFwd(String),
    
    #[error("Invalid reverse FQDN: {0}")]
    FqdnRev(String),

    #[error("Invalid state: {0}")]
    State(String),
    
    #[error("Invalid pool ID")]
    PoolId(String),
}

#[derive(thiserror::Error, Debug)]
pub enum FileParseError {
    #[error("Could not open file:\n{source}")]
    FileOpen {
        #[from]
        source: io::Error,
    },

    #[error("Empty leases file")]
    Empty,

    #[error("Error parsing lease:\n{source}")]
    ParseLease {
        source: ParseError,
        line: usize,
    },
}

impl From<(usize, ParseError)> for FileParseError {
    fn from((line, source): (usize, ParseError)) -> Self {
        Self::ParseLease { source, line }
    }
}

fn empty_str_to_none(s: &str) -> Option<String> {
    match s {
        "" => None,
        other => Some(other.to_owned()),
    }
}

fn parse_duration(s: &str) -> Result<Duration, ParseError> {
    let secs: u64 = s.parse::<u64>()
        .map_err(|_| ParseError::Duration(s.to_owned()))?;
    Ok(Duration::from_secs(secs))
}

fn parse_datetime(s: &str) -> Result<DateTime<Utc>, ParseError> {
    let secs: i64 = s.parse::<i64>()
        .map_err(|_| ParseError::Duration(s.to_owned()))?;
    DateTime::from_timestamp(secs, 0)
        .ok_or(ParseError::Time(s.to_owned()))
}

fn parse_subnet_id(s: &str) -> Result<u32, ParseError> {
    s.parse::<u32>().map_err(|_| ParseError::SubnetId(s.to_owned()))
}

fn parse_fqdn_fwd(s: &str) -> Result<u32, ParseError> {
    s.parse::<u32>().map_err(|_| ParseError::FqdnFwd(s.to_owned()))
}

fn parse_fqdn_rev(s: &str) -> Result<u32, ParseError> {
    s.parse::<u32>().map_err(|_| ParseError::FqdnRev(s.to_owned()))
}

fn parse_state(s: &str) -> Result<u32, ParseError> {
    s.parse::<u32>().map_err(|_| ParseError::State(s.to_owned()))
}

fn parse_pool_id(s: &str) -> Result<u32, ParseError> {
    s.parse::<u32>().map_err(|_| ParseError::PoolId(s.to_owned()))
}


pub fn parse_line(line: &str) -> Result<Lease, ParseError> {
    let mut fields = line.split(',');
    let mut idx: usize = 0;
    let mut next_field = || {
        idx += 1;
        fields.next().ok_or(ParseError::NotEnoughFields(idx-1)) 
    };

    let ip_addr = next_field()?.parse::<Ipv4Addr>()?;
    let hw_addr = next_field()?.to_owned();
    let client_id = empty_str_to_none(next_field()?);
    let lifetime = parse_duration(next_field()?)?;
    let expires = parse_datetime(next_field()?)?;
    let subnet_id = parse_subnet_id(next_field()?)?;
    let fqdn_fwd = parse_fqdn_fwd(next_field()?)?;
    let fqdn_rev = parse_fqdn_rev(next_field()?)?;
    let hostname = empty_str_to_none(next_field()?);
    let state = parse_state(next_field()?)?;
    let user_context = empty_str_to_none(next_field()?);
    let pool_id = parse_pool_id(next_field()?)?;

    Ok(Lease {
        ip_addr,
        hw_addr,
        client_id,
        lifetime,
        expires,
        subnet_id,
        fqdn_fwd,
        fqdn_rev,
        hostname,
        state,
        user_context,
        pool_id,
    })
}

/// Just reads the given file and looks for lease blocks by scanning over the lines
pub fn parse_file(file: &Path) -> Result<Vec<Lease>, FileParseError> {
    let contents = fs::read_to_string(file)?;
    let mut lines = contents.lines();
    let _header = lines.next().ok_or(FileParseError::Empty)?;

    lines
        .enumerate()
        .map(|(i, l)| (i, parse_line(l)))
        .map(|(i, r)| r.map_err(|e| FileParseError::from((i, e))))
        .collect()
}


#[test]
fn test_parse_lease() {
    let l = r#"192.168.1.21,c7:91:77:a0:03:dd,,86400,1716402166,1,0,0,,0,,0"#;
    eprintln!("Testing with lease:\n{l}");
    let lease = parse_line(l);

    eprintln!("{:?}", lease);
    assert!(lease.is_ok());
}

#[test]
fn test_read_lease_file() {
    let lease_file: std::path::PathBuf = "tests/dhcp4.leases".into();
    eprintln!("Testing with lease file: {lease_file:?}");
    let lease_list = parse_file(&lease_file);

    eprintln!("{:?}", lease_list);
    assert!(lease_list.is_ok());
}
