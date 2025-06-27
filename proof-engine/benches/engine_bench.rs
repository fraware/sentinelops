// benches/engine_bench.rs  (multi-pack benchmarks)
use criterion::{criterion_group, criterion_main, Criterion};
use proof_engine::{dsl, monitor::PropertyMonitor};
use std::collections::HashMap;

type Trace = Vec<HashMap<dsl::Var, f64>>;

fn bench_pack(c: &mut Criterion, n: usize) {
    let bench_name = format!("engine_latency_{}", n);
    let props: Vec<dsl::Prop> = (0..n).map(|_| dsl::Prop::Le(dsl::Var::P, 120.0)).collect();
    let mut eng = PropertyMonitor::new(props[0].clone(), 6);
    let mut sample = HashMap::new();
    sample.insert(dsl::Var::P, 100.0);
    let window: Trace = vec![sample; 6];
    c.bench_function(&bench_name, |b| b.iter(|| eng.tick(&window)));
}

fn benches(c: &mut Criterion) {
    for &n in &[10_usize, 25, 50] {
        bench_pack(c, n);
    }
}

criterion_group!(engine_latency, benches);
criterion_main!(engine_latency);
