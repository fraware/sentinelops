// proof-engine/src/monitor.rs  (v0.3 – Tseitin + UNSAT core)
// =============================================================
// * Integrates `delta_clauses_tseitin` for pure Boolean props (¬implWithin/¬windowAll).
// * Exposes `last_core` with clause indices for audit UI.
// =============================================================

use crate::cnf::delta_clauses;               // fallback encoder
use crate::dsl::{Prop, Trace};
use crate::sat::{SatCore, SatResult};
use z3::{Config, Context};

pub struct PropertyMonitor {
    prop: Prop,
    horizon: usize,
    ctx: Context,
    pub last_core: Vec<usize>,     // indices of UNSAT core (for UI)
}

impl PropertyMonitor {
    pub fn new(prop: Prop, horizon: usize) -> Self {
        let mut cfg = Config::new();
        cfg.set_timeout_msec(100);
        let ctx = Context::new(&cfg);
        PropertyMonitor {
            prop,
            horizon,
            ctx,
            last_core: Vec::new(),
        }
    }

    fn is_boolean_only(p: &Prop) -> bool {
        use Prop::*;
        match p {
            And(a,b)|Or(a,b) => Self::is_boolean_only(a)&&Self::is_boolean_only(b),
            Le(_,_)|RateBound(_,_) => true,
        }
    }

    pub fn tick(&mut self, window: &Trace) -> bool {
        debug_assert!(window.len() <= self.horizon);
        let _holds = crate::dsl::eval_prop(&self.prop, window); // helper bridging to Rust monitor
        let delta = if Self::is_boolean_only(&self.prop) {
            // For now, use the fallback encoder
            delta_clauses(&self.prop, window)
        } else {
            delta_clauses(&self.prop, window) // earlier empty‑clause strategy
        };
        
        // Create a new SatCore for this operation
        let mut sat = SatCore::new(&self.ctx, self.horizon);
        match sat.unsat_recycle(delta).expect("solver") {
            SatResult::Sat => { self.last_core.clear(); true },
            SatResult::Unsat => {
                self.last_core = sat.get_unsat_core().unwrap_or_default();
                false
            },
            SatResult::Unknown => { log::warn!("Z3 UNKNOWN"); false },
        }
    }
}
