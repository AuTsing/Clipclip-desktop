use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use std::error::Error;
use tiny_http::{Header, Request, Response};

struct Server;

impl Server {
    pub fn run() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        let server = tiny_http::Server::http("0.0.0.0:8090")?;

        loop {
            let mut req = server.recv()?;
            let req_message = RequestMessage::from_request(&mut req);

            let resp_message = match req_message {
                RequestMessage::Upload { data } => ResponseMessage::from_upload_data(data),
                RequestMessage::Download { data } => ResponseMessage::default(),
                RequestMessage::Unknown => ResponseMessage::default(),
            };
            let resp_message_json = to_string(&resp_message).unwrap_or("".to_string());
            let resp = Response::from_string(resp_message_json).with_header(Header {
                field: "Content-Type".parse().unwrap(),
                value: "application/json".parse().unwrap(),
            });

            let _ = req.respond(resp);
        }
    }
}

enum RequestMessage {
    Upload { data: RequestMessageUploadData },
    Download { data: RequestMessageDownloadData },
    Unknown,
}

#[derive(Deserialize)]
#[serde(tag = "ty")]
enum RequestMessageUploadData {
    Text { content: String },
}

struct RequestMessageDownloadData {}

impl RequestMessage {
    fn from_request(req: &mut Request) -> Self {
        match req.url() {
            "/upload" => match Self::parse_upload(req) {
                Ok(it) => it,
                Err(_) => Self::Unknown,
            },
            "/download" => Self::Download {
                data: RequestMessageDownloadData {},
            },
            _ => Self::Unknown,
        }
    }

    fn parse_upload(req: &mut Request) -> Result<RequestMessage, Box<dyn Error>> {
        let mut data_json = String::new();
        req.as_reader().read_to_string(&mut data_json)?;
        let data = from_str::<RequestMessageUploadData>(&data_json)?;
        let message = RequestMessage::Upload { data };

        Ok(message)
    }
}

#[derive(Serialize)]
struct ResponseMessage {
    success: bool,
    message: String,
    data: ResponseMessageData,
}

#[derive(Serialize)]
enum ResponseMessageData {
    Upload {},
    Download {},
    Empty {},
}

impl Default for ResponseMessage {
    fn default() -> Self {
        Self {
            success: false,
            message: "Unknown api".to_string(),
            data: ResponseMessageData::Empty {},
        }
    }
}

impl ResponseMessage {
    fn from_upload_data(data: RequestMessageUploadData) -> Self {
        Self {
            success: true,
            message: "ok".to_string(),
            data: ResponseMessageData::Empty {},
        }
    }
}
