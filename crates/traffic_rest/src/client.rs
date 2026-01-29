use std::time::{Duration, Instant};

use http::StatusCode;
use reqwest::Client;

#[derive(Clone, Debug)]
pub struct Observation {
    pub status: StatusCode,
    pub latency: Duration,
}

pub async fn send_request(
    client: &Client,
    url: &str,
    key_header: &str,
    key: Option<&str>,
) -> Result<Observation, reqwest::Error> {
    let start = Instant::now();

    let mut req = client.get(url);
    if let Some(k) = key {
        req = req.header(key_header, k);
    }

    let resp = req.send().await?;
    let status =
        StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    Ok(Observation {
        status,
        latency: start.elapsed(),
    })
}
