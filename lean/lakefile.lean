import Lake
open Lake.DSL
open System (FilePath)
open System.FilePath (mk)

def sharedExt : String := ".so"  -- adjust to ".dll" on Windows or ".dylib" on macOS

package sentinel_monitor
  (name := "sentinel_monitor")
  (srcDir := ".")
  (moreLeanArgs := #["-DLakeExportRuntime"])

@[default_target]
extern_lib sentinel_monitor_c
  (name := "sentinel_monitor_c")
  (srcDir    := "ffi")
  (srcFiles  := #["ffi.c", "blake3.c"])
  (sharedLib := true)

script copySharedLib := do
  let lib    := mk "build" / mk "lib" / mk ("libsentinel_monitor_c" ++ sharedExt)
  let outDir := mk ".."    / mk "target"
  let out    := outDir     / mk ("libsentinel_monitor"    ++ sharedExt)
  IO.FS.createDirAll outDir
  let bytes ← IO.FS.readBinFile lib
  IO.FS.writeBinFile out bytes
  IO.println s!"Copied {lib} to {out}"
  pure 0
