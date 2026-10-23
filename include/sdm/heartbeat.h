// byte 0 : sender device type (enum sdm_type)

#ifndef SDM_HEARTBEAT_H
#define SDM_HEARTBEAT_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define SDM_HEARTBEAT_TYPE_OFFSET 0

enum sdm_type {
    SDM_TYPE_UNKNOWN = 0x00,
    SDM_TYPE_LOGGER  = 0x01,
    SDM_TYPE_IMU     = 0x02,
    SDM_TYPE_COOLANT = 0x03,
};

#ifdef __cplusplus
}
#endif

#endif /* SDM_HEARTBEAT_H */
