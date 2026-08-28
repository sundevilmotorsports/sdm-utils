/* Flow:
 *   can_ota_init(&ota, &cbs);
 *   can_ota_begin(&ota, image_size, expected_crc32);   // -> flash_begin()
 *   can_ota_chunk(&ota, 0,   buf, n);                  // -> flash_write()
 *   can_ota_chunk(&ota, n,   buf, n);                  // ... in order
 *   can_ota_end(&ota);                                 // -> flash_end(), done()
 */
#ifndef CAN_OTA_H
#define CAN_OTA_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif


typedef enum {
    CAN_OTA_OK = 0,
    CAN_OTA_ERR_STATE,   /* call made in the wrong state */
    CAN_OTA_ERR_OFFSET,  /* chunk offset did not match expected position */
    CAN_OTA_ERR_SIZE,    /* data would exceed the declared image size */
    CAN_OTA_ERR_CRC,     /* CRC32 mismatch at can_ota_end() */
    CAN_OTA_ERR_FLASH,   /* a flash_* callback returned nonzero */
    CAN_OTA_ERR_ARG,     /* NULL pointer or zero size */
} can_ota_result;

typedef enum {
    CAN_OTA_STATE_IDLE = 0,
    CAN_OTA_STATE_RECEIVING,
    CAN_OTA_STATE_DONE,
    CAN_OTA_STATE_ERROR,
} can_ota_state;

/* Every callback takes the `user` pointer from can_ota_callbacks.
 * flash_* return 0 on success, nonzero on failure. */
typedef struct {
    int  (*flash_begin)(void *user, uint32_t image_size);
    int  (*flash_write)(void *user, uint32_t offset, const void *data, uint32_t len);
    int  (*flash_end)(void *user);
    void (*done)(void *user);   /* optional: image written and verified */
    void *user;
} can_ota_callbacks;

typedef struct {
    const can_ota_callbacks *cb;
    can_ota_state state;
    uint32_t image_size;
    uint32_t expected_crc;
    uint32_t offset;   /* bytes written so far */
    uint32_t crc;      /* running CRC32 of bytes written so far */
} can_ota;

/* IEEE 802.3. Sender uses this to compute the value passed to can_ota_begin(). */
uint32_t can_ota_crc32(const void *data, size_t len);
uint32_t can_ota_crc32_update(uint32_t crc, const void *data, size_t len);

void           can_ota_init(can_ota *ota, const can_ota_callbacks *cb);
can_ota_result can_ota_begin(can_ota *ota, uint32_t image_size, uint32_t expected_crc);
can_ota_result can_ota_chunk(can_ota *ota, uint32_t offset, const void *data, uint32_t len);
can_ota_result can_ota_end(can_ota *ota);
void           can_ota_abort(can_ota *ota);

static inline can_ota_state can_ota_get_state(const can_ota *ota) { return ota->state; }
static inline uint32_t      can_ota_progress(const can_ota *ota)  { return ota->offset; }

#ifdef __cplusplus
}
#endif

#endif /* CAN_OTA_H */
