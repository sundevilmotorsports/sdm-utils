//! Binary log file format: written by the logger firmware, read by the desktop
//! client. Keep in sync with `include/sdm/logfmt.h`.
//!
//! Header: `[u8 num_columns]`, then per column `[u8 name_len][name][u8 tag]`.
//! Tag `0` is a 4-byte little-endian `f32`; tag `n` (`1..=255`) is `n` opaque
//! bytes. Fixed-width rows follow, columns in header order, until EOF.
//!
//! Convention: column 0 is an 8-byte raw millisecond timestamp named
//! `"timestamp"`. The format does not enforce it.

use alloc::string::String;
use alloc::vec::Vec;

/// Column value type, as stored in the header's tag byte.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ColType {
    /// 4-byte little-endian IEEE-754 float.
    F32,
    /// `n` opaque bytes (`1..=255`); readers treat them as a little-endian uint.
    Raw(u8),
}

impl ColType {
    /// The header tag byte for this type.
    pub const fn tag(self) -> u8 {
        match self {
            ColType::F32 => 0,
            ColType::Raw(n) => n,
        }
    }

    /// Inverse of [`ColType::tag`].
    pub const fn from_tag(tag: u8) -> ColType {
        match tag {
            0 => ColType::F32,
            n => ColType::Raw(n),
        }
    }

    /// Bytes this column occupies in a row.
    pub const fn width(self) -> usize {
        match self {
            ColType::F32 => 4,
            ColType::Raw(n) => n as usize,
        }
    }
}

/// One named column.
#[derive(Clone, Debug)]
pub struct Column {
    pub name: String,
    pub ty: ColType,
}

/// The ordered column list at the head of every log file.
#[derive(Clone, Debug, Default)]
pub struct Schema {
    pub columns: Vec<Column>,
}

/// A malformed or truncated log header.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Error {
    /// The header ran past the end of the input.
    Truncated,
    /// A column name was not valid UTF-8.
    Name,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Error::Truncated => "truncated log header",
            Error::Name => "column name is not valid UTF-8",
        })
    }
}

impl Schema {
    pub fn new() -> Schema {
        Schema::default()
    }

    /// Append a column.
    pub fn push(&mut self, name: impl Into<String>, ty: ColType) {
        self.columns.push(Column {
            name: name.into(),
            ty,
        });
    }

    /// Bytes one row occupies.
    pub fn row_width(&self) -> usize {
        self.columns.iter().map(|c| c.ty.width()).sum()
    }

    /// Encode the file header.
    pub fn encode_header(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(self.columns.len() as u8);
        for c in &self.columns {
            out.push(c.name.len() as u8);
            out.extend_from_slice(c.name.as_bytes());
            out.push(c.ty.tag());
        }
        out
    }

    /// Decode a header from the front of `input`, advancing it to the first row.
    pub fn decode_header(input: &mut &[u8]) -> Result<Schema, Error> {
        let count = take(input, 1)?[0] as usize;
        let mut columns = Vec::with_capacity(count);
        for _ in 0..count {
            let name_len = take(input, 1)?[0] as usize;
            let name = core::str::from_utf8(take(input, name_len)?).map_err(|_| Error::Name)?;
            let tag = take(input, 1)?[0];
            columns.push(Column {
                name: name.into(),
                ty: ColType::from_tag(tag),
            });
        }
        Ok(Schema { columns })
    }

    /// Iterate the fixed-width rows in `body` 
    pub fn rows<'a>(&'a self, body: &'a [u8]) -> Rows<'a> {
        Rows {
            columns: &self.columns,
            body,
            width: self.row_width(),
        }
    }
}

fn take<'a>(input: &mut &'a [u8], n: usize) -> Result<&'a [u8], Error> {
    if input.len() < n {
        return Err(Error::Truncated);
    }
    let (head, tail) = input.split_at(n);
    *input = tail;
    Ok(head)
}

/// One decoded cell. `Raw` borrows straight out of the log bytes.
#[derive(Clone, Copy, Debug)]
pub enum Value<'a> {
    F32(f32),
    Raw(&'a [u8]),
}

impl Value<'_> {
    /// Render for a CSV cell: floats verbatim, raw bytes as a little-endian
    /// unsigned integer
    pub fn render(&self) -> String {
        use alloc::format;
        match self {
            Value::F32(v) => format!("{v}"),
            Value::Raw(bytes) => {
                let mut buf = [0u8; 8];
                let n = bytes.len().min(8);
                buf[..n].copy_from_slice(&bytes[..n]);
                format!("{}", u64::from_le_bytes(buf))
            }
        }
    }
}

/// Iterator over a log's rows; see [`Schema::rows`].
pub struct Rows<'a> {
    columns: &'a [Column],
    body: &'a [u8],
    width: usize,
}

impl<'a> Iterator for Rows<'a> {
    type Item = Row<'a>;

    fn next(&mut self) -> Option<Row<'a>> {
        if self.width == 0 || self.body.len() < self.width {
            return None;
        }
        let (row, rest) = self.body.split_at(self.width);
        self.body = rest;
        Some(Row {
            columns: self.columns,
            data: row,
        })
    }
}

/// One row; iterate its cells with [`Row::values`].
pub struct Row<'a> {
    columns: &'a [Column],
    data: &'a [u8],
}

impl<'a> Row<'a> {
    pub fn values(&self) -> impl Iterator<Item = Value<'a>> + 'a {
        let data = self.data;
        let mut offset = 0;
        self.columns.iter().map(move |c| {
            let slice = &data[offset..offset + c.ty.width()];
            offset += c.ty.width();
            match c.ty {
                ColType::F32 => {
                    let mut buf = [0u8; 4];
                    buf.copy_from_slice(slice);
                    Value::F32(f32::from_le_bytes(buf))
                }
                ColType::Raw(_) => Value::Raw(slice),
            }
        })
    }
}

