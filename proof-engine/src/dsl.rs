//! Minimal Rust mirror of the Lean DSL, plus a tiny executable `eval_prop`.
//! Only what the proof-engine needs right now.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Var { P, T, Flow, Valve }

#[derive(Clone, Debug, PartialEq)]
pub enum Prop {
    Le(Var, f64),
    RateBound(Var, f64),
    And(Box<Prop>, Box<Prop>),
    Or(Box<Prop>, Box<Prop>),
    // temporal nodes omitted for brevity
}

/// Evaluate the Boolean DSL on a trace window (newest-first).
pub type Sample = std::collections::HashMap<Var, f64>;
pub type Trace  = Vec<Sample>;

pub fn eval_prop(p: &Prop, trace: &Trace) -> bool {
    use Prop::*;
    match p {
        Le(v, k) => trace.first()
            .map(|s| s.get(v).copied().unwrap_or(0.0) <= *k)
            .unwrap_or(true),
        RateBound(v, k) => trace.get(1)
            .and_then(|prev| trace.first().map(|cur| {
                (cur.get(v).unwrap_or(&0.0) - prev.get(v).unwrap_or(&0.0)).abs() <= *k
            }))
            .unwrap_or(true),
        And(a, b) => eval_prop(a, trace) && eval_prop(b, trace),
        Or(a, b)  => eval_prop(a, trace) || eval_prop(b, trace),
    }
}
