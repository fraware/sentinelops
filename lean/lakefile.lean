/-
  lakefile.lean  ── Build configuration for the SentinelOps Lean project.

  Directories
  ───────────
  • `Sentinel/` namespace (in `PropSound.lean`, `TseitinSound.lean`)
  • `ffi/`      – C source for Blake3 and the `sentinel_cert_hash` shim
  • `build/lib` – output shared library `libsentinel_monitor.{so,dylib}`
-/

import Lake
open Lake DSL

package sentinel_monitor where
  -- Lean sources live next to the lakefile
  srcDir := "."

  -- Build a shared library so Rust can link against it.
  -- `ffi/ffi.c` and `ffi/blake3.c` are compiled & linked in.
  moreLeanArgs := #["-DLakeExportRuntime"]     -- export symbols
  moreServerArgs := #[]                        -- LSP server

  extraDepTargets := #[`Mathlib]

  -- C objects to build and link
  extraObjs :=
    #[("ffi/ffi.c"), ("ffi/blake3.c")]

/-- Helper target: build + copy so the Rust workspace can find it under `../target/`. -/
target copySharedLib : FilePath := do
  let lib ← fetch <| by
    target `shared «Package.artifactFile»
  let out := "../target/libsentinel_monitor.so"
  IO.FS.createDirAll out.parent!
  IO.FS.copyFile lib out (overwrite := true)
  pure out

-- Default build also materialises the copy
packageDepTargets := #[`copySharedLib]
