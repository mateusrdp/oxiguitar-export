// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Mateus Rocha de Paula and the OxiGuitar contributors

//! WAV: 16- or 24-bit integers or 32-bit float, at 44.1 or 48 kHz.
//!
//! The header names the length, so the one written first has zero there
//! and OxiGuitar writes the one handed back at the end over it. A WAV file
//! holds at most 4 GiB of sound (its sizes are 32 bits): about 3 h 40 min
//! at 24 bits and 48 kHz. Past it the export fails, saying so.

use oxiguitar_plugin::{Error, ErrorKind};

use crate::pcm::Quantizer;

/// What a WAV file holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Depth {
    /// 16-bit integers, as CDs.
    Sixteen,
    /// 24-bit integers.
    TwentyFour,
    /// 32-bit floating point.
    Float,
}

impl Depth {
    /// The depth the option `key` names (`choices`), 24 bits otherwise.
    pub fn of(key: &str) -> Depth {
        match key {
            "16" => Depth::Sixteen,
            "32f" => Depth::Float,
            _ => Depth::TwentyFour,
        }
    }

    /// Bytes a sample.
    pub fn bytes(self) -> u16 {
        match self {
            Depth::Sixteen => 2,
            Depth::TwentyFour => 3,
            Depth::Float => 4,
        }
    }
}

/// A WAV header for `frames` stereo frames: 44 bytes for integers; 58 for
/// float (an 18-byte format and a `fact` chunk, as the format asks for
/// samples that aren't integers).
pub fn header(depth: Depth, rate: u32, frames: u64) -> Vec<u8> {
    let float = depth == Depth::Float;
    let block = 2 * u32::from(depth.bytes());
    let data = (frames * u64::from(block)).min(u64::from(u32::MAX)) as u32;
    let format_len: u32 = if float { 18 } else { 16 };
    let fact_len: u32 = if float { 12 } else { 0 };
    let riff = 4 + (8 + format_len) + fact_len + 8 + data;
    let mut h = Vec::with_capacity(58);
    h.extend_from_slice(b"RIFF");
    h.extend_from_slice(&riff.to_le_bytes());
    h.extend_from_slice(b"WAVEfmt ");
    h.extend_from_slice(&format_len.to_le_bytes());
    h.extend_from_slice(&(if float { 3u16 } else { 1 }).to_le_bytes());
    h.extend_from_slice(&2u16.to_le_bytes());
    h.extend_from_slice(&rate.to_le_bytes());
    h.extend_from_slice(&(rate * block).to_le_bytes());
    h.extend_from_slice(&(block as u16).to_le_bytes());
    h.extend_from_slice(&(depth.bytes() * 8).to_le_bytes());
    if float {
        h.extend_from_slice(&0u16.to_le_bytes());
        h.extend_from_slice(b"fact");
        h.extend_from_slice(&4u32.to_le_bytes());
        h.extend_from_slice(&(frames.min(u64::from(u32::MAX)) as u32).to_le_bytes());
    }
    h.extend_from_slice(b"data");
    h.extend_from_slice(&data.to_le_bytes());
    h
}

/// The most bytes of sound a WAV file holds: its sizes are 32 bits, and
/// the header counts too.
pub const MOST: u64 = u32::MAX as u64 - 64;

/// A WAV export under way.
#[derive(Debug)]
pub struct Wav {
    depth: Depth,
    rate: u32,
    frames: u64,
    quantizer: Quantizer,
}

impl Wav {
    /// Starts one; the header comes first.
    pub fn start(depth: Depth, rate: u32) -> (Wav, Vec<u8>) {
        let wav = Wav {
            depth,
            rate,
            frames: 0,
            quantizer: Quantizer::new(if depth == Depth::Sixteen { 16 } else { 24 }),
        };
        (wav, header(depth, rate, 0))
    }

    /// The bytes of `interleaved` stereo frames.
    pub fn samples(&mut self, interleaved: &[f32]) -> Result<Vec<u8>, Error> {
        let frames = (interleaved.len() / 2) as u64;
        if (self.frames + frames) * 2 * u64::from(self.depth.bytes()) > MOST {
            return Err(Error::new(
                ErrorKind::Failed,
                "a WAV file holds at most 4 GB of sound: export fewer bars, or as FLAC",
            ));
        }
        self.frames += frames;
        let mut out = Vec::with_capacity(interleaved.len() * usize::from(self.depth.bytes()));
        for &v in interleaved {
            match self.depth {
                Depth::Sixteen => {
                    out.extend_from_slice(&(self.quantizer.sample(v) as i16).to_le_bytes());
                }
                Depth::TwentyFour => {
                    out.extend_from_slice(&self.quantizer.sample(v).to_le_bytes()[..3]);
                }
                Depth::Float => out.extend_from_slice(&v.to_le_bytes()),
            }
        }
        Ok(out)
    }

    /// The header with the length, to write over the first.
    pub fn finish(&self) -> Vec<u8> {
        header(self.depth, self.rate, self.frames)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headers_name_the_format_and_the_length() {
        let h = header(Depth::TwentyFour, 48_000, 48_000);
        assert_eq!(h.len(), 44);
        assert_eq!(&h[..4], b"RIFF");
        assert_eq!(&h[8..16], b"WAVEfmt ");
        assert_eq!(u16::from_le_bytes([h[20], h[21]]), 1, "PCM");
        assert_eq!(u16::from_le_bytes([h[22], h[23]]), 2, "stereo");
        assert_eq!(u32::from_le_bytes(h[24..28].try_into().unwrap()), 48_000);
        assert_eq!(
            u32::from_le_bytes(h[28..32].try_into().unwrap()),
            48_000 * 6
        );
        assert_eq!(u16::from_le_bytes([h[32], h[33]]), 6);
        assert_eq!(u16::from_le_bytes([h[34], h[35]]), 24);
        assert_eq!(&h[36..40], b"data");
        assert_eq!(
            u32::from_le_bytes(h[40..44].try_into().unwrap()),
            48_000 * 6
        );
        assert_eq!(
            u32::from_le_bytes(h[4..8].try_into().unwrap()),
            36 + 48_000 * 6
        );

        let f = header(Depth::Float, 44_100, 10);
        assert_eq!(f.len(), 58);
        assert_eq!(u16::from_le_bytes([f[20], f[21]]), 3, "IEEE float");
        assert_eq!(&f[38..42], b"fact");
        assert_eq!(u32::from_le_bytes(f[46..50].try_into().unwrap()), 10);
        assert_eq!(&f[50..54], b"data");
        assert_eq!(u32::from_le_bytes(f[54..58].try_into().unwrap()), 80);
        assert_eq!(u32::from_le_bytes(f[4..8].try_into().unwrap()), 58 - 8 + 80);
        // The header at the start and the one at the end are as long.
        assert_eq!(f.len(), header(Depth::Float, 44_100, 0).len());
    }

    #[test]
    fn samples_become_bytes_at_each_depth() {
        let (mut w, start) = Wav::start(Depth::TwentyFour, 48_000);
        assert_eq!(start.len(), 44);
        let b = w.samples(&[1.0, -1.0, 2.0, 0.0]).unwrap();
        assert_eq!(b.len(), 12);
        assert_eq!(&b[0..3], &[0xFF, 0xFF, 0x7F], "full scale");
        assert_eq!(&b[3..6], &[0x01, 0x00, 0x80], "minus full scale");
        assert_eq!(&b[6..9], &[0xFF, 0xFF, 0x7F], "louder is clipped");
        assert_eq!(&b[9..12], &[0, 0, 0]);
        let h = w.finish();
        assert_eq!(u32::from_le_bytes(h[40..44].try_into().unwrap()), 12);

        let (mut w, _) = Wav::start(Depth::Float, 48_000);
        let b = w.samples(&[1.5, -0.25]).unwrap();
        assert_eq!(f32::from_le_bytes(b[0..4].try_into().unwrap()), 1.5);
        assert_eq!(f32::from_le_bytes(b[4..8].try_into().unwrap()), -0.25);

        let (mut w, _) = Wav::start(Depth::Sixteen, 44_100);
        let b = w.samples(&[0.25; 2000]).unwrap();
        let values: Vec<i16> = b
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| i16::from_le_bytes(*c))
            .collect();
        assert!(values.iter().all(|v| (*v - 8192).abs() <= 1), "dithered");
    }

    #[test]
    fn past_four_gigabytes_the_export_stops() {
        let (mut w, _) = Wav::start(Depth::TwentyFour, 48_000);
        w.frames = MOST / 6 - 1;
        assert!(w.samples(&[0.0, 0.0]).is_ok());
        let e = w.samples(&[0.0; 4]).unwrap_err();
        assert!(e.message.contains("4 GB"), "{}", e.message);
    }
}
