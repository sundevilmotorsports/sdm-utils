/* Extended 29-bit identifiers, low two bytes used:
 *   bits 28..16 : reserved (0)
 *   bits 15.. 8 : message type
 *   bits  7.. 0 : node id
 */
#ifndef SDM_CAN_ID_H
#define SDM_CAN_ID_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Mirror any change to the shifts or enums below into rust/src/lib.rs. */
#define SDM_CAN_MSG_SHIFT  8
#define SDM_CAN_NODE_SHIFT 0

enum sdm_node {
    SDM_NODE_NONE      = 0x00,
    SDM_NODE_LOGGER    = 0x01,
    SDM_NODE_BROADCAST = 0xFF,
};

enum sdm_msg {
    SDM_MSG_FAULT     = 0x00,
    SDM_MSG_HEARTBEAT = 0x01,

    SDM_MSG_OTA_ACK   = 0xF0,   /* target -> sender: [u32 offset][u8 status]  */
    SDM_MSG_OTA_START = 0xF1,   /* sender -> target: [u32 size][u32 crc32]     */
    SDM_MSG_OTA_END   = 0xF2,   /* sender -> target: empty                     */
    SDM_MSG_OTA_DATA  = 0xF3,   /* sender -> target: [u8 seq][<=7 fw bytes]    */
};

static inline uint32_t sdm_can_id(uint8_t msg, uint8_t node) {
    return ((uint32_t)msg << SDM_CAN_MSG_SHIFT) | ((uint32_t)node << SDM_CAN_NODE_SHIFT);
}
static inline uint8_t sdm_can_id_msg(uint32_t id)  { return (id >> SDM_CAN_MSG_SHIFT)  & 0xFFu; }
static inline uint8_t sdm_can_id_node(uint32_t id) { return (id >> SDM_CAN_NODE_SHIFT) & 0xFFu; }

#ifdef __cplusplus
}
#endif

#endif /* SDM_CAN_ID_H */
