use select::{predicate::{Descendant, Name, And, Attr}, document::Document};
use thiserror::Error;

use crate::data::{WorkerScore, WorkerStatus, AccessCounts};

#[derive(Debug, Error)]
pub enum WorkerScoreParseError {
    #[error("invalid headers")]
    InvalidHeaders(),
    #[error("invalid cell count: {0}")]
    InvalidCellCount(String),

    // parse_worker_status()
    #[error("status code must be exactly one character long: `{0}`")]
    StatusCodeMustBeChar(String),
    #[error("invalid status code `{0}`")]
    InvalidStatusCode(char),

    // parse_acc()
    #[error("invalid field count when parsing `{0}`, expected `1/2/3`")]
    AccessCountsInvalidFieldCount(String),

    // parse_srv()
    #[error("the \"Srv\" column is not in format `x-x`: `{0}`")]
    SrvFieldUnknownFormat(String),

    // generic parse errors
    #[error(transparent)]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
}

/// Find the table with worker scores and convert it to a vector of `WorkerScore`s.
pub fn parse_worker_scores(document: &Document) -> Result<Vec<WorkerScore>, WorkerScoreParseError> {
    const TR_PREDICATE:
        Descendant<And<Name<&str>, Attr<&str, &str>>, Name<&str>> =
        Descendant(And(Name("table"), Attr("border", "0")), Name("tr"));

    let mut scores: Vec<WorkerScore> = Vec::with_capacity(2^8);
    
    for (i, row) in document.find(TR_PREDICATE).enumerate() {
        match i {
            0 => {
                // Don't continue if the headers are not valid.
                let _ = validate_headers(&row)?;
            },
            _ => {
                match parse_row(&row) {
                    Ok(score) => scores.push(score),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        println!("Row: {}", row.html());
                        std::process::exit(1);
                    }
                }
                // scores.push(parse_row(&row)?);
            }
        }
    }

    Ok(scores)
}

/// Validate that the headers are right
fn validate_headers(row: &select::node::Node) -> Result<(), WorkerScoreParseError> {
    const VALID_HEADERS: &[&str] = &[
        "Srv",
        "PID",
        "Acc",
        "M",
        "CPU", // TODO this one is optional
        "SS",
        "Req",
        "Dur",
        "Conn",
        "Child",
        "Slot",
        "Client",
        "Protocol",
        "VHost",
        "Request",
    ];

    let result = row.children()
        .zip(VALID_HEADERS.iter())
        .all(|(td, &expected_text)| td.text().trim() == expected_text);
    match result {
        true => Ok(()),
        false => Err(WorkerScoreParseError::InvalidHeaders()),
    }
}

/// Parse a row of the scoreboard table
fn parse_row(row: &select::node::Node) -> Result<WorkerScore, WorkerScoreParseError> {
    let mut cols: Vec<_> = row.children().map(|td| td.text().trim().to_string()).collect();

    match row.children().count() {
        15 => (),
        // Insert a dummy field in place of "CPU" if HAS_TIMES isn't set.
        14 => cols.insert(4, String::from("0")),
        _ => return Err(WorkerScoreParseError::InvalidCellCount(row.html())),
    }

    let cols = cols;
    
    Ok(WorkerScore{
        generation: parse_srv(&cols[0])?.1,
        pid: parse_pid(&cols[1])?,
        access_counts: parse_acc(&cols[2])?,
        status: parse_worker_status(&cols[3])?, // "M" column (= "mode of operation")
        cpu: cols[4].parse()?,
        seconds_since_s: cols[5].parse()?, // "SS" column
        request_time_ms: cols[6].parse()?, // "Req" column
        duration_ms: cols[7].parse()?,     // "Dur" column
        conn_kib: cols[8].parse()?,
        child_mib: cols[9].parse()?,
        slot_mib: cols[10].parse()?,
        client: String::from(&cols[11]),
        protocol: String::from(&cols[12]),
        vhost: String::from(&cols[13]),
        request: String::from(&cols[14]),
    })
}

/// Parse mod_status "Srv" column ("<Child server number>-<generation>")
/// 
/// Returns a tuple of (child_server_number, generation)
fn parse_srv(s: &str) -> Result<(i32, i32), WorkerScoreParseError> {
    let splitted: Vec<&str> = s.split('-').collect();

    if 2 != splitted.len() {
        return Err(WorkerScoreParseError::SrvFieldUnknownFormat(s.to_string()));
    }

    Ok((splitted[0].parse::<i32>()?, splitted[1].parse::<i32>()?))
}

/// Parse mod_status "PID" column
/// "-" means dead process
fn parse_pid(s: &str) -> Result<Option<i32>, WorkerScoreParseError> {
    Ok(match s {
        "-" => None,
        text @ _ => Some(text.parse::<i32>()?),
    })
}

/// Parse mod_status "M" column
/// 
/// See:
/// - https://github.com/apache/httpd/blob/2.4.56/modules/generators/mod_status.c#L865
fn parse_worker_status(s: &str) -> Result<WorkerStatus, WorkerScoreParseError> {
    if 1 != s.len() {
        return Err(WorkerScoreParseError::StatusCodeMustBeChar(s.to_string()));
    }

    let code = s.chars().nth(0).unwrap();

    match code {
        '_' => Ok(WorkerStatus::Ready),
        'S' => Ok(WorkerStatus::Starting),
        'R' => Ok(WorkerStatus::BusyRead),
        'W' => Ok(WorkerStatus::BusyWrite),
        'K' => Ok(WorkerStatus::BusyKeepAlive),
        'L' => Ok(WorkerStatus::BusyLog),
        'D' => Ok(WorkerStatus::BusyDns),
        'C' => Ok(WorkerStatus::Closing),
        '.' => Ok(WorkerStatus::Dead),
        'G' => Ok(WorkerStatus::Graceful),
        'I' => Ok(WorkerStatus::IdleKill),
        _ => Err(WorkerScoreParseError::InvalidStatusCode(code)),
    }
}

/// Parse mod_status "Acc" column
fn parse_acc (s: &str) -> Result<AccessCounts, WorkerScoreParseError> {
    let parts: Vec<&str> = s.split('/').collect();

    if parts.len() != 3 {
        return Err(WorkerScoreParseError::AccessCountsInvalidFieldCount(s.to_string()));
    }

    let connection = parts[0].parse::<u32>()?;
    let child = parts[1].parse::<u32>()?;
    let slot = parts[2].parse::<u32>()?;

    Ok(AccessCounts { connection, child, slot })
}
