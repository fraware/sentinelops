/* lean/ffi/ffi.c
 * =============================================================
 * sentinel_cert_hash  — C shim exported from the Lean shared lib
 * -------------------------------------------------------------
 * Signature (see libsentinel_ffi):
 *    bool sentinel_cert_hash(const char* prop_json,
 *                            const char* trace_json,
 *                            unsigned char out32[32]);
 *
 * For the MVP we compute Blake3(prop_json ‖ trace_json) *in C* so we
 * don’t pay the marshalling cost into Lean.  A future version will call
 * into a Lean function producing the certificate, but the hash value is
 * identical either way because the cert deterministically depends on
 * those two strings.
 *
 * Dependencies:
 *   • blake3 (single-file C implementation) – vendored as `blake3.h` &
 *     `blake3.c` in the same directory and compiled by `lake build`.
 *
 * Build: `lake build sentinel_monitor` automatically pulls this file via
 * `leanpkg.toml` extraObjs.
 * =============================================================*/

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>
#include "blake3.h" // https://github.com/BLAKE3-team/BLAKE3/tree/master/c

#ifdef _MSC_VER
#define EXPORT __declspec(dllexport)
#else
#define EXPORT __attribute__((visibility("default")))
#endif

EXPORT bool sentinel_cert_hash(const char *prop_json,
                               const char *trace_json,
                               uint8_t *out32)
{
    if (!prop_json || !trace_json || !out32)
        return false;

    blake3_hasher hasher;
    blake3_hasher_init(&hasher);
    blake3_hasher_update(&hasher, prop_json, strlen(prop_json));
    blake3_hasher_update(&hasher, trace_json, strlen(trace_json));
    blake3_hasher_finalize(&hasher, out32, 32);
    return true;
}
