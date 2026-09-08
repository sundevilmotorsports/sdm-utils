/// Result of an [`Ota`] state-machine call.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Error {
    /// Call made in the wrong state.
    State,
    /// Chunk offset did not match the expected write position.
    Offset,
    /// Data would exceed the declared image size.
    Size,
    /// CRC32 mismatch at [`Ota::end`].
    Crc,
    /// A [`Flash`] operation failed.
    Flash,
    /// Zero image size, or an empty chunk.
    Arg,
}

/// Transfer state; read with [`Ota::state`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum State {
    #[default]
    Idle,
    Receiving,
    Done,
    Error,
}

/// A [`Flash`] operation failed
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FlashError;

/// Storage backend for the received image.
pub trait Flash {
    fn begin(&mut self, image_size: u32) -> Result<(), FlashError>;
    fn write(&mut self, offset: u32, data: &[u8]) -> Result<(), FlashError>;
    fn end(&mut self) -> Result<(), FlashError>;
    fn done(&mut self) {}
}

/// CRC32 (IEEE 802.3); a sender passes this to [`Ota::begin`].
pub fn crc32(data: &[u8]) -> u32 {
    crc32_update(0, data)
}

/// CAN-OTA receiver. Drive it `new` → `begin` → `chunk`… → `end`.
pub struct Ota<F: Flash> {
    state: State,
    image_size: u32,
    expected_crc: u32,
    offset: u32,
    crc: u32,
    flash: F,
}

impl<F: Flash> Ota<F> {
    pub fn new(flash: F) -> Self {
        Ota {
            state: State::Idle,
            image_size: 0,
            expected_crc: 0,
            offset: 0,
            crc: 0,
            flash,
        }
    }

    fn fail(&mut self, e: Error) -> Error {
        self.state = State::Error;
        e
    }

    /// Map a flash result into the state machine, going to [`State::Error`] on failure.
    fn io(&mut self, r: Result<(), FlashError>) -> Result<(), Error> {
        match r {
            Ok(()) => Ok(()),
            Err(FlashError) => Err(self.fail(Error::Flash)),
        }
    }

    /// Start receiving `image_size` bytes with the given CRC32.
    pub fn begin(&mut self, image_size: u32, expected_crc: u32) -> Result<(), Error> {
        if image_size == 0 {
            return Err(Error::Arg);
        }
        if self.state == State::Receiving {
            return Err(Error::State);
        }
        self.image_size = image_size;
        self.expected_crc = expected_crc;
        self.offset = 0;
        self.crc = 0;
        self.state = State::Receiving;

        let r = self.flash.begin(image_size);
        self.io(r)
    }

    /// Feed the next chunk; `offset` must equal [`Ota::progress`].
    pub fn chunk(&mut self, offset: u32, data: &[u8]) -> Result<(), Error> {
        if data.is_empty() {
            return Err(Error::Arg);
        }
        if self.state != State::Receiving {
            return Err(Error::State);
        }
        if offset != self.offset {
            return Err(Error::Offset);
        }
        if data.len() as u32 > self.image_size - self.offset {
            return Err(self.fail(Error::Size));
        }

        let r = self.flash.write(offset, data);
        self.io(r)?;

        self.crc = crc32_update(self.crc, data);
        self.offset += data.len() as u32;
        Ok(())
    }

    /// Verify size + CRC, finalize flash, call [`Flash::done`].
    pub fn end(&mut self) -> Result<(), Error> {
        if self.state != State::Receiving {
            return Err(Error::State);
        }
        if self.offset != self.image_size {
            return Err(self.fail(Error::Size));
        }
        if self.crc != self.expected_crc {
            return Err(self.fail(Error::Crc));
        }

        let r = self.flash.end();
        self.io(r)?;

        self.state = State::Done;
        self.flash.done();
        Ok(())
    }

    pub fn abort(&mut self) {
        self.state = State::Idle;
        self.offset = 0;
        self.crc = 0;
    }

    pub fn progress(&self) -> u32 {
        self.offset
    }
    pub fn state(&self) -> State {
        self.state
    }
    pub fn flash_mut(&mut self) -> &mut F {
        &mut self.flash
    }
}

/// Bit-serial CRC32, matching `can_ota_crc32_update` in the C source.
fn crc32_update(crc: u32, data: &[u8]) -> u32 {
    let mut crc = !crc;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            // C: (crc >> 1) ^ (0xEDB88320u & -(crc & 1u))
            crc = (crc >> 1) ^ (0xEDB8_8320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}
