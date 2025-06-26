// benches/engine_bench.rs  (multi-pack benchmarks)
use criterion::{criterion_group, criterion_main, Criterion};
use sentinel_proof_engine::{dsl::*, monitor::Engine};
use std::collections::HashMap;

type Trace = Vec<HashMap<Var, f64>>;

fn bench_pack(c: &mut Criterion, n: usize) {
    let bench_name = format!("engine_latency_{}", n);
    let props: Vec<Prop> = (0..n).map(|_| Prop::Le(Var::P, 120.0)).collect();
    let mut eng = Engine::new(props, 6);
    let mut sample = HashMap::new();
    sample.insert(Var::P, 100.0);
    let window: Trace = vec![sample; 6];
    c.bench_function(&bench_name, |b| b.iter(|| eng.step(&window)));
}

fn benches(c: &mut Criterion) {
    for &n in &[10_usize, 25, 50] {
        bench_pack(c, n);
    }
}

criterion_group!(engine_latency, benches);
criterion_main!(engine_latency);
