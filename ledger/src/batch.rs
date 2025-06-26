//! Hourly Merkle-root builder and Polygon anchor.
//! Run inside `batcher.rs` (see `ledger/bin/batcher.rs`).

use blake3::Hasher;
use chrono::{DateTime, Timelike, Utc};
use ethers::prelude::*;
use serde::Deserialize;
use std::collections::VecDeque;
use tokio_postgres::NoTls;

#[derive(Debug, Deserialize)]
struct ProofPacket {
    property_id: String,
    start_ts: i64,
    end_ts:   i64,
    trace_hash: String,
    cert_hash:  String,
    verdict:    String,          // "PASS" | "FAIL"
}

/* ---------- hashing helpers ------------------------------------------------ */

fn packet_hash(bytes: &[u8]) -> [u8; 32] { blake3::hash(bytes).into() }

fn build_merkle(leaves: &[[u8; 32]]) -> ([u8; 32], Vec<[u8; 32]>) {
    if leaves.is_empty() { return ([0; 32], vec![]); }
    let mut level = leaves.to_vec();
    let mut dag   = level.clone();
    while level.len() > 1 {
        let mut next = vec![];
        for chunk in level.chunks(2) {
            let cat = if chunk.len() == 2 {
                [chunk[0].as_slice(), chunk[1].as_slice()].concat()
            } else {
                chunk[0].to_vec()
            };
            let h = blake3::hash(&cat).into();
            next.push(h);
        }
        dag.extend(&next);
        level = next;
    }
    (level[0], dag)
}

/* ---------- Polygon anchor ------------------------------------------------- */

async fn anchor_polygon(root: [u8; 32]) -> anyhow::Result<TxHash> {
    let rpc  = std::env::var("POLYGON_RPC")
        .unwrap_or_else(|_| "https://polygon-rpc.com".into());
    let key  = std::env::var("POLYGON_PRIVATE_KEY")?;
    let addr : Address = std::env::var("ANCHOR_CONTRACT")?.parse()?;
    let chain_id = 137u64;

    let wallet   = LocalWallet::from_str(&key)?.with_chain_id(chain_id);
    let provider = Provider::<Http>::try_from(rpc.clone())?;
    let client   = SignerMiddleware::new(provider, wallet);

    // ABI call: anchor(bytes32)
    let f = Function {
        name: "anchor".to_string(),
        inputs: vec![Param {
            name: "root".into(),
            kind: ParamType::FixedBytes(32),
            internal_type: None,
        }],
        outputs: vec![],
        constant: None,
        state_mutability: ethers::abi::StateMutability::NonPayable,
    };
    let data = f.encode_input(&[Token::FixedBytes(root.to_vec())])?;
    let gas  = std::env::var("ANCHOR_GAS_LIMIT")
        .unwrap_or_else(|_| "80000".into()).parse()?;

    let pending = client
        .send_transaction(
            TransactionRequest::new()
                .to(addr)
                .data(data)
                .gas(gas),
            None,
        )
        .await?;
    let receipt = pending.await?.ok_or_else(|| anyhow::anyhow!("tx dropped"))?;
    Ok(receipt.transaction_hash)
}

/* ---------- BatchAnchor struct -------------------------------------------- */

pub struct BatchAnchor {
    buf: VecDeque<Vec<u8>>,          // raw packet bytes
    db : tokio_postgres::Client,
}

impl BatchAnchor {
    pub async fn new() -> anyhow::Result<Self> {
        let url = std::env::var("PG_URL")
            .unwrap_or_else(|_| "postgres://sentinel:sentinel@localhost/sentinel".into());
        let (db, conn) = tokio_postgres::connect(&url, NoTls).await?;
        tokio::spawn(conn);

        db.batch_execute(
            "CREATE TABLE IF NOT EXISTS merkle_batches(
               id     SERIAL       PRIMARY KEY,
               ts     TIMESTAMPTZ  NOT NULL,
               root   BYTEA        NOT NULL,
               txhash VARCHAR(66)  NOT NULL,
               dag    BYTEA        NOT NULL)"
        ).await?;

        Ok(Self { buf: VecDeque::new(), db })
    }

    /* ingest one proof-packet (JSON bytes) */
    pub async fn ingest(&mut self, bytes: Vec<u8>) -> anyhow::Result<()> {
        self.buf.push_back(bytes);
        let now = Utc::now();
        if now.minute() == 0 && now.second() < 5 {   // top-of-hour
            self.flush(now).await?;
        }
        Ok(())
    }

    async fn flush(&mut self, ts: DateTime<Utc>) -> anyhow::Result<()> {
        if self.buf.is_empty() { return Ok(()); }

        let batch : Vec<Vec<u8>>   = self.buf.drain(..).collect();
        let leaves: Vec<[u8; 32]>  = batch.iter().map(|b| packet_hash(b)).collect();
        let (root, dag)            = build_merkle(&leaves);

        let tx = anchor_polygon(root).await?;
        self.db.execute(
            "INSERT INTO merkle_batches(ts, root, txhash, dag) VALUES($1,$2,$3,$4)",
            &[&ts, &root.as_slice(), &tx.to_string(), &dag.concat()],
        ).await?;

        log::info!("Batch anchored  root={}  tx={:?}", hex::encode(root), tx);
        Ok(())
    }
}
