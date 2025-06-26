import Lake
open Lake DSL
open System

package sentinel_monitor where
  srcDir := "."
  moreLeanArgs := #["-DLakeExportRuntime"]
  moreServerArgs := #[]
  -- If you use mathlib:
  -- extraDepTargets := #[`Mathlib]

-- Compile C files into a shared library
extern_lib sentinel_monitor_c where
  srcDir := "ffi"
  srcFiles := #["ffi.c", "blake3.c"]
  buildStatic := false

-- Copy the shared lib for Rust
target copySharedLib : FilePath := do
  let lib := "build" / "lib" / ("libsentinel_monitor_c" ++ sharedLibExt)
  let out := ".." / "target" / ("libsentinel_monitor" ++ sharedLibExt)
  IO.FS.createDirAll (out.parentD!)
  IO.FS.copyFile lib out (overwrite := true)
  return out
