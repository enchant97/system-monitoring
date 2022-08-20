use actix_web::{
    dev::Payload, error::ErrorUnauthorized, get, http::header::HeaderValue, web, web::Json, App,
    Error, FromRequest, HttpRequest, HttpServer,
};
use core::future::Future;
use monitoring_core::metrics::{CpuMetrics, MemoryMetrics, Metrics};
use psutil::cpu::CpuPercentCollector;
use std::pin::Pin;
use std::sync::Mutex;

mod config;
mod state;

use config::{read_config_or_defaults, Config, CONFIG_FN};
use state::CollectorState;

struct Client {
    authenticated: bool,
}

fn ip_allowed(client_ip: String, allowed_ips: &Vec<String>) -> bool {
    // TODO implement
    true
}

fn auth_value_allowed(value: Option<&HeaderValue>, allowed_keys: &Vec<String>) -> bool {
    // TODO implement
    // Valid value will look like: 'Bearer key'
    true
}

impl FromRequest for Client {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Client, Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let config = req.app_data::<Config>().unwrap();
        let auth_config = &config.authentication;
        let authorization_value = req.headers().get("Authorization");
        // get the clients ip address
        let client_ip = match config.using_proxy {
            false => req.connection_info().peer_addr().unwrap().to_string(),
            true => req
                .connection_info()
                .realip_remote_addr()
                .unwrap()
                .to_string(),
        };
        // ensures client is allowed
        let authenticated = match (auth_config.check_ip, auth_config.check_key) {
            (false, false) => Some(false), // auth is not needed as it's disabled
            (false, true) => {
                // only key authentication is required
                match auth_value_allowed(authorization_value, &auth_config.allowed_keys) {
                    true => Some(true),
                    false => None,
                }
            }
            (true, false) => {
                // only client ip check is required
                match ip_allowed(client_ip, &auth_config.allowed_ip) {
                    true => Some(true),
                    false => None,
                }
            }
            (true, true) => {
                // both key authentication and client ip is required
                match (
                    auth_value_allowed(authorization_value, &auth_config.allowed_keys),
                    ip_allowed(client_ip, &auth_config.allowed_ip),
                ) {
                    (true, true) => Some(true),
                    _ => None,
                }
            }
        };
        Box::pin(async move {
            match authenticated {
                Some(v) => Ok(Client { authenticated: v }),
                None => Err(ErrorUnauthorized("")),
            }
        })
    }
}

#[get("/")]
async fn get_all(
    _client: Client,
    collector: web::Data<CollectorState>,
) -> actix_web::Result<Json<Metrics>> {
    let cpu_metrics = collector.get_cpu_metrics();
    let memory_metrics = collector.get_memory_metrics();
    let metrics = Metrics {
        cpu: cpu_metrics,
        memory: memory_metrics,
    };
    Ok(Json(metrics))
}

#[get("/cpu")]
async fn get_cpu(
    _client: Client,
    collector: web::Data<CollectorState>,
) -> actix_web::Result<Json<CpuMetrics>> {
    let cpu_metrics = collector.get_cpu_metrics();
    Ok(Json(cpu_metrics))
}

#[get("/memory")]
async fn get_memory(
    _client: Client,
    collector: web::Data<CollectorState>,
) -> actix_web::Result<Json<MemoryMetrics>> {
    let memory_metrics = collector.get_memory_metrics();
    Ok(Json(memory_metrics))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let collector = web::Data::new(CollectorState {
        cpu_collector: Mutex::new(CpuPercentCollector::new().unwrap()),
    });

    let config_path = std::path::PathBuf::from(CONFIG_FN);
    let config = match config_path.is_file() {
        true => read_config_or_defaults(&config_path),
        false => Default::default(),
    };
    let bind = (config.host.clone(), config.port);
    HttpServer::new(move || {
        App::new()
            .app_data(collector.clone())
            .app_data(config.clone())
            .service(get_all)
            .service(get_cpu)
            .service(get_memory)
    })
    .bind(bind)?
    .run()
    .await
}
