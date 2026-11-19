// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Mateus Rocha de Paula and the OxiGuitar contributors

//! Samples as integers: 16 bits get triangular dither of one step, so
//! quiet passages fade into noise rather than into steps; 24 bits are
//! rounded. Both are clipped at full scale.

/// Turns the mixer's samples (full scale ±1) into integers of one depth.
#[derive(Debug)]
pub struct Quantizer {
    bits: u32,
    /// The dither's generator (xorshift), the same for every export so a
    /// file comes out the same twice.
    noise: u32,
}

impl Quantizer {
    /// For 16 or 24 bits.
    pub fn new(bits: u32) -> Quantizer {
        Quantizer {
            bits,
            noise: 0x9E37_79B9,
        }
    }

    fn next(&mut self) -> f32 {
        let mut x = self.noise;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.noise = x;
        x as f32 / u32::MAX as f32
    }

    /// `v` as an integer sample.
    pub fn sample(&mut self, v: f32) -> i32 {
        if self.bits == 16 {
            let dither = self.next() - self.next();
            (v * 32_767.0 + dither).round().clamp(-32_768.0, 32_767.0) as i32
        } else {
            (v.clamp(-1.0, 1.0) * 8_388_607.0).round() as i32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_scale_and_dither() {
        let mut q = Quantizer::new(24);
        assert_eq!(q.sample(1.0), 8_388_607);
        assert_eq!(q.sample(-2.0), -8_388_607, "clipped");
        let mut q = Quantizer::new(16);
        let values: Vec<i32> = (0..2000).map(|_| q.sample(0.25)).collect();
        assert!(values.iter().all(|v| (*v - 8192).abs() <= 1), "{values:?}");
        let mean = values.iter().map(|v| f64::from(*v)).sum::<f64>() / 2000.0;
        assert!((mean - 8191.75).abs() < 0.1, "{mean}");
    }
}
