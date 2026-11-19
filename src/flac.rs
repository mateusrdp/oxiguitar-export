// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Mateus Rocha de Paula and the OxiGuitar contributors

//! FLAC through rusty_flac (Apache-2.0), 16 or 24 bits at 44.1 or 48 kHz,
//! at its default compression level.
//!
//! The stream starts with a STREAMINFO whose totals are unknown (zero, as
//! the format allows); each piece of sound becomes the frames it
//! completes, and at the end the last, shorter frame comes out with the
//! real STREAMINFO (total samples, MD5, frame sizes), as long as the
//! first, which OxiGuitar writes over the start. rusty_flac's frames are
//! the ones it would write for the whole stream at once.
//!
//! flacenc, the older crate, was tried first: its 24-bit encoder
//! overflowed on loud plucked notes (a frame of 1.5 Gbit that it then
//! failed to allocate), so it isn't used.

use oxiguitar_plugin::{Error, ErrorKind};

use crate::pcm::Quantizer;

/// A FLAC export under way.
pub struct Flac {
    encoder: rusty_flac::Encoder,
    quantizer: Quantizer,
    /// The piece's samples as integers, kept to save allocations.
    ints: Vec<i32>,
}

impl std::fmt::Debug for Flac {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Flac").finish_non_exhaustive()
    }
}

fn failed(e: impl std::fmt::Display) -> Error {
    Error::new(ErrorKind::Failed, format!("the FLAC encoder failed: {e}"))
}

impl Flac {
    /// Starts one at `bits` (16 or 24) and `rate`; the stream's header
    /// comes first.
    pub fn start(bits: u32, rate: u32) -> Result<(Flac, Vec<u8>), Error> {
        let encoder = rusty_flac::Encoder::new(rate, 2, bits).map_err(failed)?;
        let header = encoder.stream_info_placeholder().stream_header();
        Ok((
            Flac {
                encoder,
                quantizer: Quantizer::new(bits),
                ints: Vec::new(),
            },
            header,
        ))
    }

    /// The frames `interleaved` stereo frames complete.
    pub fn samples(&mut self, interleaved: &[f32]) -> Result<Vec<u8>, Error> {
        self.ints.clear();
        self.ints
            .extend(interleaved.iter().map(|&v| self.quantizer.sample(v)));
        self.encoder.push_interleaved(&self.ints).map_err(failed)?;
        Ok(self
            .encoder
            .take_frames()
            .into_iter()
            .flat_map(|f| f.data)
            .collect())
    }

    /// The last frames, and the stream's header with its totals.
    pub fn finish(self) -> (Vec<u8>, Vec<u8>) {
        let (last, info) = self.encoder.finish_frames();
        (
            last.into_iter().flat_map(|f| f.data).collect(),
            info.stream_header(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The header at the end is as long as the first and names the
    /// samples; a quiet stream is far smaller than its PCM.
    #[test]
    fn a_stream_in_pieces() {
        let (mut flac, first) = Flac::start(24, 48_000).unwrap();
        assert_eq!(&first[..4], b"fLaC");
        let mut body = Vec::new();
        for piece in 0..50 {
            let sound: Vec<f32> = (0..1000)
                .map(|i| ((piece * 1000 + i) as f32 * 0.01).sin() * 0.5)
                .collect();
            body.extend(flac.samples(&sound).unwrap());
        }
        let (tail, header) = flac.finish();
        body.extend(tail);
        assert_eq!(header.len(), first.len());
        // STREAMINFO: 36 bits of total samples end its 18th byte.
        let info = &header[8..];
        let total = (u64::from(info[13] & 0x0F) << 32)
            | u64::from(u32::from_be_bytes(info[14..18].try_into().unwrap()));
        assert_eq!(total, 25_000);
        assert!(body.len() < 25_000 * 6 / 2, "{}", body.len());
    }
}
