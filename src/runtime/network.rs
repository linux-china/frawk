use std::collections::HashMap;
use std::env;
use std::sync::Mutex;
use lazy_static::lazy_static;
use reqwest::blocking::Response;
use reqwest::header::{HeaderMap, HeaderName};
use serde::Serialize;
use url::Url;
use crate::runtime::{Str, StrMap};

pub fn local_ip() -> String {
    if let Ok(my_ip) = local_ip_address::local_ip() {
        return my_ip.to_string();
    }
    "127.0.0.1".to_owned()
}

pub(crate) fn http_get<'a>(url: &str, headers: &StrMap<'a, Str<'a>>) -> StrMap<'a, Str<'a>> {
    use reqwest::blocking::Client;
    let client = Client::new();
    let resp_obj: StrMap<Str> = StrMap::default();
    let mut builder = client.get(url);
    if headers.len() > 0 {
        builder = builder.headers(convert_to_http_headers(headers));
    }
    if let Ok(resp) = builder.send() {
        fill_response(resp, &resp_obj);
    } else {
        resp_obj.insert(Str::from("status"), Str::from("0"));
    }
    resp_obj
}


pub(crate) fn http_post<'a>(url: &str, headers: &StrMap<'a, Str<'a>>, body: &Str) -> StrMap<'a, Str<'a>> {
    use reqwest::blocking::Client;
    let client = Client::new();
    let resp_obj: StrMap<Str> = StrMap::default();
    let mut builder = client.post(url);
    if headers.len() > 0 {
        builder = builder.headers(convert_to_http_headers(headers));
    }
    if !body.is_empty() {
        builder = builder.body(body.to_string());
    }
    if let Ok(resp) = builder.send() {
        fill_response(resp, &resp_obj);
    } else {
        resp_obj.insert(Str::from("status"), Str::from("0"));
    }
    resp_obj
}

fn convert_to_http_headers<'a>(headers: &StrMap<'a, Str<'a>>) -> HeaderMap {
    let mut request_headers = HeaderMap::new();
    for name in &headers.to_vec() {
        request_headers.insert(HeaderName::from_bytes(name.to_string().as_bytes()).unwrap(), headers.get(name).to_string().parse().unwrap());
    }
    request_headers
}

fn fill_response(resp: Response, resp_obj: &StrMap<Str>) {
    let status = resp.status();
    resp_obj.insert(Str::from("status"), Str::from(status.as_u16().to_string()));
    let response_headers = resp.headers();
    for (name, value) in response_headers.into_iter() {
        resp_obj.insert(Str::from(name.to_string()), Str::from(value.to_str().unwrap().to_string()));
    }
    if let Ok(body) = resp.text() {
        if !body.is_empty() {
            resp_obj.insert(Str::from("text"), Str::from(body.clone()));
        }
    }
}

// todo graceful shutdown
lazy_static! {
    static ref NATS_CONNECTIONS: Mutex<HashMap<String, nats::Connection>> = Mutex::new(HashMap::new());
}

pub(crate) fn publish(namespace: &str, body: &str) {
    if namespace.starts_with("nats://") || namespace.starts_with("nats+tls://") {
        if let Ok(url) = &Url::parse(namespace) {
            let schema = url.scheme();
            let topic = if url.path().starts_with('/') {
                url.path()[1..].to_string()
            } else {
                url.path().to_string()
            };
            let conn_url = if schema.contains("tls") {
                format!("tls://{}:{}", url.host().unwrap(), url.port().unwrap_or(4443))
            } else {
                format!("{}:{}", url.host().unwrap(), url.port().unwrap_or(4222))
            };
            let mut pool = NATS_CONNECTIONS.lock().unwrap();
            let nc = pool.entry(conn_url.clone()).or_insert_with(|| {
                nats::connect(&conn_url).unwrap()
            });
            nc.publish(&topic, body).unwrap();
        }
    } else {
        notify_rust::Notification::new()
            .summary(namespace)
            .body(body)
            .show().unwrap();
    }
}

#[derive(Debug, Serialize)]
struct MailSendRequest {
    from: MailAddress,
    to: Vec<MailAddress>,
    subject: String,
    text: String,
}

#[derive(Debug, Serialize)]
struct MailAddress {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub email: String,
}

impl MailAddress {
    pub fn new(email: &str) -> Self {
        MailAddress {
            name: None,
            email: email.to_string(),
        }
    }
}

pub fn send_mail(from: &str, to: &str, subject: &str, text: &str) {
    let req = MailSendRequest {
        from: MailAddress::new(from),
        to: vec![MailAddress::new(to)],
        subject: subject.to_string(),
        text: text.to_string(),
    };
    let (api_url, api_key) = if let Ok(api_key) = env::var("MLSN_API_KEY") {
        ("https://api.mailersend.com/v1/email".to_owned(), api_key)
    } else {
        ("".to_owned(), "".to_owned())
    };
    if env::var("DRY_RUN").is_ok() {
        println!("====DRY_RUN MODE====");
        println!("API URL: {}", api_url);
        println!("JSON Body: {}", serde_json::to_string(&req).unwrap());
        return;
    }
    if !api_url.is_empty() {
        let client = reqwest::blocking::Client::new();
        let _resp = client.post(api_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&req)
            .send()
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use local_ip_address::local_ip;
    use super::*;

    #[test]
    fn test_local_ip() {
        let my_local_ip = local_ip().unwrap();
        println!("This is my local IP address: {:?}", my_local_ip);
    }

    #[test]
    fn test_http_get() {
        let url = "https://httpbin.org/ip";
        let headers: StrMap<Str> = StrMap::default();
        let resp = http_get(url, &headers);
        println!("{}", resp.get(&Str::from("text")));
    }

    #[test]
    fn test_http_post() {
        let url = "https://httpbin.org/post";
        let headers: StrMap<Str> = StrMap::default();
        let body = Str::from("Hello");
        let resp = http_post(url, &headers, &body);
        println!("{}", resp.get(&Str::from("text")));
    }

    #[test]
    fn test_publish_nats() {
        let url = "nats://localhost:4222/topic1";
        publish(url, "Hello World!");
    }

    #[test]
    fn test_send_email() {
        dotenv::dotenv().ok();
        let from = "support@trial-3zxk54v3ykzgjy6v.mlsender.net";
        let to = "linux_china@hotmail.com";
        let subject = "demo.csv processed successfully by zawk";
        let text = "rows: 180, total: 1000";
        send_mail(from, to, subject, text);
    }
}
