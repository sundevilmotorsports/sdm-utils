#![cfg_attr(not(test), no_std)]

#[cfg(any(feature = "alloc", test))]
extern crate alloc;

#[cfg(any(feature = "alloc", test))]
pub mod logfmt;

pub mod ota;

pub const MSG_SHIFT: u32 = 8;
pub const NODE_SHIFT: u32 = 0;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Node {
    None = 0x00,
    Logger = 0x01,
    Broadcast = 0xFF,
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Msg {
    Fault = 0x00,
    Heartbeat = 0x01,
    OtaAck = 0xF0,
    OtaStart = 0xF1,
    OtaEnd = 0xF2,
    OtaData = 0xF3,
}

pub const fn can_id(msg: u8, node: u8) -> u32 {
    ((msg as u32) << MSG_SHIFT) | ((node as u32) << NODE_SHIFT)
}
pub const fn can_id_msg(id: u32) -> u8 {
    (id >> MSG_SHIFT) as u8
}
pub const fn can_id_node(id: u32) -> u8 {
    (id >> NODE_SHIFT) as u8
}

/// Device class, carried as byte 0 of a [`Msg::Heartbeat`] payload.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DeviceType {
    Unknown = 0x00,
    Logger = 0x01,
}

impl DeviceType {
    pub const fn from_byte(b: u8) -> Self {
        match b {
            0x01 => Self::Logger,
            _ => Self::Unknown,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Logger => "logger",
        }
    }
}
