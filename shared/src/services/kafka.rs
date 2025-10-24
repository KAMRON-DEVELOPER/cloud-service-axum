use rdkafka::config::ClientConfig;
use rdkafka::consumer::StreamConsumer;
use rdkafka::producer::FutureProducer;

use crate::utilities::config::Config;
use crate::utilities::errors::AppError;

pub fn build_kafka_producer(config: &Config) -> Result<FutureProducer, AppError> {
    let producer = ClientConfig::new()
        .set("bootstrap.servers", config.kafka_brokers.clone().unwrap())
        .set("message.timeout.ms", "5000")
        .set("queue.buffering.max.ms", "1")
        .create::<FutureProducer>()?;

    Ok(producer)
}

pub fn build_kafka_consumer(config: &Config, group_id: &str) -> Result<StreamConsumer, AppError> {
    let consumer = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", config.kafka_brokers.clone().unwrap())
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "earliest")
        .create::<StreamConsumer>()?;

    Ok(consumer)
}
