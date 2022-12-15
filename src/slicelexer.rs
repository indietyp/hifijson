use crate::{error, escape, Escape, Lexer, NumParts, Read};
use core::num::NonZeroUsize;

/// JSON lexer from a shared byte slice.
pub struct SliceLexer<'a> {
    slice: &'a [u8],
}

impl<'a> SliceLexer<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        Self { slice }
    }
}

fn digits(s: &[u8]) -> usize {
    s.iter()
        .position(|c| !matches!(c, b'0'..=b'9'))
        .unwrap_or(s.len())
}

impl<'a> Read for SliceLexer<'a> {
    type Bytes = &'a [u8];

    fn strip_prefix<const N: usize>(&mut self, s: [u8; N]) -> bool {
        if let Some(rest) = self.slice.strip_prefix(&s) {
            self.slice = rest;
            true
        } else {
            false
        }
    }

    fn peek_byte(&self) -> Option<&u8> {
        self.slice.first()
    }

    fn read_byte(&mut self) -> Option<u8> {
        let (head, rest) = self.slice.split_first()?;
        self.slice = rest;
        Some(*head)
    }

    fn skip_until(&mut self, stop: impl FnMut(u8) -> bool) {
        self.read_until(&mut &[][..], stop)
    }

    fn read_until(&mut self, bytes: &mut &'a [u8], mut stop: impl FnMut(u8) -> bool) {
        let pos = self.slice.iter().position(|c| stop(*c));
        let pos = pos.unwrap_or(self.slice.len());
        *bytes = &self.slice[..pos];
        self.slice = &self.slice[pos..]
    }
}

impl<'a> Lexer for SliceLexer<'a> {
    type Num = &'a str;

    fn num_bytes(&mut self, bytes: &mut Self::Bytes) -> Result<NumParts, error::Num> {
        let mut pos = usize::from(self.slice[0] == b'-');
        let mut parts = NumParts::default();

        let digits1 = |s| NonZeroUsize::new(digits(s)).ok_or(error::Num::ExpectedDigit);

        pos += if self.slice.get(pos) == Some(&b'0') {
            1
        } else {
            digits1(&self.slice[pos..])?.get()
        };

        loop {
            match self.slice.get(pos) {
                Some(b'.') if parts.dot.is_none() && parts.exp.is_none() => {
                    parts.dot = Some(NonZeroUsize::new(pos).unwrap());
                    pos += 1;
                    pos += digits1(&self.slice[pos..])?.get()
                }
                Some(b'e' | b'E') if parts.exp.is_none() => {
                    parts.exp = Some(NonZeroUsize::new(pos).unwrap());
                    pos += 1;
                    if matches!(self.slice.get(pos), Some(b'+' | b'-')) {
                        pos += 1;
                    }
                    pos += digits1(&self.slice[pos..])?.get()
                }
                None | Some(_) => {
                    *bytes = &self.slice[..pos];
                    self.slice = &self.slice[pos..];
                    return Ok(parts);
                }
            }
        }
    }

    fn num_string(&mut self) -> Result<(Self::Num, NumParts), error::Num> {
        let mut num = Default::default();
        let pos = self.num_bytes(&mut num)?;
        // SAFETY: conversion to UTF-8 always succeeds because
        // lex_number validates everything it writes to num
        Ok((core::str::from_utf8(num).unwrap(), pos))
    }

    fn escape(&mut self) -> Result<Escape, error::Escape> {
        let typ = self.slice.first().ok_or(error::Escape::Eof)?;
        self.slice = &self.slice[1..];
        let escape = Escape::try_from(*typ).ok_or(error::Escape::UnknownKind)?;
        if matches!(escape, Escape::Unicode(_)) {
            let hex = self.slice.get(..4).ok_or(error::Escape::Eof)?;
            // SAFETY: `unwrap()` always succeeds, because `slice.get(..4)`
            // must return a slice of size 4 if it succeeds
            let hex: [u8; 4] = hex.try_into().unwrap();
            self.slice = &self.slice[4..];
            let hex = escape::decode_hex4(hex).ok_or(error::Escape::InvalidHex)?;
            Ok(Escape::Unicode(hex))
        } else {
            Ok(escape)
        }
    }
}
