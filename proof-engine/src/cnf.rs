// proof-engine/src/cnf.rs
// =============================================================
// *Naïve* CNF encoder for Sentinel proof‑engine.
// -------------------------------------------------------------
// Goal (MVP): produce an incremental **Δ‑clause list** per tick such that
//   • If the monitored property *p* **holds** on the sliding window τᵗ, no
//     empty clause is emitted ⇒ SATCore remains **SAT**.
//   • If *p* **violates**, we emit a single empty clause `⊥` which forces
//     UNSAT *until* it ages out of the clause window (≥ horizon H).
//
// This design is deliberately simple; it lets us wire `SatCore` into the
// main loop *today* while we work on a full Tseitin encoder.  The empty
// clause trick is guaranteed sound (no false positives) but may miss some
// optimisation opportunities (it cannot provide UNSAT cores).  That is
// acceptable for latency benchmarking and demo pilots.
//
// Future work (tracked in #42):
//   • Replace empty‑clause shortcut with true CNF (Tseitin) so we can pull
//     minimal counterexamples from Z3.
//   • Prove encoder soundness in Lean; reuse the proof for both Rust & Lean
//     via extraction.
// =============================================================

use crate::sat::{Clause, Lit};
use crate::dsl::{Prop, Trace};       // `dsl` module re‑exports `Prop` and helpers.
use crate::monitor::prop_monitor;     // Rust mirror of Lean `eval` (already vetted).

/// Generate the **delta** clause set for the new tick.
///
/// * If `prop_monitor(p, τ)` is `true` → returns `vec![]` (no change ⇒ SAT).
/// * Else (*violation*) → returns `[Clause(vec![])]` i.e. the empty clause ⊥
///   which makes the solver UNSAT until clause is aged out of the window.
pub fn delta_clauses(p: &Prop, window: &Trace) -> Vec<Clause> {
    if prop_monitor(p, window) {
        Vec::new()
    } else {
        vec![Clause(Vec::new())] // empty clause ⇒ immediate UNSAT
    }
}

// ---------------------------
// Unit tests
// ---------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::{Var};
    use std::collections::HashMap;

    fn sample_pressure(v: f64) -> HashMap<Var, f64> {
        let mut s = HashMap::new();
        s.insert(Var::P, v);
        s
    }

    #[test]
    fn holds_no_clause() {
        let prop = Prop::Le(Var::P, 10.0);
        let trace = vec![sample_pressure(5.0)];
        let delta = delta_clauses(&prop, &trace);
        assert!(delta.is_empty());
    }

    #[test]
    fn violation_empty_clause() {
        let prop = Prop::Le(Var::P, 1.0);
        let trace = vec![sample_pressure(5.0)];
        let delta = delta_clauses(&prop, &trace);
        assert_eq!(delta.len(), 1);
        assert!(delta[0].0.is_empty()); // ⊥
    }
}
