fn main() {
    cc::Build::new()
        .file("blake3.c")
        .file("ffi.c")
        .include(".")
        .compile("sentinelffi");
}
