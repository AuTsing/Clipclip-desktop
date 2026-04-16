use anyhow::{Error, Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use std::{
    sync::mpsc::{Sender, channel},
    thread::{self, JoinHandle},
};
use tiny_http::{Header, Request, Response};

pub struct Server {
    listening_handle: Option<JoinHandle<()>>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            listening_handle: None,
        }
    }

    pub fn start_listening(
        &mut self,
        save_clip_tx: Sender<String>,
        get_clip_tx: Sender<Sender<String>>,
    ) {
        self.listening_handle = Some(thread::spawn(move || {
            let server = tiny_http::Server::http("0.0.0.0:8090").unwrap();
            loop {
                let mut req = match server.recv() {
                    Ok(it) => it,
                    Err(_) => {
                        // TODO(Log err)
                        continue;
                    }
                };

                let response_message =
                    Server::to_response_message(&mut req, &save_clip_tx, &get_clip_tx)
                        .unwrap_or_else(|e| ResponseMessage::failed(e));

                if let Err(_) = Server::response(req, &response_message) {
                    // TODO(Log err)
                    continue;
                }
            }
        }));
    }

    fn to_response_message(
        req: &mut Request,
        save_clip_tx: &Sender<String>,
        get_clip_tx: &Sender<Sender<String>>,
    ) -> Result<ResponseMessage> {
        let req_message = RequestMessage::from_request(req)?;
        let resp_message = match req_message {
            RequestMessage::Upload { data } => {
                match data {
                    RequestMessageUploadData::Text { content } => {
                        save_clip_tx.send(content)?;
                    }
                }
                ResponseMessage::upload_success()
            }
            RequestMessage::Download {} => {
                let (get_clip_result_tx, get_clip_result_rx) = channel();
                get_clip_tx.send(get_clip_result_tx)?;
                let clip = get_clip_result_rx.recv()?;
                ResponseMessage::download_success(clip)
            }
        };

        Ok(resp_message)
    }

    fn response(req: Request, response_message: &ResponseMessage) -> Result<()> {
        let resp_message_json = to_string(response_message).unwrap_or("".to_string());
        let resp = Response::from_string(resp_message_json).with_header(Header {
            field: "Content-Type"
                .parse()
                .map_err(|_| anyhow!("Parse header failed"))?,
            value: "application/json"
                .parse()
                .map_err(|_| anyhow!("Parse header failed"))?,
        });
        req.respond(resp)?;

        Ok(())
    }
}

enum RequestMessage {
    Upload { data: RequestMessageUploadData },
    Download {},
}

#[derive(Deserialize)]
#[serde(tag = "ty")]
enum RequestMessageUploadData {
    Text { content: String },
}

impl RequestMessage {
    fn from_request(req: &mut Request) -> Result<Self> {
        match req.url() {
            "/upload" => Self::parse_upload(req),
            "/download" => Self::parse_download(req),
            route => Err(anyhow!("Unknown api: {}", route)),
        }
    }

    fn parse_upload(req: &mut Request) -> Result<RequestMessage> {
        let mut data_json = String::new();
        req.as_reader().read_to_string(&mut data_json)?;
        let data = from_str::<RequestMessageUploadData>(&data_json)?;
        let message = RequestMessage::Upload { data };

        Ok(message)
    }

    fn parse_download(_req: &mut Request) -> Result<RequestMessage> {
        let message = RequestMessage::Download {};

        Ok(message)
    }
}

#[derive(Serialize)]
#[serde(untagged)]
enum ResponseMessage {
    UploadResult {
        success: bool,
        message: String,
    },
    DownloadResult {
        success: bool,
        message: String,
        clip: String,
    },
    Unknown {
        success: bool,
        message: String,
    },
}

impl ResponseMessage {
    fn upload_success() -> Self {
        Self::UploadResult {
            success: true,
            message: "ok".to_string(),
        }
    }

    fn download_success(clip: String) -> Self {
        Self::DownloadResult {
            success: true,
            message: "ok".to_string(),
            clip,
        }
    }

    fn failed(e: Error) -> Self {
        Self::Unknown {
            success: false,
            message: format!("{:?}", e),
        }
    }
}
