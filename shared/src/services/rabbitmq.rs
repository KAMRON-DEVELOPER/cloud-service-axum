use lapin::{
    BasicProperties, Confirmation, Connection, ConnectionProperties,
    message::DeliveryResult,
    options::*,
    tcp::{OwnedIdentity, OwnedTLSConfig},
    types::FieldTable,
};
use tracing::info;

pub async fn get_rabbitmq() {
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());

    let properties = ConnectionProperties::async_global_executor::block_on(async {
        let conn = Connection::connect(&addr, ConnectionProperties::default())
            .await
            .expect("connection error");
    });
}

fn get_tls_config() -> OwnedTLSConfig {
    let cert_chain = "" /* include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/path/to/ca_certificate.pem"
    )) */;
    let client_cert_and_key = b""
        /* include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/path/to/client.pfx")) */;
    let client_cert_and_key_password = "bunnies";

    OwnedTLSConfig {
        identity: Some(OwnedIdentity::PKCS12 {
            der: client_cert_and_key.to_vec(),
            password: client_cert_and_key_password.to_string(),
        }),
        cert_chain: Some(cert_chain.to_string()),
    }
}
