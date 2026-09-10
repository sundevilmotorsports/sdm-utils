# sdm-utils

## Building a node

1. **Pick a node id.** A `sdm_node` / `Node` value, unique on your bus. Every id you send is
   `sdm_can_id(msg, your_id)` (C) / `can_id(msg, your_id)` (Rust).

2. **Dispatch each frame on its message type**, ignoring ids for other nodes:
   ```c
   if (sdm_can_id_node(id) != MY_NODE) continue;
   switch (sdm_can_id_msg(id)) {
   case SDM_MSG_IDENTIFY: blink_led(); break;
   ...
   }
   ```
   ```rust
   match Msg::from_byte(can_id_msg(id)) {
       Some(Msg::Identify) => blink_led(),
       ...
   }
   ```

3. **Implement each message:**
   - `HEARTBEAT` — you send this one, on your own timer every 500 ms. Put your device type in byte 0.
   - `FAULT` — send it when something goes wrong on your end.
   - `IDENTIFY` — on receipt, blink your LED for a few seconds. No reply.
   - `START` / `STOP` / `RESTART` — on receipt, actually change your run state. No ack for these

## firmware updates

`can_ota` (C) / `ota::Ota` (Rust) already do the CRC32 and the transfer state machine. You add:

1. **A flash backend** — 3 calls, begin/write/end. C: fill in `can_ota_callbacks`. Rust: `impl Flash`.

2. **Framing** — feed OTA frames into it from the same dispatch as above, then ack so the sender
   knows where you are:
   ```c
   case SDM_MSG_OTA_START: send_ack(can_ota_begin(&ota, size, crc));       break;
   case SDM_MSG_OTA_DATA:  send_ack(can_ota_chunk(&ota, can_ota_progress(&ota), &data[1], len - 1)); break;
   case SDM_MSG_OTA_END:   send_ack(can_ota_end(&ota));                    break;
   ```
   ```rust
   Some(Msg::OtaStart) => send_ack(ota.begin(size, crc)),
   Some(Msg::OtaData)  => send_ack(ota.chunk(ota.progress(), &data[1..])),
   Some(Msg::OtaEnd)   => send_ack(ota.end()),
   ```
   where `send_ack` sends `OTA_ACK [u32 offset][u8 status]`.
