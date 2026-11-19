// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Mateus Rocha de Paula and the OxiGuitar contributors

//! MP3 through rusty_mp3 (Apache-2.0): MPEG-1 Layer III, stereo, at a
//! variable bitrate by LAME's V numbers or a constant one, at 44.1 or
//! 48 kHz.
//!
//! The first frame is a LAME-style Info (constant) or Xing (variable) tag,
//! whose counts are known only at the end: the encoder completes it then,
//! as long as the first, and OxiGuitar writes it over the start, so
//! players that read it play exactly the song's length (gapless) and seek
//! right.

use oxiguitar_plugin::{Error, ErrorKind};
use rusty_mp3::{Mp3Encoder, Mp3EncoderConfig};

/// An MP3 export under way.
pub struct Mp3 {
    encoder: Mp3Encoder,
    rate: u32,
}

impl std::fmt::Debug for Mp3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mp3")
            .field("rate", &self.rate)
            .finish_non_exhaustive()
    }
}

fn failed(e: impl std::fmt::Display) -> Error {
    Error::new(ErrorKind::Failed, format!("the MP3 encoder failed: {e}"))
}

/// The encoder's settings for the quality `key` (`choices`): `v0`, `v2`,
/// `v4` for variable bitrates, or a constant bitrate in kbps.
pub fn config(key: &str) -> Mp3EncoderConfig {
    match key.strip_prefix('v').and_then(|v| v.parse::<f32>().ok()) {
        Some(v) => Mp3EncoderConfig {
            bitrate_kbps: 0,
            vbr_quality: Some(rusty_mp3::vbr_quality_index(v)),
        },
        None => Mp3EncoderConfig {
            bitrate_kbps: key.parse().unwrap_or(192),
            vbr_quality: None,
        },
    }
}

impl Mp3 {
    /// Starts one; the frames, the tag first, come with the sound.
    pub fn start(quality: &str, rate: u32) -> Mp3 {
        Mp3 {
            encoder: Mp3Encoder::new(config(quality)),
            rate,
        }
    }

    /// Every frame the encoder has ready.
    fn drain(&mut self) -> Result<Vec<u8>, Error> {
        let mut out = Vec::new();
        loop {
            match self.encoder.next_packet() {
                Ok(frame) => out.extend(frame),
                Err(rusty_mp3::Error::Again | rusty_mp3::Error::Eof) => return Ok(out),
                Err(e) => return Err(failed(e)),
            }
        }
    }

    /// The frames `interleaved` stereo frames complete.
    pub fn samples(&mut self, interleaved: &[f32]) -> Result<Vec<u8>, Error> {
        self.encoder
            .push_pcm_f32(interleaved, 2, self.rate)
            .map_err(failed)?;
        self.drain()
    }

    /// The last frames, and the completed tag to write over the first.
    pub fn finish(mut self) -> Result<(Vec<u8>, Option<Vec<u8>>), Error> {
        self.encoder.finish();
        let tail = self.drain()?;
        Ok((tail, self.encoder.lame_tag_frame().map(<[u8]>::to_vec)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qualities() {
        assert!(config("v2").vbr_quality.is_some());
        let cbr = config("320");
        assert_eq!((cbr.bitrate_kbps, cbr.vbr_quality), (320, None));
    }

    /// The tag at the end is as long as the frame it is written over, and
    /// every frame starts with a frame sync.
    #[test]
    fn a_stream_in_pieces() {
        let mut mp3 = Mp3::start("v2", 44_100);
        let mut body = Vec::new();
        for piece in 0..40 {
            let sound: Vec<f32> = (0..2000)
                .map(|i| ((piece * 1000 + i / 2) as f32 * 0.03).sin() * 0.5)
                .collect();
            body.extend(mp3.samples(&sound).unwrap());
        }
        let (tail, tag) = mp3.finish().unwrap();
        body.extend(tail);
        let tag = tag.unwrap();
        assert!(body.len() > tag.len());
        assert_eq!(&body[..2], &tag[..2]);
        assert_eq!((body[0], body[1] & 0xE0), (0xFF, 0xE0), "frame sync");
    }
}
