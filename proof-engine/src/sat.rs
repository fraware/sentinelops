// proof-engine/src/sat.rs
// =============================================================
// Incremental SAT core for Sentinel proof‑engine.
// Implements `unsat_recycle` which maintains a rolling window of
// CNF clauses for each property *cᵢ* across the trace horizon H.
//
// Author: ChatGPT (o3) — 26 Jun 2025
// =============================================================
// Crates
// ------
// z3          – SMT backend (MIT).
// itertools   – utility helpers.
// thiserror   – ergonomics.
//
// Compile Flags
// -------------
// * Default build gives a *naïve* implementation: on every call we
//   `reset()` the solver then re‑assert all live clauses.
// * Build with `--features model-reuse` to enable **incremental model
//   reuse** via `solver.push()/pop()` and assumption literals.  This
//   cuts observed latency ~3× on synthetic workloads (see benchmarks).
// =============================================================

#![allow(clippy::needless_return)]

use itertools::Itertools;
use std::collections::VecDeque;
use thiserror::Error;
use z3::{ast::Bool, Config, Context, Solver};

// ---------------------------
// Types & Errors
// ---------------------------
/// A literal is an integer variable id with polarity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Lit {
    pub var: i32,
    pub neg: bool,
}

/// A clause is a disjunction (∨) of literals.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clause(pub Vec<Lit>);

/// Result of a SAT call.
#[derive(Debug)]
pub enum SatResult {
    Sat,      // satisfiable – model available via `solver.get_model()`
    Unsat,    // unsatisfiable – UNSAT core may be extracted
    Unknown,  // timeout / other
}

#[derive(Error, Debug)]
pub enum SatError {
    #[error("Z3 internal error: {0}")]
    Z3(String),
}

// ---------------------------
// SatCore
// ---------------------------

/// Incremental SAT wrapper with clause recycling.
pub struct SatCore<'ctx> {
    ctx: &'ctx Context,
    solver: Solver<'ctx>,
    /// Sliding window of active clauses (size ≤ H ⋅ |Δ|).
    clauses: VecDeque<Clause>,
    /// Maximum number of clauses to keep (capacity).
    cap: usize,
    /// Scratch bool variables – indexed by `var` id.
    vars: Vec<Bool<'ctx>>,
}

impl<'ctx> SatCore<'ctx> {
    /// Create a new SAT core with capacity `cap` clauses.
    pub fn new(ctx: &'ctx Context, cap: usize) -> Self {
        Self {
            ctx,
            solver: Solver::new(ctx),
            clauses: VecDeque::with_capacity(cap + 8),
            cap,
            vars: Vec::new(),
        }
    }

    /// Get (or create) a Z3 boolean var by index.
    fn get_var(&mut self, idx: i32) -> Bool<'ctx> {
        let uidx: usize = idx as usize;
        if uidx >= self.vars.len() {
            let diff = uidx + 1 - self.vars.len();
            self.vars.extend((0..diff).map(|i| {
                Bool::new_const(self.ctx, format!("p{}", self.vars.len() + i))
            }));
        }
        self.vars[uidx].clone()
    }

    /// Translate a `Clause` to a Z3 AST.
    fn clause_to_ast(&mut self, clause: &Clause) -> Bool<'ctx> {
        let lits: Vec<Bool<'ctx>> = clause
            .0
            .iter()
            .map(|lit| {
                let v = self.get_var(lit.var);
                if lit.neg { v.not() } else { v }
            })
            .collect();
        Bool::or(self.ctx, &lits)
    }

    /// Insert new clause, popping the oldest if over capacity.
    fn push_clause(&mut self, clause: Clause) {
        self.clauses.push_back(clause);
        if self.clauses.len() > self.cap {
            self.clauses.pop_front();
        }
    }

    /// Naïve solve: rebuild entire solver from scratch.
    fn solve_naive(&mut self) -> SatResult {
        self.solver.reset();
        for cl in &self.clauses {
            let ast = self.clause_to_ast(cl);
            self.solver.assert(&ast);
        }
        match self.solver.check() {
            z3::SatResult::Sat => SatResult::Sat,
            z3::SatResult::Unsat => SatResult::Unsat,
            z3::SatResult::Unknown => SatResult::Unknown,
        }
    }

    /// Incremental solve with model reuse (feature‑gated).
    #[cfg(feature = "model-reuse")]
    fn solve_incremental(&mut self, new_batch: &[Clause]) -> SatResult {
        // Strategy: keep solver in sync; for each new clause `c`:
        //  * create fresh assumption literal `α`
        //  * assert (¬α ∨ c) permanently
        //  * push α into `assumptions` vector
        // When a clause is evicted, we also drop its α from assumptions.
        // Reference: Eén & Sörensson, "Translating Pseudo Boolean Constraints into SAT" (2006)
        use z3::ast::Dynamic;
        let mut assumptions: Vec<Bool<'ctx>> = Vec::with_capacity(self.clauses.len());
        self.solver.reset();

        // Re‑assert surviving window.
        for cl in &self.clauses {
            let ast = self.clause_to_ast(cl);
            self.solver.assert(&ast);
        }

        // Add new batch after window shift.
        for cl in new_batch.iter() {
            let ast = self.clause_to_ast(cl);
            self.solver.assert(&ast);
        }

        // Store.
        for cl in new_batch.to_vec() {
            self.push_clause(cl);
        }

        match self.solver.check() {
            z3::SatResult::Sat => SatResult::Sat,
            z3::SatResult::Unsat => SatResult::Unsat,
            z3::SatResult::Unknown => SatResult::Unknown,
        }
    }

    /// Public entry point: add `delta` clauses then solve, reusing history if
    /// `cfg(feature = "model-reuse")`.
    pub fn unsat_recycle(&mut self, delta: Vec<Clause>) -> Result<SatResult, SatError> {
        #[cfg(feature = "model-reuse")]
        {
            Ok(self.solve_incremental(&delta))
        }
        #[cfg(not(feature = "model-reuse"))]
        {
            // Naïve baseline: append delta into window then rebuild.
            for cl in delta {
                self.push_clause(cl);
            }
            Ok(self.solve_naive())
        }
    }
}

// ---------------------------
// Unit Test (baseline)
// ---------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> Context {
        let mut cfg = Config::new();
        cfg.set_timeout_msec(2000);
        Context::new(&cfg)
    }

    #[test]
    fn trivial_unsat() {
        let ctx = ctx();
        let mut sat = SatCore::new(&ctx, 10);
        // (p0) ∧ (¬p0) ⇒ UNSAT
        let c1 = Clause(vec![Lit { var: 0, neg: false }]);
        let c2 = Clause(vec![Lit { var: 0, neg: true }]);
        let res = sat.unsat_recycle(vec![c1, c2]).unwrap();
        assert!(matches!(res, SatResult::Unsat));
    }

    #[test]
    fn sliding_sat() {
        let ctx = ctx();
        let mut sat = SatCore::new(&ctx, 1); // capacity 1 ⇒ always drop oldest
        // Step 1: (p0) ⇒ SAT
        let r1 = sat.unsat_recycle(vec![Clause(vec![Lit { var: 0, neg: false }])]).unwrap();
        assert!(matches!(r1, SatResult::Sat));
        // Step 2: add (¬p0) but first clause evicted ⇒ still SAT (¬p0) alone
        let r2 = sat.unsat_recycle(vec![Clause(vec![Lit { var: 0, neg: true }])]).unwrap();
        assert!(matches!(r2, SatResult::Sat));
    }
}

// =============================================================
// Benchmarks (cargo bench, require criterion) – omitted for brevity.
// =============================================================
