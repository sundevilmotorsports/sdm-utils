use core::ffi::{c_int, c_void};

mod sys {
    #![allow(non_camel_case_types, non_upper_case_globals, dead_code)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use sys::{can_ota_result as Error, can_ota_state as State};

/// CRC32 (IEEE 802.3); a sender passes this to [`Ota::begin`].
pub fn crc32(data: &[u8]) -> u32 {
    unsafe { sys::can_ota_crc32(data.as_ptr().cast(), data.len()) }
}

/// A [`Flash`] operation failed (maps to a nonzero C callback return).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FlashError;

/// Storage backend for the received image.
pub trait Flash {
    fn begin(&mut self, image_size: u32) -> Result<(), FlashError>;
    fn write(&mut self, offset: u32, data: &[u8]) -> Result<(), FlashError>;
    fn end(&mut self) -> Result<(), FlashError>;
    fn done(&mut self) {}
}

fn to_c(r: Result<(), FlashError>) -> c_int {
    r.map_or(-1, |_| 0)
}
fn check(r: Error) -> Result<(), Error> {
    if r == Error::CAN_OTA_OK {
        Ok(())
    } else {
        Err(r)
    }
}

unsafe extern "C" fn t_begin<F: Flash>(u: *mut c_void, n: u32) -> c_int {
    to_c((*u.cast::<F>()).begin(n))
}
unsafe extern "C" fn t_write<F: Flash>(u: *mut c_void, off: u32, d: *const c_void, n: u32) -> c_int {
    to_c((*u.cast::<F>()).write(off, core::slice::from_raw_parts(d.cast(), n as usize)))
}
unsafe extern "C" fn t_end<F: Flash>(u: *mut c_void) -> c_int {
    to_c((*u.cast::<F>()).end())
}
unsafe extern "C" fn t_done<F: Flash>(u: *mut c_void) {
    (*u.cast::<F>()).done()
}

/// CAN-OTA receiver. Movable: the pointers the C struct caches are refreshed
/// before every call, and C only calls back synchronously within a call.
pub struct Ota<F: Flash> {
    c: sys::can_ota,
    cbs: sys::can_ota_callbacks,
    flash: F,
}

impl<F: Flash> Ota<F> {
    pub fn new(flash: F) -> Self {
        let mut ota = Ota {
            c: unsafe { core::mem::zeroed() },
            cbs: sys::can_ota_callbacks {
                flash_begin: Some(t_begin::<F>),
                flash_write: Some(t_write::<F>),
                flash_end: Some(t_end::<F>),
                done: Some(t_done::<F>),
                user: core::ptr::null_mut(),
            },
            flash,
        };
        ota.sync();
        unsafe { sys::can_ota_init(&mut ota.c, &ota.cbs) };
        ota
    }

    fn sync(&mut self) {
        self.cbs.user = core::ptr::addr_of_mut!(self.flash).cast();
        self.c.cb = core::ptr::addr_of!(self.cbs);
    }

    /// Start receiving `image_size` bytes with the given CRC32.
    pub fn begin(&mut self, image_size: u32, expected_crc: u32) -> Result<(), Error> {
        self.sync();
        check(unsafe { sys::can_ota_begin(&mut self.c, image_size, expected_crc) })
    }

    /// Feed the next chunk; `offset` must equal [`Ota::progress`].
    pub fn chunk(&mut self, offset: u32, data: &[u8]) -> Result<(), Error> {
        self.sync();
        check(unsafe {
            sys::can_ota_chunk(&mut self.c, offset, data.as_ptr().cast(), data.len() as u32)
        })
    }

    /// Verify size + CRC, finalize flash, call [`Flash::done`].
    pub fn end(&mut self) -> Result<(), Error> {
        self.sync();
        check(unsafe { sys::can_ota_end(&mut self.c) })
    }

    pub fn abort(&mut self) {
        self.sync();
        unsafe { sys::can_ota_abort(&mut self.c) };
    }

    pub fn progress(&self) -> u32 {
        self.c.offset
    }
    pub fn state(&self) -> State {
        self.c.state
    }
    pub fn flash_mut(&mut self) -> &mut F {
        &mut self.flash
    }
}
