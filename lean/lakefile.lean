import Lake
open Lake DSL

package sentinel_monitor where
  srcDir := "."
  moreLeanArgs := #["-DLakeExportRuntime"]

/-- Our Lean code lives in `PropSound.lean` and `TseitinSound.lean` so we
    expose a library named `Sentinel`. -/
lean_lib Sentinel
