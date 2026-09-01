/* Binary log wire format: written by the logger, read by the desktop client.
 *
 * Header:
 *   [u8  num_columns]
 *   num_columns times:
 *     [u8  name_len]
 *     [name_len bytes  column name, UTF-8, not NUL-terminated]
 *     [u8  type_tag]   see enum sdm_logcol
 *
 * Rows: fixed width, back to back until EOF, columns in header order.
 *   f32 column  -> 4 bytes, little-endian IEEE-754
 *   raw column  -> type_tag bytes, opaque (readers render them as a
 *                  little-endian unsigned integer)
 * A trailing partial row is ignored.
 *
 * Convention: column 0 is an 8-byte raw millisecond timestamp named
 * "timestamp". The format does not enforce it.
 *
 * Mirror any change here into rust/src/logfmt.rs.
 */
#ifndef SDM_LOGFMT_H
#define SDM_LOGFMT_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

enum sdm_logcol {
    SDM_LOGCOL_F32 = 0, /* 4-byte little-endian float                */
                        /* 1..255: raw column, value is that many bytes */
};

/* Bytes one row occupies, given each column's type tag (0 = f32). */
static inline size_t sdm_logfmt_row_width(const uint8_t *tags, size_t n) {
    size_t width = 0;
    for (size_t i = 0; i < n; i++) {
        width += tags[i] ? (size_t)tags[i] : 4u;
    }
    return width;
}

#ifdef __cplusplus
}
#endif

#endif /* SDM_LOGFMT_H */
