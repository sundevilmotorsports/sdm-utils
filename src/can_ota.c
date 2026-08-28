#include "sdm/can_ota.h"

uint32_t can_ota_crc32_update(uint32_t crc, const void *data, size_t len) {
    const uint8_t *p = (const uint8_t *)data;
    crc = ~crc;
    for (size_t i = 0; i < len; i++) {
        crc ^= p[i];
        for (int k = 0; k < 8; k++)
            crc = (crc >> 1) ^ (0xEDB88320u & -(crc & 1u));
    }
    return ~crc;
}

uint32_t can_ota_crc32(const void *data, size_t len) {
    return can_ota_crc32_update(0, data, len);
}

void can_ota_init(can_ota *ota, const can_ota_callbacks *cb) {
    ota->cb = cb;
    ota->state = CAN_OTA_STATE_IDLE;
    ota->image_size = 0;
    ota->expected_crc = 0;
    ota->offset = 0;
    ota->crc = 0;
}

static can_ota_result fail(can_ota *ota, can_ota_result r) {
    ota->state = CAN_OTA_STATE_ERROR;
    return r;
}

can_ota_result can_ota_begin(can_ota *ota, uint32_t image_size, uint32_t expected_crc) {
    if (image_size == 0)
        return CAN_OTA_ERR_ARG;
    if (ota->state == CAN_OTA_STATE_RECEIVING)
        return CAN_OTA_ERR_STATE;

    ota->image_size = image_size;
    ota->expected_crc = expected_crc;
    ota->offset = 0;
    ota->crc = 0;
    ota->state = CAN_OTA_STATE_RECEIVING;

    if (ota->cb->flash_begin && ota->cb->flash_begin(ota->cb->user, image_size) != 0)
        return fail(ota, CAN_OTA_ERR_FLASH);
    return CAN_OTA_OK;
}

can_ota_result can_ota_chunk(can_ota *ota, uint32_t offset, const void *data, uint32_t len) {
    if (data == NULL || len == 0)
        return CAN_OTA_ERR_ARG;
    if (ota->state != CAN_OTA_STATE_RECEIVING)
        return CAN_OTA_ERR_STATE;
    if (offset != ota->offset)
        return CAN_OTA_ERR_OFFSET;
    if (len > ota->image_size - ota->offset)
        return fail(ota, CAN_OTA_ERR_SIZE);

    if (ota->cb->flash_write &&
        ota->cb->flash_write(ota->cb->user, offset, data, len) != 0)
        return fail(ota, CAN_OTA_ERR_FLASH);

    ota->crc = can_ota_crc32_update(ota->crc, data, len);
    ota->offset += len;
    return CAN_OTA_OK;
}

can_ota_result can_ota_end(can_ota *ota) {
    if (ota->state != CAN_OTA_STATE_RECEIVING)
        return CAN_OTA_ERR_STATE;
    if (ota->offset != ota->image_size)
        return fail(ota, CAN_OTA_ERR_SIZE);
    if (ota->crc != ota->expected_crc)
        return fail(ota, CAN_OTA_ERR_CRC);
    if (ota->cb->flash_end && ota->cb->flash_end(ota->cb->user) != 0)
        return fail(ota, CAN_OTA_ERR_FLASH);

    ota->state = CAN_OTA_STATE_DONE;
    if (ota->cb->done)
        ota->cb->done(ota->cb->user);
    return CAN_OTA_OK;
}

void can_ota_abort(can_ota *ota) {
    ota->state = CAN_OTA_STATE_IDLE;
    ota->offset = 0;
    ota->crc = 0;
}
