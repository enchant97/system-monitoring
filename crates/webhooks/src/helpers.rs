use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::sign::Signer;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    redirect::Policy,
    Client,
};

static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

/// Sign a webhooks body with HMAC-sha256
pub fn sign_body(body: &String, secret: String) -> String {
    let key = PKey::hmac(secret.as_bytes()).unwrap();
    let mut signer = Signer::new(MessageDigest::sha256(), &key).unwrap();
    signer.update(body.as_bytes()).unwrap();
    let signed = signer.sign_to_vec().unwrap();
    signed
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<String>>()
        .join("")
}

/// Create a client ready for sending webhook requests
pub fn new_client() -> Client {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    Client::builder()
        .user_agent(USER_AGENT)
        .redirect(Policy::none())
        .default_headers(headers)
        .build()
        .expect("unable to build webhook client")
}
