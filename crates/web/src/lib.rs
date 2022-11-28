use actix_web::{middleware::Logger, web, App, HttpServer};
use agent_collector::CollectorState;
use agent_config::types::Config;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::sync::Arc;

mod extractor;
mod routes;

pub async fn run(config: &Config, collector: Arc<CollectorState>) -> std::io::Result<()> {
    let config = config.clone();
    // Create the HTTP server
    let bind = (config.web.host.clone(), config.web.port);
    let ssl_builder = match config.web.certificate {
        Some(ref v) => {
            // TODO remove unwrap usage
            let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
            builder
                .set_private_key_file(v.private_path.to_str().unwrap(), SslFiletype::PEM)
                .unwrap();
            builder
                .set_certificate_chain_file(v.public_path.to_str().unwrap())
                .unwrap();

            Some(builder)
        }
        None => None,
    };

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(collector.clone()))
            .app_data(web::Data::new(config.clone()))
            .service(routes::get_is_healthy)
            .service(routes::get_agent_id)
            .service(
                web::scope("/metrics")
                    .service(routes::get_all)
                    .service(
                        web::scope("/cpu").service(routes::get_cpu).service(
                            web::scope("/load")
                                .service(routes::get_cpu_load)
                                .service(routes::get_cpu_load_average)
                                .service(routes::get_cpu_load_per_core),
                        ),
                    )
                    .service(
                        web::scope("/memory")
                            .service(routes::get_memory)
                            .service(routes::get_memory_perc_used)
                            .service(routes::get_memory_detailed),
                    ),
            )
    });

    // start the server
    match ssl_builder {
        Some(builder) => {
            log::info!("serving over HTTPS on: {bind:?}");
            server.bind_openssl(bind, builder)?.run().await
        }
        None => {
            log::info!("serving over HTTP on: {bind:?}");
            server.bind(bind)?.run().await
        }
    }
}
