use anyhow::{anyhow, Error, Result};
use eventsource::{event::Event, reqwest::Client as EventSourceClient};
use reqwest::{blocking::Client, Url};
use serde_json::json;
use std::{
    io::{BufRead, BufReader},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
};

pub struct DataStreamer {
    data_stream_thread: JoinHandle<()>,
    rx: Receiver<Result<Event, eventsource::reqwest::Error>>,
    stop: Arc<AtomicBool>,
}

impl DataStreamer {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = stop.clone();
        let data_stream_thread =
            thread::spawn(move || create_data_stream_thread(tx, stop_thread).unwrap());

        Self {
            data_stream_thread,
            rx,
            stop,
        }
    }

    pub fn next(&self) -> Result<Option<serde_json::Value>> {
        let res = match self.rx.try_recv() {
            Ok(event) => {
                let event = event.map_err(|e| anyhow!(e.to_string()))?;
                let json = serde_json::from_str::<serde_json::Value>(&event.data)?;
                Ok(Some(json))
            }
            Err(e) => match e {
                mpsc::TryRecvError::Empty => Ok(None),
                mpsc::TryRecvError::Disconnected => Err(anyhow!("Channel Disconnected")),
            },
        };
        
        res
    }
}

impl Drop for DataStreamer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

fn create_data_stream_thread(
    tx: Sender<Result<Event, eventsource::reqwest::Error>>,
    stop: Arc<AtomicBool>,
) -> Result<()> {
    let api_url = env!("SALTAPI_URL");
    let api_user = env!("SALTAPI_USER");
    let api_password = env!("SALTAPI_PASS");
    let api_eauth = env!("SALTAPI_EAUTH");

    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    let login_body = json!({
        "username": api_user,
        "password": api_password,
        "eauth": api_eauth,
    });

    let response: serde_json::Value = client
        .post(Url::parse(&format!("{api_url}/login"))?)
        .json(&login_body)
        .send()?
        .json()?;

    let token = response["return"][0]["token"]
        .as_str()
        .ok_or(anyhow!("No auth token!"))?;
    dbg!(&response);
    dbg!(&token);
    let event_url = Url::parse(&format!("{api_url}/events?token={token}"))?;
    dbg!(&event_url);
    let event_source = EventSourceClient::new_with_client(event_url, client);

    for event in event_source {
        if stop.load(Ordering::Relaxed) {
            return Ok(());
        }
        tx.send(event).map_err(|e| {
            let err_string = e.to_string();
            anyhow!(err_string)
        })?
    }

    Ok(())
}
