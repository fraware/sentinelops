//! CLI:  `cargo run -p ledger --bin export_phmsa -- --hour \"2025-07-01T00:00Z\"`
//! Generates a PHMSA-compatible XML snippet for the requested UTC hour.

use chrono::{DateTime, TimeZone, Utc};
use quick_xml::events::{BytesStart, Event};
use quick_xml::writer::Writer;
use std::io::Cursor;
use tokio_postgres::NoTls;

fn main() -> anyhow::Result<()> {
    let mut args = pico_args::Arguments::from_env();
    let hour_iso: String = args.value_from_str("--hour")?;
    let hour: DateTime<Utc> = DateTime::parse_from_rfc3339(&hour_iso)?
        .with_timezone(&Utc);

    let url = std::env::var("PG_URL")
        .unwrap_or_else(|_| "postgres://sentinel:sentinel@localhost/sentinel".into());

    let xml = tokio::runtime::Runtime::new()?.block_on(async {
        let (db, conn) = tokio_postgres::connect(&url, NoTls).await?;
        tokio::spawn(conn);

        let rows = db.query(
            "SELECT root, txhash FROM merkle_batches
             WHERE date_trunc('hour', ts) = $1",
            &[&hour]).await?;

        if rows.is_empty() {
            anyhow::bail!("no batch anchored at that hour");
        }
        let root  : &[u8] = rows[0].get(0);
        let txhash: &str  = rows[0].get(1);

        /* build XML */
        let mut w = Writer::new(Cursor::new(Vec::<u8>::new()));
        let mut start = BytesStart::new("IntegrityEvidence");
        start.push_attribute(("xmlns", "urn:phmsa:mega-rule:2024"));
        w.write_event(Event::Start(start))?;

        let mut seg = BytesStart::new("Segment");
        seg.push_attribute(("id", "hourly-batch"));
        w.write_event(Event::Start(seg))?;

        let mut anchor = BytesStart::new("ProofAnchor");
        anchor.push_attribute(("chain", "polygon"));
        anchor.push_attribute(("tx", txhash));
        anchor.push_attribute(("root", hex::encode(root).as_str()));
        w.write_event(Event::Empty(anchor))?;

        w.write_event(Event::End(BytesStart::new("Segment")))?;
        w.write_event(Event::End(BytesStart::new("IntegrityEvidence")))?;
        Ok::<_, anyhow::Error>(String::from_utf8(w.into_inner().into_inner())?)
    })?;

    println!("{xml}");
    Ok(())
}
