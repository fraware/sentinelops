//! Modbus-to-Kafka Edge Agent.

use chrono::Utc;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;
use tokio_modbus::prelude::*;

#[derive(Serialize)]
struct TracePacket<'a> {
    ts:   i64,
    tags: &'a HashMap<&'static str, f64>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let plc_addr = std::env::var("PLC_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let kafka    = std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".into());
    let topic    = std::env::var("KAFKA_TRACE_TOPIC").unwrap_or_else(|_| "plc.trace".into());

    let producer: FutureProducer =
        ClientConfig::new().set("bootstrap.servers", &kafka).create()?;

    let sock = format!("{plc_addr}:502").parse()?;
    let mut ctx = tcp::connect(sock).await?;

    // (register idx, tag name, scale factor)
    let cfg = vec![
        (0, "P", 1.0),
        (1, "T", 1.0),
        (2, "Flow", 0.1),
        (3, "Valve", 1.0),
    ];

    loop {
        // 1. Read registers
        let regs = ctx.read_holding_registers(0, cfg.len() as u16).await?;
        let mut map = HashMap::with_capacity(cfg.len());
        for ((_, tag, scale), raw) in cfg.iter().zip(regs) {
            map.insert(*tag, (raw as f64) * scale);
        }

        // 2. Serialize + send
        let pkt   = TracePacket { ts: Utc::now().timestamp(), tags: &map };
        let bytes = serde_json::to_vec(&pkt)?;

        producer
            .send(
                FutureRecord::<(), _>::to(&topic).payload(&bytes),
                Duration::from_secs(0),
            )
            .await
            .map_err(|(e, _msg)| e)?;   // convert tuple → KafkaError

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
