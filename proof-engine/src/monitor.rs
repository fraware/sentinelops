// proof-engine/src/monitor.rs  (v0.3 – Tseitin + UNSAT core)
// =============================================================
// * Integrates `delta_clauses_tseitin` for pure Boolean props (¬implWithin/¬windowAll).
// * Exposes `last_core` with clause indices for audit UI.
// =============================================================

use crate::cnf::delta_clauses;               // fallback encoder
use crate::cnf_tseitin::{delta_clauses_tseitin, EncoderState};
use crate::dsl::{Prop, Trace};
use crate::sat::{SatCore, SatResult};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use z3::{Config, Context};

static Z3_CTX: Lazy<Context> = Lazy::new(|| {
    let mut c = Config::new();
    c.set_timeout_msec(100);
    Context::new(&c)
});

pub struct PropertyMonitor<'ctx> {
    prop: Prop,
    horizon: usize,
    sat: SatCore<'ctx>,
    enc_state: EncoderState,       // keeps aux-var mapping
    pub last_core: Vec<usize>,     // indices of UNSAT core (for UI)
}

impl<'ctx> PropertyMonitor<'ctx> {
    pub fn new(prop: Prop, horizon: usize) -> Self {
        let sat = SatCore::new(&*Z3_CTX, horizon);
        Self { prop, horizon, sat, enc_state: EncoderState::new(10_000), last_core: vec![] }
    }

    fn is_boolean_only(p: &Prop) -> bool {
        use Prop::*;
        match p {
            And(a,b)|Or(a,b) => Self::is_boolean_only(a)&&Self::is_boolean_only(b),
            Le(_,_)|RateBound(_,_) => true,
            _ => false,
        }
    }

    pub fn tick(&mut self, window: &Trace) -> bool {
        debug_assert!(window.len() <= self.horizon);
        let holds = crate::dsl::eval_prop(&self.prop, window); // helper bridging to Rust monitor
        let delta = if Self::is_boolean_only(&self.prop) {
            delta_clauses_tseitin(&self.prop, holds, &mut self.enc_state)
        } else {
            delta_clauses(&self.prop, window) // earlier empty‑clause strategy
        };
        match self.sat.unsat_recycle(delta).expect("solver") {
            SatResult::Sat => { self.last_core.clear(); true },
            SatResult::Unsat => {
                self.last_core = self.sat.get_unsat_core().unwrap_or_default();
                false
            },
            SatResult::Unknown => { log::warn!("Z3 UNKNOWN"); false },
        }
    }
}
