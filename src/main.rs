use std::io::{self, Read};

use data::ServerStatus;
use select::document::Document;

mod parser;
mod data;

fn main() {
    let document: Document = {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).unwrap();
        Document::from(buffer.as_str())
    };

    let workers = match parser::parse_worker_scores(&document) {
        Ok(workers) => workers,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };
    let server_status = ServerStatus{
        workers,
    };

    {
        let json = serde_json::to_string_pretty(&server_status).unwrap();
        println!("{}", json);
    }
}
