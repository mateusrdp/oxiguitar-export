// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Mateus Rocha de Paula and the OxiGuitar contributors

//! Opus in Ogg (RFC 7845) through opus-pure (BSD-3-Clause), stereo at
//! 48 kHz, the rate Opus always decodes to, as music (CELT) at the
//! bitrate picked.
//!
//! The stream's pages come out as the packets fill them. The encoder's
//! lookahead is the header's pre-skip; at the end the last packet's
//! granule is cut back (the end trim of RFC 7845 §4.4) so players play
//! exactly the song's length.

use opus_pure::{
    Application, MAX_PACKET_BYTES, OggOpusWriter, OpusEncoder, OpusHead, RateControl, Signal,
};
use oxiguitar_plugin::{Error, ErrorKind};

/// The rate Opus encodes at.
pub const RATE: u32 = 48_000;

/// Frames a packet: 20 ms.
const FRAME: usize = 960;

/// An Opus export under way.
pub struct Opus {
    encoder: OpusEncoder,
    writer: OggOpusWriter<Vec<u8>>,
    /// Samples waiting for a whole packet.
    pending: Vec<f32>,
    /// Stereo frames handed over, and the header's pre-skip.
    frames: u64,
    pre_skip: u64,
    packet: Vec<u8>,
}

impl std::fmt::Debug for Opus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Opus")
            .field("frames", &self.frames)
            .finish_non_exhaustive()
    }
}

fn failed(e: impl std::fmt::Display) -> Error {
    Error::new(ErrorKind::Failed, format!("the Opus encoder failed: {e}"))
}

impl Opus {
    /// Starts one at `kbps`; the stream's two header pages come first.
    pub fn start(kbps: u32) -> Result<(Opus, Vec<u8>), Error> {
        let mut encoder = OpusEncoder::new(RATE as i32, 2, Application::Audio).map_err(failed)?;
        encoder.bitrate_bps = i32::try_from(kbps * 1000).unwrap_or(160_000);
        encoder.signal_type = Some(Signal::Music);
        // A file wants bits where the music needs them.
        encoder.rate_control = RateControl::Vbr;
        let head = OpusHead::for_encoder(&encoder, RATE);
        let pre_skip = u64::from(head.pre_skip);
        let mut writer = OggOpusWriter::new(Vec::new(), head).map_err(failed)?;
        let first = writer.get_mut().map(std::mem::take).unwrap_or_default();
        Ok((
            Opus {
                encoder,
                writer,
                pending: Vec::with_capacity(FRAME * 4),
                frames: 0,
                pre_skip,
                packet: vec![0; MAX_PACKET_BYTES],
            },
            first,
        ))
    }

    /// Encodes the first packet's worth of `pending` and writes it, its
    /// granule advancing by `duration` frames.
    fn encode(&mut self, duration: u32) -> Result<(), Error> {
        let n = self
            .encoder
            .encode(&self.pending[..FRAME * 2], FRAME, &mut self.packet)
            .map_err(failed)?;
        self.pending.drain(..FRAME * 2);
        self.writer
            .write_packet_with_duration(&self.packet[..n], duration)
            .map_err(failed)
    }

    /// The pages `interleaved` stereo frames complete.
    pub fn samples(&mut self, interleaved: &[f32]) -> Result<Vec<u8>, Error> {
        self.frames += (interleaved.len() / 2) as u64;
        self.pending.extend_from_slice(interleaved);
        while self.pending.len() >= FRAME * 2 {
            self.encode(FRAME as u32)?;
        }
        Ok(self
            .writer
            .get_mut()
            .map(std::mem::take)
            .unwrap_or_default())
    }

    /// The last pages: what is still in the encoder comes out on silence,
    /// and the last packet's granule ends the stream at the song's end.
    pub fn finish(mut self) -> Result<Vec<u8>, Error> {
        // The encoder runs `pre_skip` frames behind: feed that much
        // silence, then make up a whole packet.
        let wanted = self.pending.len() / 2 + self.pre_skip as usize;
        let whole = wanted.div_ceil(FRAME).max(1) * FRAME;
        self.pending.resize(whole * 2, 0.0);
        let end = self.pre_skip + self.frames;
        while self.pending.len() > FRAME * 2 {
            self.encode(FRAME as u32)?;
        }
        let reached = u64::try_from(self.writer.granule()).unwrap_or(0);
        let last = end.saturating_sub(reached).min(FRAME as u64) as u32;
        self.encode(last)?;
        self.writer.finish().map_err(failed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The stream is Ogg pages from the first byte to the last, and the
    /// last page's granule is the pre-skip and the song.
    #[test]
    fn a_stream_in_pieces() {
        let (mut opus, mut file) = Opus::start(160).unwrap();
        assert_eq!(&file[..4], b"OggS");
        for piece in 0..30 {
            let sound: Vec<f32> = (0..2002)
                .map(|i| ((piece * 1001 + i / 2) as f32 * 0.03).sin() * 0.5)
                .collect();
            file.extend(opus.samples(&sound).unwrap());
        }
        let pre_skip = opus.pre_skip;
        file.extend(opus.finish().unwrap());
        let last = file
            .windows(4)
            .rposition(|w| w == b"OggS")
            .expect("a last page");
        assert_eq!(file[last + 5] & 4, 4, "end of stream");
        let granule = u64::from_le_bytes(file[last + 6..last + 14].try_into().unwrap());
        assert_eq!(granule, pre_skip + 30 * 1001);
    }
}
